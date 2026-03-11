use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::transcript;
use crate::types::*;

/// Decode a sanitized project directory name back to the real project name.
/// Claude Code sanitizes paths like /Users/foo/my-project to -Users-foo-my-project.
/// We walk the filesystem to reconstruct which hyphens are path separators vs part of names.
fn decode_project_name(sanitized: &str) -> String {
    let name = sanitized.strip_prefix('-').unwrap_or(sanitized);
    let parts: Vec<&str> = name.split('-').collect();

    let mut path = PathBuf::from("/");
    let mut i = 0;

    while i < parts.len() {
        let mut found = false;
        // Try longest segment first (up to 6 parts) to handle hyphenated dir names
        let max_end = (i + 6).min(parts.len());
        for end in (i + 1..=max_end).rev() {
            let segment = parts[i..end].join("-");
            let candidate = path.join(&segment);
            if candidate.exists() {
                path = candidate;
                i = end;
                found = true;
                break;
            }
        }
        if !found {
            // Can't resolve further — join remaining parts as the name
            return parts[i..].join("-");
        }
    }

    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| sanitized.to_string())
}

/// Discovers all project directories under ~/.claude/projects/
pub fn discover_project_dirs() -> Vec<PathBuf> {
    let claude_dir = match dirs::home_dir() {
        Some(home) => home.join(".claude").join("projects"),
        None => return Vec::new(),
    };

    if !claude_dir.exists() {
        return Vec::new();
    }

    let mut project_dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(&claude_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                project_dirs.push(path);
            }
        }
    }
    project_dirs
}

/// Find all .jsonl files in a project directory, sorted by modification time (newest first).
pub fn find_jsonl_files(project_dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                let mtime = fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                files.push((path, mtime));
            }
        }
    }

    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.into_iter().map(|(p, _)| p).collect()
}

/// Find subagent JSONL files under a project directory.
/// They live in `<project-dir>/<session-id>/subagents/*.jsonl`.
fn find_subagent_files(project_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let subagents_dir = path.join("subagents");
                if subagents_dir.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&subagents_dir) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                                files.push(sub_path);
                            }
                        }
                    }
                }
            }
        }
    }
    files
}

fn is_recent_file(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|mtime| mtime.elapsed().ok())
        .is_some_and(|age| age.as_secs() < 7200)
}

/// Scan all project dirs and discover active agents from recent JSONL files.
/// Returns new agents that weren't previously known.
pub fn scan_for_agents(
    known_files: &mut HashSet<PathBuf>,
    agents: &mut HashMap<u32, AgentState>,
    next_id: &mut u32,
) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    let project_dirs = discover_project_dirs();

    for project_dir in &project_dirs {
        let jsonl_files = find_jsonl_files(project_dir);

        // Only take the most recent JSONL file per project directory.
        if let Some(file) = jsonl_files.into_iter().next() {
            let parent_id;

            if known_files.contains(&file) {
                // Find existing parent agent id for subagent discovery
                parent_id = agents.iter()
                    .find(|(_, a)| a.jsonl_file == file && !a.is_subagent)
                    .map(|(id, _)| *id);
            } else {
                known_files.insert(file.clone());

                if !is_recent_file(&file) {
                    continue;
                }

                let folder_name = project_dir
                    .file_name()
                    .map(|n| decode_project_name(&n.to_string_lossy()));

                let id = *next_id;
                *next_id += 1;

                let agent = AgentState::new(id, project_dir.clone(), file, folder_name);
                agents.insert(id, agent);

                events.push(AgentEvent::StatusChange {
                    agent_id: id,
                    status: AgentStatus::Idle,
                });

                parent_id = Some(id);
            }

            // Discover subagents for this project
            if let Some(pid) = parent_id {
                let subagent_files = find_subagent_files(project_dir);
                let parent_folder = agents.get(&pid).and_then(|a| a.folder_name.clone());

                for sub_file in subagent_files {
                    if known_files.contains(&sub_file) {
                        continue;
                    }
                    if !is_recent_file(&sub_file) {
                        continue;
                    }
                    known_files.insert(sub_file.clone());

                    let sub_id = *next_id;
                    *next_id += 1;

                    let agent = AgentState::new_subagent(
                        sub_id,
                        project_dir.clone(),
                        sub_file,
                        parent_folder.clone(),
                        pid,
                    );
                    agents.insert(sub_id, agent);

                    events.push(AgentEvent::StatusChange {
                        agent_id: sub_id,
                        status: AgentStatus::Idle,
                    });
                }
            }
        }
    }

    events
}

/// Read new lines from an agent's JSONL file and process them.
pub fn read_new_lines(agent: &mut AgentState) -> Vec<AgentEvent> {
    let mut events = Vec::new();

    let metadata = match fs::metadata(&agent.jsonl_file) {
        Ok(m) => m,
        Err(_) => return events,
    };

    let file_size = metadata.len();
    if file_size <= agent.file_offset {
        return events;
    }

    let mut file = match fs::File::open(&agent.jsonl_file) {
        Ok(f) => f,
        Err(_) => return events,
    };

    use std::io::Seek;
    if file.seek(std::io::SeekFrom::Start(agent.file_offset)).is_err() {
        return events;
    }

    let bytes_to_read = (file_size - agent.file_offset) as usize;
    let mut buf = vec![0u8; bytes_to_read];
    match file.read_exact(&mut buf) {
        Ok(()) => {}
        Err(_) => return events,
    }

    agent.file_offset = file_size;

    let text = format!("{}{}", agent.line_buffer, String::from_utf8_lossy(&buf));
    let mut lines: Vec<&str> = text.split('\n').collect();
    agent.line_buffer = lines.pop().unwrap_or("").to_string();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let line_events = transcript::process_transcript_line(trimmed, agent);
        events.extend(line_events);
    }

    events
}

/// Poll all agents for new data.
pub fn poll_agents(agents: &mut HashMap<u32, AgentState>) -> Vec<AgentEvent> {
    let ids: Vec<u32> = agents.keys().copied().collect();
    let mut all_events = Vec::new();

    for id in ids {
        if let Some(agent) = agents.get_mut(&id) {
            let events = read_new_lines(agent);
            all_events.extend(events);
        }
    }

    all_events
}
