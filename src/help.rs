use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help(frame: &mut Frame, scroll: u16) {
    let area = frame.area();

    let panel_w = 64u16.min(area.width.saturating_sub(4));
    let panel_h = 42u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(panel_w)) / 2;
    let y = area.y + (area.height.saturating_sub(panel_h)) / 2;
    let panel = Rect::new(x, y, panel_w, panel_h);

    frame.render_widget(Clear, panel);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(140, 180, 255)))
        .title(" Help ")
        .title_style(Style::default().fg(Color::Rgb(200, 220, 255)).add_modifier(Modifier::BOLD));

    let dim = Style::default().fg(Color::Rgb(100, 100, 140));
    let label = Style::default().fg(Color::Rgb(180, 190, 220));
    let _green = Style::default().fg(Color::Rgb(80, 200, 120));
    let _blue = Style::default().fg(Color::Rgb(100, 160, 220));
    let _yellow = Style::default().fg(Color::Rgb(220, 180, 50));
    let _white = Style::default().fg(Color::Rgb(200, 200, 220));
    let heading = Style::default().fg(Color::Rgb(140, 180, 255)).add_modifier(Modifier::BOLD);

    // Sprite preview colors — agent and subagent share shirt, differ in skin/hair
    let skin = Color::Rgb(255, 213, 170);
    let hair = Color::Rgb(80, 50, 30);
    let sub_skin = Color::Rgb(195, 150, 100);
    let sub_hair = Color::Rgb(40, 25, 15);
    let shirt = Color::Rgb(70, 130, 200);
    let pants = Color::Rgb(50, 50, 80);
    let eyes = Color::Rgb(40, 40, 40);
    let s = |fg| Style::default().fg(fg);
    let sb = |fg, bg| Style::default().fg(fg).bg(bg);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Character States", heading)),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Typing at desk    ", label),
            Span::styled("Arms animate while writing code", dim),
        ]),
        Line::from(vec![
            Span::styled("    Reading at desk   ", label),
            Span::styled("Holding a book while reviewing code", dim),
        ]),
        Line::from(vec![
            Span::styled("    Standing idle     ", label),
            Span::styled("Turn ended, wanders then rests", dim),
        ]),
        Line::from(vec![
            Span::styled("    Walking           ", label),
            Span::styled("Legs animate, pathfinds through doors", dim),
        ]),
        Line::from(vec![
            Span::styled("    Sitting on couch  ", label),
            Span::styled("Resting in the lounge when idle", dim),
        ]),
        Line::from(vec![
            Span::styled("    Using vending     ", label),
            Span::styled("Grabbing a snack, arm wiggles", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Agent Types", heading)),
        Line::from(""),
        // Agent preview row 1 (hair)
        Line::from(vec![
            Span::styled("    ", dim),
            Span::styled(" ", dim),
            Span::styled("\u{2584}\u{2584}\u{2584}", s(hair)),
            Span::styled("  Agent (5\u{00D7}5)     ", label),
            Span::styled("Main Claude Code session", dim),
        ]),
        // Agent preview row 2 (face)
        Line::from(vec![
            Span::styled("    ", dim),
            Span::styled(" ", dim),
            Span::styled("\u{25CF}", sb(eyes, skin)),
            Span::styled("\u{2588}", s(skin)),
            Span::styled("\u{25CF}", sb(eyes, skin)),
            Span::styled("  ", dim),
            Span::styled("                 ", label),
            Span::styled("Seated at assigned desk", dim),
        ]),
        // Agent preview row 3 (shirt)
        Line::from(vec![
            Span::styled("    ", dim),
            Span::styled(" ", dim),
            Span::styled("\u{2588}\u{2588}\u{2588}", s(shirt)),
            Span::styled("  ", dim),
            Span::styled("                 ", label),
            Span::styled("Shirt color matches activity panel", dim),
        ]),
        // Agent preview row 4 (pants)
        Line::from(vec![
            Span::styled("    ", dim),
            Span::styled(" ", dim),
            Span::styled("\u{2588}", s(pants)),
            Span::styled(" ", dim),
            Span::styled("\u{2588}", s(pants)),
            Span::styled("", dim),
        ]),
        Line::from(""),
        // Subagent preview row 1 (hair)
        Line::from(vec![
            Span::styled("     ", dim),
            Span::styled("\u{2584}", s(sub_hair)),
            Span::styled("   Subagent (3\u{00D7}4)  ", label),
            Span::styled("Team/spawned agent (smaller)", dim),
        ]),
        // Subagent preview row 2 (face)
        Line::from(vec![
            Span::styled("     ", dim),
            Span::styled("\u{25CF}", sb(eyes, sub_skin)),
            Span::styled("   ", dim),
            Span::styled("                  ", label),
            Span::styled("Same shirt, different person", dim),
        ]),
        // Subagent preview row 3 (shirt)
        Line::from(vec![
            Span::styled("     ", dim),
            Span::styled("\u{2588}", s(shirt)),
            Span::styled("   ", dim),
            Span::styled("                  ", label),
            Span::styled("Indented under parent in panel", dim),
        ]),
        // Subagent preview row 4 (pants)
        Line::from(vec![
            Span::styled("     ", dim),
            Span::styled("\u{2588}", s(pants)),
            Span::styled("", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Activity Panel", heading)),
        Line::from(""),
        Line::from(vec![
            Span::styled("    \u{25B6}", label),
            Span::styled("  Active     ", label),
            Span::styled("Agent using tools (coding, reading, etc.)", dim),
        ]),
        Line::from(vec![
            Span::styled("    \u{25CF}", label),
            Span::styled("  Waiting    ", label),
            Span::styled("Needs user permission to proceed", dim),
        ]),
        Line::from(vec![
            Span::styled("    \u{25CB}", label),
            Span::styled("  Idle       ", label),
            Span::styled("Between turns, agent resting", dim),
        ]),
        Line::from(vec![
            Span::styled("    \u{2514}", label),
            Span::styled("  Subagent   ", label),
            Span::styled("Nested under parent in activity list", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Controls", heading)),
        Line::from(""),
        Line::from(vec![
            Span::styled("    q", label),
            Span::styled(" / ", dim),
            Span::styled("Esc", label),
            Span::styled("    Quit", dim),
        ]),
        Line::from(vec![
            Span::styled("    \u{2191}\u{2193}", label),
            Span::styled(" / ", dim),
            Span::styled("j k", label),
            Span::styled("    Move cursor in activity panel", dim),
        ]),
        Line::from(vec![
            Span::styled("    h", label),
            Span::styled("          Collapse agent (from any child)", dim),
        ]),
        Line::from(vec![
            Span::styled("    l", label),
            Span::styled("          Expand agent tree", dim),
        ]),
        Line::from(vec![
            Span::styled("    Enter", label),
            Span::styled("      Toggle collapse/expand", dim),
        ]),
        Line::from(vec![
            Span::styled("    d", label),
            Span::styled(" / ", dim),
            Span::styled("u", label),
            Span::styled("        Page down / up", dim),
        ]),
        Line::from(vec![
            Span::styled("    gg", label),
            Span::styled("         Go to top of list", dim),
        ]),
        Line::from(vec![
            Span::styled("    G", label),
            Span::styled("          Go to bottom of list", dim),
        ]),
        Line::from(vec![
            Span::styled("    r", label),
            Span::styled("          Refresh agent list", dim),
        ]),
        Line::from(vec![
            Span::styled("    ?", label),
            Span::styled("          Toggle this help", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  j/k:scroll  ? or Esc:close", dim)),
    ];

    // Clamp scroll so we don't scroll past content
    let content_lines = lines.len() as u16;
    let inner_height = panel_h.saturating_sub(2); // account for borders
    let max_scroll = content_lines.saturating_sub(inner_height);
    let effective_scroll = scroll.min(max_scroll);

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((effective_scroll, 0));
    frame.render_widget(paragraph, panel);
}
