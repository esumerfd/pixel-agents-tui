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

    render_scene(frame, h_chunks[0], scene);
    render_activity_panel(frame, h_chunks[1], agents, scene);
    render_status_bar(frame, v_chunks[1], agents);

    if show_help {
        help::render_help(frame);
    }
}

fn render_scene(frame: &mut Frame, area: Rect, scene: &Scene) {
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

    // Render characters sorted by row (z-order)
    let mut chars: Vec<&Character> = scene.characters.values().collect();
    chars.sort_by_key(|ch| ch.row);

    for ch in chars {
        render_character(frame, inner, ch);
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

fn render_character(frame: &mut Frame, area: Rect, ch: &Character) {
    let (grid, half_w, row_offset) = if ch.is_subagent {
        (sprites::get_subagent_grid(ch), sprites::SUBAGENT_WIDTH / 2, 2u16)
    } else {
        (sprites::get_character_grid(ch), sprites::CHARACTER_WIDTH / 2, 2u16)
    };
    let base_col = ch.col.saturating_sub(half_w);

    for (dy, sprite_row) in grid.iter().enumerate() {
        for (dx, cell) in sprite_row.iter().enumerate() {
            if cell.ch == ' ' && cell.bg.is_none() {
                continue;
            }
            let x = area.x + base_col + dx as u16;
            let y = area.y + ch.row.saturating_sub(row_offset) + dy as u16;
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

fn render_activity_panel(
    frame: &mut Frame,
    area: Rect,
    agents: &std::collections::HashMap<u32, AgentState>,
    scene: &Scene,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 120)))
        .title(" Activity ")
        .title_style(Style::default().fg(Color::Rgb(180, 200, 255)));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build ordered list: parents sorted by folder name, then their subagents nested
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
        let mut sub_ids: Vec<u32> = agents.iter()
            .filter(|(_, a)| a.parent_id == Some(*pid))
            .map(|(id, _)| *id)
            .collect();
        sub_ids.sort();
        ordered.extend(sub_ids);
    }

    let mut row = 0u16;
    for id in &ordered {
        if row >= inner.height {
            break;
        }

        let agent = match agents.get(id) {
            Some(a) => a,
            None => continue,
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
        let max_name_len = (inner.width as usize).saturating_sub(12);

        let line = if agent.is_subagent {
            let name: String = format!("\u{2514} subagent").chars().take(max_name_len).collect();
            Line::from(vec![
                Span::styled(format!("   {status_icon} "), Style::default().fg(status_color)),
                Span::styled("\u{2588} ", Style::default().fg(palette_color)),
                Span::styled(name, Style::default().fg(Color::Rgb(120, 125, 150))),
                Span::styled(" \u{2192} ", Style::default().fg(Color::Rgb(80, 80, 120))),
                Span::styled(status_text, Style::default().fg(status_color)),
            ])
        } else {
            let folder = agent.folder_name.as_deref().unwrap_or("unknown");
            let short_folder: String = folder.chars().take(max_name_len).collect();
            Line::from(vec![
                Span::styled(format!(" {status_icon} "), Style::default().fg(status_color)),
                Span::styled("\u{2588} ", Style::default().fg(palette_color)),
                Span::styled(short_folder, Style::default().fg(Color::Rgb(160, 170, 200))),
                Span::styled(" \u{2192} ", Style::default().fg(Color::Rgb(80, 80, 120))),
                Span::styled(status_text, Style::default().fg(status_color)),
            ])
        };

        frame.render_widget(
            Paragraph::new(line),
            Rect::new(inner.x, inner.y + row, inner.width, 1),
        );
        row += 1;
    }
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
        Span::styled(" | q:quit  r:refresh  ?:help ", Style::default().fg(Color::Rgb(80, 80, 120))),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(80, 80, 120)));

    frame.render_widget(Paragraph::new(Line::from(spans)).block(block).wrap(Wrap { trim: true }), area);
}
