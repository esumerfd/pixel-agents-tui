mod constants;
mod help;
mod layout;
mod pathfinding;
mod renderer;
mod scene;
mod sprites;
mod transcript;
mod types;
mod watcher;

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::constants::*;
use crate::types::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut agents: HashMap<u32, AgentState> = HashMap::new();
    let mut known_files: HashSet<PathBuf> = HashSet::new();
    let mut next_id: u32 = 1;
    let mut show_help = false;
    let mut activity_scroll: u16 = 0;
    let mut activity_cursor: u16 = 0;
    let mut collapsed: HashSet<u32> = HashSet::new();
    let mut pending_g = false;
    let mut help_scroll: u16 = 0;

    // Initial scan for agents (to get count for desk layout)
    let initial_events = watcher::scan_for_agents(&mut known_files, &mut agents, &mut next_id);

    // Build office with desk count: top-level agents + named team members (they get their own desks)
    let top_level_count = agents.values()
        .filter(|a| !a.is_subagent || a.member_name.is_some())
        .count();
    let office = layout::build_office(top_level_count);
    let mut scene = scene::Scene::new(office);

    scene.handle_events(&initial_events, &agents);

    for id in agents.keys().copied().collect::<Vec<_>>() {
        scene.ensure_character(id, &agents);
    }

    // Read existing content from all agent files to get current state.
    // For stale files (not modified recently), skip to end to avoid replaying old history.
    let agent_ids: Vec<u32> = agents.keys().copied().collect();
    for id in agent_ids {
        if let Some(agent) = agents.get_mut(&id) {
            let is_stale = std::fs::metadata(&agent.jsonl_file)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|mtime| mtime.elapsed().ok())
                .is_some_and(|age| age.as_secs() >= STALE_ACTIVE_TIMEOUT_SECS);

            if is_stale {
                // Skip to end — file is old, don't replay history
                if let Ok(meta) = std::fs::metadata(&agent.jsonl_file) {
                    agent.file_offset = meta.len();
                }
            } else {
                let events = watcher::read_new_lines(agent);
                scene.handle_events(&events, &agents);
            }
        }
    }

    let tick_duration = Duration::from_millis(TICK_RATE_MS);
    let poll_duration = Duration::from_millis(FILE_POLL_INTERVAL_MS);
    let mut last_tick = Instant::now();
    let mut last_poll = Instant::now();

    loop {
        // Auto-scroll to keep cursor visible
        let list_len = renderer::build_activity_list(&agents, &collapsed).len() as u16;
        let term_size = terminal.size()?;
        let panel_visible = term_size.height.saturating_sub(5);
        if list_len <= panel_visible {
            activity_scroll = 0;
        } else {
            if activity_cursor < activity_scroll {
                activity_scroll = activity_cursor;
            }
            if panel_visible > 0 && activity_cursor >= activity_scroll + panel_visible {
                activity_scroll = activity_cursor - panel_visible + 1;
            }
        }
        // Clamp cursor to list bounds
        if list_len > 0 {
            activity_cursor = activity_cursor.min(list_len - 1);
        }

        terminal.draw(|frame| {
            renderer::render(frame, &scene, &agents, show_help, help_scroll, activity_scroll, activity_cursor, &collapsed);
        })?;

        let timeout = tick_duration.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if show_help {
                        match key.code {
                            KeyCode::Char('?') | KeyCode::Esc => show_help = false,
                            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                            KeyCode::Up | KeyCode::Char('k') => {
                                help_scroll = help_scroll.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                help_scroll += 1;
                            }
                            KeyCode::Char('u') => {
                                let page = terminal.size()?.height.saturating_sub(8) / 2;
                                help_scroll = help_scroll.saturating_sub(page);
                            }
                            KeyCode::Char('d') => {
                                let page = terminal.size()?.height.saturating_sub(8) / 2;
                                help_scroll += page;
                            }
                            KeyCode::Char('g') => {
                                help_scroll = 0;
                            }
                            KeyCode::Char('G') => {
                                help_scroll = u16::MAX;
                            }
                            _ => show_help = false,
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                            KeyCode::Esc => return Ok(()),
                            KeyCode::Char('?') => { show_help = true; help_scroll = 0; }
                            KeyCode::Up | KeyCode::Char('k') => {
                                pending_g = false;
                                activity_cursor = activity_cursor.saturating_sub(1);
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                pending_g = false;
                                let list = renderer::build_activity_list(&agents, &collapsed);
                                let max = (list.len() as u16).saturating_sub(1);
                                if activity_cursor < max {
                                    activity_cursor += 1;
                                }
                            }
                            KeyCode::Char('u') => {
                                pending_g = false;
                                let page = terminal.size()?.height.saturating_sub(8) / 2;
                                activity_cursor = activity_cursor.saturating_sub(page);
                            }
                            KeyCode::Char('d') => {
                                pending_g = false;
                                let list = renderer::build_activity_list(&agents, &collapsed);
                                let max = (list.len() as u16).saturating_sub(1);
                                let page = terminal.size()?.height.saturating_sub(8) / 2;
                                activity_cursor = (activity_cursor + page).min(max);
                            }
                            KeyCode::Char('g') => {
                                if pending_g {
                                    activity_cursor = 0;
                                    pending_g = false;
                                } else {
                                    pending_g = true;
                                }
                            }
                            KeyCode::Char('G') => {
                                pending_g = false;
                                let list = renderer::build_activity_list(&agents, &collapsed);
                                activity_cursor = (list.len() as u16).saturating_sub(1);
                            }
                            KeyCode::Char('h') => {
                                pending_g = false;
                                let list = renderer::build_activity_list(&agents, &collapsed);
                                if let Some(&id) = list.get(activity_cursor as usize) {
                                    // Find the parent to collapse
                                    let parent_id = agents.get(&id)
                                        .and_then(|a| if a.is_subagent { a.parent_id } else { Some(id) });
                                    if let Some(pid) = parent_id {
                                        collapsed.insert(pid);
                                    }
                                }
                            }
                            KeyCode::Char('l') => {
                                pending_g = false;
                                let list = renderer::build_activity_list(&agents, &collapsed);
                                if let Some(&id) = list.get(activity_cursor as usize) {
                                    if !agents.get(&id).map_or(true, |a| a.is_subagent) {
                                        collapsed.remove(&id);
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                pending_g = false;
                                let list = renderer::build_activity_list(&agents, &collapsed);
                                if let Some(&id) = list.get(activity_cursor as usize) {
                                    if let Some(agent) = agents.get(&id) {
                                        if !agent.is_subagent {
                                            if !collapsed.remove(&id) {
                                                collapsed.insert(id);
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                pending_g = false;
                                let events = watcher::scan_for_agents(
                                    &mut known_files, &mut agents, &mut next_id,
                                );
                                scene.handle_events(&events, &agents);
                                for id in agents.keys().copied().collect::<Vec<_>>() {
                                    scene.ensure_character(id, &agents);
                                }
                            }
                            _ => { pending_g = false; }
                        }
                    }
                }
                _ => {}
            }
        }

        // Tick update
        if last_tick.elapsed() >= tick_duration {
            let dt = last_tick.elapsed().as_secs_f64();
            last_tick = Instant::now();
            scene.update(dt);
        }

        // Poll for new JSONL data
        if last_poll.elapsed() >= poll_duration {
            last_poll = Instant::now();

            let scan_events =
                watcher::scan_for_agents(&mut known_files, &mut agents, &mut next_id);
            scene.handle_events(&scan_events, &agents);
            for id in agents.keys().copied().collect::<Vec<_>>() {
                scene.ensure_character(id, &agents);
            }

            let events = watcher::poll_agents(&mut agents);
            scene.handle_events(&events, &agents);

            let stale_ids: Vec<u32> = agents
                .iter()
                .filter(|(_, a)| {
                    // Remove if file was deleted
                    if !a.jsonl_file.exists() {
                        return true;
                    }
                    // Remove if file hasn't been modified for STALE_AGENT_REMOVE_SECS
                    std::fs::metadata(&a.jsonl_file)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|mtime| mtime.elapsed().ok())
                        .is_some_and(|age| age.as_secs() >= STALE_AGENT_REMOVE_SECS)
                })
                .map(|(id, _)| *id)
                .collect();
            for id in stale_ids {
                known_files.remove(&agents[&id].jsonl_file);
                agents.remove(&id);
                scene.characters.remove(&id);
            }
        }
    }
}
