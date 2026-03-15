use ratatui::{
    layout::{Constraint, Direction as LayoutDir, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::help;
use crate::layout::{FurnitureKind, Tile};
use crate::scene::Scene;
use crate::sprites;
use crate::types::*;

pub fn render(
    frame: &mut Frame,
    scene: &Scene,
    agents: &std::collections::HashMap<u32, AgentState>,
    show_help: bool,
    help_scroll: u16,
    activity_scroll: u16,
    activity_cursor: u16,
    collapsed: &std::collections::HashSet<u32>,
) {
    let area = frame.area();

    // Vertical split: main content on top, status bar at bottom
    let v_chunks = Layout::default()
        .direction(LayoutDir::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    // Horizontal split: scene left, activity panel right
    let h_chunks = Layout::default()
        .direction(LayoutDir::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(v_chunks[0]);

    render_scene(frame, h_chunks[0], scene, agents, collapsed);
    render_activity_panel(frame, h_chunks[1], agents, scene, activity_scroll, activity_cursor, collapsed);
    render_status_bar(frame, v_chunks[1], agents);

    if show_help {
        help::render_help(frame, help_scroll);
    }
}

fn render_scene(
    frame: &mut Frame,
    area: Rect,
    scene: &Scene,
    agents: &std::collections::HashMap<u32, AgentState>,
    collapsed: &std::collections::HashSet<u32>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 120)))
        .title(" Pixel Agents ")
        .title_style(Style::default().fg(Color::Rgb(140, 180, 255)).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let layout = &scene.layout;

    // Render tiles (floor, walls, void)
    for row in 0..layout.rows.min(inner.height) {
        for col in 0..layout.cols.min(inner.width) {
            let tile = layout.get_tile(col, row);
            let x = inner.x + col;
            let y = inner.y + row;

            let (ch, fg, bg) = match tile {
                Tile::Void => continue,
                Tile::Wall => ('\u{2588}', Color::Rgb(60, 65, 85), None),
                Tile::Floor => {
                    if (col + row) % 2 == 0 {
                        ('\u{2591}', Color::Rgb(55, 58, 75), Some(Color::Rgb(35, 38, 50)))
                    } else {
                        (' ', Color::Reset, Some(Color::Rgb(38, 40, 55)))
                    }
                }
                Tile::Corridor => {
                    if (col + row) % 2 == 0 {
                        ('\u{2591}', Color::Rgb(50, 55, 80), Some(Color::Rgb(30, 35, 55)))
                    } else {
                        (' ', Color::Reset, Some(Color::Rgb(33, 38, 58)))
                    }
                }
                Tile::Carpet => {
                    ('\u{2591}', Color::Rgb(70, 55, 65), Some(Color::Rgb(45, 35, 42)))
                }
                Tile::TileFloor => {
                    if (col + row) % 2 == 0 {
                        ('\u{2591}', Color::Rgb(65, 65, 55), Some(Color::Rgb(40, 42, 35)))
                    } else {
                        (' ', Color::Reset, Some(Color::Rgb(42, 44, 38)))
                    }
                }
            };

            let mut style = Style::default().fg(fg);
            if let Some(bg_color) = bg {
                style = style.bg(bg_color);
            }
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(ch.to_string(), style))),
                Rect::new(x, y, 1, 1),
            );
        }
    }

    // Render room labels on walls
    for (col, row, text) in &layout.room_labels {
        let x = inner.x + col;
        let y = inner.y + row;
        if y < inner.y + inner.height && x + (text.len() as u16) <= inner.x + inner.width {
            let style = Style::default()
                .fg(Color::Rgb(90, 95, 115))
                .add_modifier(Modifier::BOLD);
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(*text, style))),
                Rect::new(x, y, text.len() as u16, 1),
            );
        }
    }

    // Render furniture
    for f in &layout.furniture {
        render_furniture(frame, inner, f.kind, f.col, f.row);
    }

    // Render characters sorted by row (z-order), hiding collapsed subagents
    let mut chars: Vec<&Character> = scene.characters.values()
        .filter(|ch| {
            !agents.get(&ch.id)
                .and_then(|a| a.parent_id)
                .is_some_and(|pid| collapsed.contains(&pid))
        })
        .collect();
    chars.sort_by_key(|ch| ch.row);

    for ch in chars {
        render_character(frame, inner, ch, scene);
    }
}

fn render_furniture(frame: &mut Frame, area: Rect, kind: FurnitureKind, col: u16, row: u16) {
    let grid = sprites::get_furniture_grid(kind);
    for (dy, sprite_row) in grid.iter().enumerate() {
        for (dx, cell) in sprite_row.iter().enumerate() {
            if cell.ch == ' ' && cell.bg.is_none() {
                continue;
            }
            let x = area.x + col + dx as u16;
            let y = area.y + row + dy as u16;
            if x >= area.x + area.width || y >= area.y + area.height {
                continue;
            }
            let mut style = Style::default().fg(cell.fg);
            if let Some(bg) = cell.bg {
                style = style.bg(bg);
            }
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(cell.ch.to_string(), style))),
                Rect::new(x, y, 1, 1),
            );
        }
    }
}

fn render_character(frame: &mut Frame, area: Rect, ch: &Character, scene: &Scene) {
    let (grid, half_w, row_offset) = if ch.is_subagent {
        (sprites::get_subagent_grid(ch), sprites::SUBAGENT_WIDTH / 2, 4u16)
    } else {
        (sprites::get_character_grid(ch), sprites::CHARACTER_WIDTH / 2, 4u16)
    };
    let base_col = ch.col.saturating_sub(half_w);

    for (dy, sprite_row) in grid.iter().enumerate() {
        for (dx, cell) in sprite_row.iter().enumerate() {
            if cell.ch == ' ' && cell.bg.is_none() {
                continue;
            }
            let tile_col = base_col + dx as u16;
            let tile_row = ch.row.saturating_sub(row_offset) + dy as u16;
            let x = area.x + tile_col;
            let y = area.y + tile_row;
            if x >= area.x + area.width || y >= area.y + area.height {
                continue;
            }
            // Only clip sprite pixels that land on Void (outside the building).
            // Walls, floors, furniture etc. are all inside — sprites should draw over them.
            let tile = scene.layout.get_tile(tile_col, tile_row);
            if tile == crate::layout::Tile::Void {
                continue;
            }
            let mut style = Style::default().fg(cell.fg);
            if let Some(bg) = cell.bg {
                style = style.bg(bg);
            }
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(cell.ch.to_string(), style))),
                Rect::new(x, y, 1, 1),
            );
        }
    }
}

fn render_activity_panel(
    frame: &mut Frame,
    area: Rect,
    agents: &std::collections::HashMap<u32, AgentState>,
    scene: &Scene,
    scroll_offset: u16,
    cursor_pos: u16,
    collapsed: &std::collections::HashSet<u32>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 120)))
        .title(" Activity ")
        .title_style(Style::default().fg(Color::Rgb(180, 200, 255)));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let ordered = build_activity_list(agents, collapsed);

    let total_items = ordered.len() as u16;
    let visible_height = inner.height;

    // Don't scroll if everything fits
    let effective_scroll = if total_items <= visible_height { 0 } else { scroll_offset };

    let mut row = 0u16;
    let mut item_idx = 0u16;
    for id in &ordered {
        if item_idx < effective_scroll {
            item_idx += 1;
            continue;
        }
        if row >= visible_height {
            break;
        }

        let agent = match agents.get(id) {
            Some(a) => a,
            None => continue,
        };

        let is_selected = item_idx == cursor_pos;
        let row_bg = if is_selected {
            Some(Color::Rgb(40, 45, 65))
        } else {
            None
        };

        let palette_color = scene.characters.get(id)
            .map(|ch| sprites::palette_color(ch.palette))
            .unwrap_or(Color::White);

        let (status_icon, status_color) = match agent.status {
            AgentStatus::Active => ("\u{25B6}", Color::Rgb(80, 200, 120)),
            AgentStatus::WaitingPermission => ("\u{25CF}", Color::Rgb(220, 180, 50)),
            AgentStatus::Idle => ("\u{25CB}", Color::Rgb(100, 100, 140)),
        };

        let status_text = agent.current_status_text();
        let max_name_len = (inner.width as usize).saturating_sub(16);

        let apply_bg = |mut s: Style| -> Style {
            if let Some(bg) = row_bg { s = s.bg(bg); }
            s
        };

        let line = if agent.is_subagent {
            let display_name = agent.member_name.as_deref().unwrap_or("subagent");
            let name: String = format!("\u{2514} {display_name}").chars().take(max_name_len).collect();
            let name_color = if agent.member_name.is_some() {
                Color::Rgb(140, 145, 170)
            } else {
                Color::Rgb(120, 125, 150)
            };
            Line::from(vec![
                Span::styled(format!("   {status_icon} "), apply_bg(Style::default().fg(status_color))),
                Span::styled("\u{2588} ", apply_bg(Style::default().fg(palette_color))),
                Span::styled(name, apply_bg(Style::default().fg(name_color))),
                Span::styled(" \u{2192} ", apply_bg(Style::default().fg(Color::Rgb(80, 80, 120)))),
                Span::styled(status_text, apply_bg(Style::default().fg(status_color))),
            ])
        } else {
            let has_children = agents.values().any(|a| a.parent_id == Some(*id));
            let collapse_indicator = if has_children {
                if collapsed.contains(id) { "\u{25B8}" } else { "\u{25BE}" }
            } else {
                " "
            };

            let display_name = agent.member_name.as_deref()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| agent.folder_name.as_deref().unwrap_or("unknown"));
            let short_name: String = display_name.chars().take(max_name_len).collect();
            Line::from(vec![
                Span::styled(collapse_indicator, apply_bg(Style::default().fg(Color::Rgb(120, 130, 160)))),
                Span::styled(format!("{status_icon} "), apply_bg(Style::default().fg(status_color))),
                Span::styled("\u{2588} ", apply_bg(Style::default().fg(palette_color))),
                Span::styled(short_name, apply_bg(Style::default().fg(Color::Rgb(160, 170, 200)))),
                Span::styled(" \u{2192} ", apply_bg(Style::default().fg(Color::Rgb(80, 80, 120)))),
                Span::styled(status_text, apply_bg(Style::default().fg(status_color))),
            ])
        };

        frame.render_widget(
            Paragraph::new(line),
            Rect::new(inner.x, inner.y + row, inner.width.saturating_sub(1), 1),
        );
        row += 1;
        item_idx += 1;
    }

    // Draw scrollbar only if content exceeds visible area
    if total_items > visible_height && visible_height > 0 {
        let track_height = visible_height as f64;
        let thumb_height = ((visible_height as f64 / total_items as f64) * track_height)
            .max(1.0) as u16;
        let max_scroll = (total_items - visible_height).max(1);
        let thumb_pos = ((effective_scroll as f64 / max_scroll as f64)
            * (track_height - thumb_height as f64)) as u16;

        let bar_x = inner.x + inner.width - 1;
        for y in 0..visible_height {
            let in_thumb = y >= thumb_pos && y < thumb_pos + thumb_height;
            let (ch, color) = if in_thumb {
                ('\u{2588}', Color::Rgb(100, 120, 160))
            } else {
                ('\u{2502}', Color::Rgb(50, 50, 70))
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    ch.to_string(),
                    Style::default().fg(color),
                ))),
                Rect::new(bar_x, inner.y + y, 1, 1),
            );
        }
    }
}

/// Build the ordered activity list, respecting collapsed state.
/// Parents sorted by folder name, then their children (unless collapsed).
pub fn build_activity_list(
    agents: &std::collections::HashMap<u32, AgentState>,
    collapsed: &std::collections::HashSet<u32>,
) -> Vec<u32> {
    let mut parent_ids: Vec<u32> = agents.iter()
        .filter(|(_, a)| !a.is_subagent)
        .map(|(id, _)| *id)
        .collect();
    parent_ids.sort_by(|a, b| {
        let name_a = agents.get(a).and_then(|ag| ag.folder_name.as_deref()).unwrap_or("");
        let name_b = agents.get(b).and_then(|ag| ag.folder_name.as_deref()).unwrap_or("");
        name_a.to_lowercase().cmp(&name_b.to_lowercase())
    });

    let mut ordered: Vec<u32> = Vec::new();
    for pid in &parent_ids {
        ordered.push(*pid);
        if collapsed.contains(pid) {
            continue;
        }
        let mut sub_ids: Vec<u32> = agents.iter()
            .filter(|(_, a)| a.parent_id == Some(*pid))
            .map(|(id, _)| *id)
            .collect();
        sub_ids.sort();
        ordered.extend(sub_ids);
    }
    ordered
}

fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    agents: &std::collections::HashMap<u32, AgentState>,
) {
    let active_count = agents.values().filter(|a| a.status == AgentStatus::Active).count();
    let total_count = agents.len();

    let spans = vec![
        Span::styled(format!(" Agents: {total_count} "), Style::default().fg(Color::Rgb(140, 180, 255))),
        Span::styled(" | ", Style::default().fg(Color::Rgb(80, 80, 120))),
        Span::styled(format!("Active: {active_count} "), Style::default().fg(Color::Rgb(80, 200, 120))),
        Span::styled(format!("Idle: {} ", total_count - active_count), Style::default().fg(Color::Rgb(100, 100, 140))),
        Span::styled(" | q:quit  j/k:select  \u{21B5}:fold  ?:help ", Style::default().fg(Color::Rgb(80, 80, 120))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 120)));

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block).wrap(Wrap { trim: true }), area);
}
