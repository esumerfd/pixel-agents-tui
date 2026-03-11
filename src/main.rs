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

    // Initial scan for agents (to get count for desk layout)
    let initial_events = watcher::scan_for_agents(&mut known_files, &mut agents, &mut next_id);

    // Build office with desk count matching top-level agents
    let top_level_count = agents.values().filter(|a| !a.is_subagent).count();
    let office = layout::build_office(top_level_count);
    let mut scene = scene::Scene::new(office);

    scene.handle_events(&initial_events, &agents);

    for id in agents.keys().copied().collect::<Vec<_>>() {
        scene.ensure_character(id, &agents);
    }

    // Read existing content from all agent files to get current state
    let agent_ids: Vec<u32> = agents.keys().copied().collect();
    for id in agent_ids {
        if let Some(agent) = agents.get_mut(&id) {
            let events = watcher::read_new_lines(agent);
            scene.handle_events(&events, &agents);
        }
    }

    let tick_duration = Duration::from_millis(TICK_RATE_MS);
    let poll_duration = Duration::from_millis(FILE_POLL_INTERVAL_MS);
    let mut last_tick = Instant::now();
    let mut last_poll = Instant::now();

    loop {
        terminal.draw(|frame| {
            renderer::render(frame, &scene, &agents, show_help);
        })?;

        let timeout = tick_duration.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if show_help {
                        // Any key closes help
                        match key.code {
                            KeyCode::Char('?') | KeyCode::Esc => show_help = false,
                            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                            _ => show_help = false,
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                            KeyCode::Esc => return Ok(()),
                            KeyCode::Char('?') => show_help = true,
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                let events = watcher::scan_for_agents(
                                    &mut known_files, &mut agents, &mut next_id,
                                );
                                scene.handle_events(&events, &agents);
                                for id in agents.keys().copied().collect::<Vec<_>>() {
                                    scene.ensure_character(id, &agents);
                                }
                            }
                            _ => {}
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
                .filter(|(_, a)| !a.jsonl_file.exists())
                .map(|(id, _)| *id)
                .collect();
            for id in stale_ids {
                agents.remove(&id);
                scene.characters.remove(&id);
            }
        }
    }
}
