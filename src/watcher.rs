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
            return parts[i..].join("-");
        }
    }

    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| sanitized.to_string())
}

/// Sanitize a filesystem path the same way Claude Code does for project directories.
/// Non-alphanumeric characters become hyphens.
fn sanitize_path(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}

// ── Team Discovery ─────────────────────────────────────────────

/// Parsed team config from ~/.claude/teams/<name>/config.json
#[derive(Debug)]
#[allow(dead_code)]
struct TeamConfig {
    name: String,
    description: String,
    lead_session_id: String,
    created_at: u64,
    members: Vec<TeamMember>,
}

#[derive(Debug)]
#[allow(dead_code)]
struct TeamMember {
    name: String,
    color: Option<String>,
    cwd: String,
    is_lead: bool,
    backend_type: Option<String>,
    prompt: Option<String>,
}

/// Read and parse all team configs from ~/.claude/teams/
fn discover_teams() -> Vec<TeamConfig> {
    let teams_dir = match dirs::home_dir() {
        Some(home) => home.join(".claude").join("teams"),
        None => return Vec::new(),
    };

    if !teams_dir.exists() {
        return Vec::new();
    }

    let mut teams = Vec::new();
    let entries = match fs::read_dir(&teams_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let config_path = entry.path().join("config.json");
        if !config_path.exists() {
            continue;
        }
        if let Some(team) = parse_team_config(&config_path) {
            teams.push(team);
        }
    }

    teams
}

fn parse_team_config(path: &Path) -> Option<TeamConfig> {
    let data = fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;

    let name = json.get("name")?.as_str()?.to_string();
    let description = json.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let lead_session_id = json.get("leadSessionId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let created_at = json.get("createdAt").and_then(|v| v.as_u64()).unwrap_or(0);

    let members_arr = json.get("members")?.as_array()?;
    let mut members = Vec::new();
    let lead_agent_id = json.get("leadAgentId").and_then(|v| v.as_str()).unwrap_or("");

    for m in members_arr {
        let agent_id = m.get("agentId").and_then(|v| v.as_str()).unwrap_or("");
        let member_name = m.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let color = m.get("color").and_then(|v| v.as_str()).map(|s| s.to_string());
        let cwd = m.get("cwd").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let backend_type = m.get("backendType").and_then(|v| v.as_str()).map(|s| s.to_string());
        let prompt = m.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string());

        members.push(TeamMember {
            name: member_name,
            color,
            cwd,
            is_lead: agent_id == lead_agent_id,
            backend_type,
            prompt,
        });
    }

    Some(TeamConfig { name, description, lead_session_id, created_at, members })
}

/// Check if a team is recent (created within the last 2 hours).
fn is_recent_team(team: &TeamConfig) -> bool {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    now_ms.saturating_sub(team.created_at) < 7_200_000
}

/// Find the project directory for a given cwd by matching sanitized path.
fn find_project_dir_for_cwd(cwd: &str) -> Option<PathBuf> {
    let projects_dir = dirs::home_dir()?.join(".claude").join("projects");
    if !projects_dir.exists() {
        return None;
    }
    let sanitized = sanitize_path(cwd);
    let candidate = projects_dir.join(&sanitized);
    if candidate.is_dir() {
        return Some(candidate);
    }
    None
}

/// For a team session, find the most recent subagent JSONL per member name using .meta.json.
/// Returns a map of member_name -> jsonl_path for the most recent file per member.
fn find_best_subagent_per_member(
    project_dir: &Path,
    session_id: &str,
    member_names: &[&str],
) -> HashMap<String, PathBuf> {
    let subagents_dir = project_dir.join(session_id).join("subagents");
    if !subagents_dir.is_dir() {
        return HashMap::new();
    }

    // Collect all meta.json files and their agent types
    let mut by_member: HashMap<String, Vec<(PathBuf, std::time::SystemTime)>> = HashMap::new();

    if let Ok(entries) = fs::read_dir(&subagents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
            if !name.ends_with(".meta") {
                continue;
            }

            if let Some(agent_type) = read_subagent_meta_from_path(&path) {
                if member_names.contains(&agent_type.as_str()) {
                    let jsonl_path = path.with_file_name(
                        name.strip_suffix(".meta").unwrap_or(name).to_string() + ".jsonl"
                    );
                    if jsonl_path.exists() {
                        let mtime = fs::metadata(&jsonl_path)
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        by_member.entry(agent_type).or_default().push((jsonl_path, mtime));
                    }
                }
            }
        }
    }

    // Pick the most recent JSONL per member
    let mut result = HashMap::new();
    for (member, mut files) in by_member {
        files.sort_by(|a, b| b.1.cmp(&a.1));
        if let Some((path, _)) = files.into_iter().next() {
            result.insert(member, path);
        }
    }
    result
}

fn read_subagent_meta_from_path(meta_path: &Path) -> Option<String> {
    let data = fs::read_to_string(meta_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;
    json.get("agentType").and_then(|v| v.as_str()).map(|s| s.to_string())
}

// ── Project-Level Discovery (fallback for non-team agents) ─────

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

/// Find subagent JSONL files under a project directory (any session).
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

// ── Main Scanner ───────────────────────────────────────────────

/// Scan for agents using a two-phase approach:
/// 1. Discover teams from ~/.claude/teams/ and match members to JSONL files
/// 2. Fall back to project-dir scan for non-team agents
pub fn scan_for_agents(
    known_files: &mut HashSet<PathBuf>,
    agents: &mut HashMap<u32, AgentState>,
    next_id: &mut u32,
) -> Vec<AgentEvent> {
    let mut events = Vec::new();

    // Track which JSONL files are claimed by teams so the fallback scan skips them
    let mut team_claimed_files: HashSet<PathBuf> = HashSet::new();

    // Phase 1: Team discovery
    // When multiple teams share the same lead session, keep only the most recent one
    let mut teams = discover_teams();
    teams.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let mut seen_lead_sessions: HashSet<String> = HashSet::new();
    teams.retain(|t| {
        if t.lead_session_id.is_empty() {
            return false;
        }
        seen_lead_sessions.insert(t.lead_session_id.clone())
    });

    for team in &teams {
        if team.lead_session_id.is_empty() {
            continue;
        }

        // Find the lead's cwd to locate the project directory
        let lead_cwd = team.members.iter()
            .find(|m| m.is_lead)
            .map(|m| m.cwd.as_str())
            .unwrap_or("");

        let project_dir = match find_project_dir_for_cwd(lead_cwd) {
            Some(d) => d,
            None => continue,
        };

        // Find the lead's JSONL file
        let lead_jsonl = project_dir.join(format!("{}.jsonl", team.lead_session_id));
        if !lead_jsonl.exists() {
            continue;
        }
        // Team is active if its lead JSONL is recent
        if !is_recent_file(&lead_jsonl) && !is_recent_team(team) {
            continue;
        }

        // Claim the lead's JSONL file (not the entire directory — other
        // standalone sessions in the same project dir should still be discovered)
        team_claimed_files.insert(lead_jsonl.clone());

        // Create/find lead agent
        let lead_id;
        if known_files.contains(&lead_jsonl) {
            lead_id = agents.iter()
                .find(|(_, a)| a.jsonl_file == lead_jsonl && !a.is_subagent)
                .map(|(id, _)| *id);
        } else if lead_jsonl.exists() {
            known_files.insert(lead_jsonl.clone());
            team_claimed_files.insert(lead_jsonl.clone());

            let id = *next_id;
            *next_id += 1;

            let folder_name = project_dir
                .file_name()
                .map(|n| decode_project_name(&n.to_string_lossy()));

            let lead_member = team.members.iter().find(|m| m.is_lead);
            let agent = AgentState::new_team_member(
                id,
                project_dir.clone(),
                lead_jsonl,
                folder_name,
                team.name.clone(),
                String::new(), // lead uses folder_name for display
                lead_member.and_then(|m| m.color.clone()),
                true,
                None,
            );
            agents.insert(id, agent);
            events.push(AgentEvent::StatusChange {
                agent_id: id,
                status: AgentStatus::Idle,
            });
            lead_id = Some(id);
        } else {
            lead_id = None;
        }

        let Some(lid) = lead_id else { continue };

        // Build member name list for meta.json matching
        let non_lead_members: Vec<&TeamMember> = team.members.iter()
            .filter(|m| !m.is_lead)
            .collect();
        let member_names: Vec<&str> = non_lead_members.iter().map(|m| m.name.as_str()).collect();

        // Find the best (most recent) subagent JSONL per member via .meta.json
        let best_per_member = find_best_subagent_per_member(
            &project_dir, &team.lead_session_id, &member_names,
        );

        let parent_folder = agents.get(&lid).and_then(|a| a.folder_name.clone());

        for member in &non_lead_members {
            // Check if we already have an agent for this member
            let existing_id = agents.iter()
                .find(|(_, a)| {
                    a.team_name.as_deref() == Some(&team.name)
                        && a.member_name.as_deref() == Some(&member.name)
                })
                .map(|(id, _)| *id);

            if let Some(eid) = existing_id {
                // Mark their file as team-claimed
                if let Some(best_path) = best_per_member.get(&member.name) {
                    team_claimed_files.insert(best_path.clone());

                    // If a newer JSONL file exists for this member, switch to it
                    if let Some(agent) = agents.get_mut(&eid) {
                        if agent.jsonl_file != *best_path {
                            known_files.insert(best_path.clone());
                            agent.jsonl_file = best_path.clone();
                            agent.file_offset = 0;
                            agent.line_buffer.clear();
                            agent.clear_activity();
                            events.push(AgentEvent::ClearActivity { agent_id: eid });
                        }
                    }
                }
                continue;
            }

            let sub_id = *next_id;
            *next_id += 1;

            let jsonl_file = best_per_member.get(&member.name).cloned();

            if let Some(ref jf) = jsonl_file {
                known_files.insert(jf.clone());
                team_claimed_files.insert(jf.clone());
            }

            // Use the best JSONL if available, otherwise use a placeholder path
            // (the agent will show in the activity panel but won't have live transcript data)
            let file_path = jsonl_file.unwrap_or_else(|| {
                project_dir.join(format!("__team_placeholder__{}.jsonl", member.name))
            });

            let agent = AgentState::new_team_member(
                sub_id,
                project_dir.clone(),
                file_path,
                parent_folder.clone(),
                team.name.clone(),
                member.name.clone(),
                member.color.clone(),
                false,
                Some(lid),
            );
            agents.insert(sub_id, agent);

            events.push(AgentEvent::StatusChange {
                agent_id: sub_id,
                status: AgentStatus::Idle,
            });
        }
    }

    // Phase 2: Fallback project-dir scan for non-team agents
    let project_dirs = discover_project_dirs();

    for project_dir in &project_dirs {
        let jsonl_files = find_jsonl_files(project_dir);

        // Take the most recent JSONL file that isn't claimed by a team
        if let Some(file) = jsonl_files.into_iter().find(|f| !team_claimed_files.contains(f)) {
            let parent_id;

            if known_files.contains(&file) {
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
                    if known_files.contains(&sub_file) || team_claimed_files.contains(&sub_file) {
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
/// Also detects stale agents that are still marked Active but whose JSONL
/// file hasn't been modified recently — these are transitioned to Idle.
pub fn poll_agents(agents: &mut HashMap<u32, AgentState>) -> Vec<AgentEvent> {
    let ids: Vec<u32> = agents.keys().copied().collect();
    let mut all_events = Vec::new();

    for id in ids {
        if let Some(agent) = agents.get_mut(&id) {
            let events = read_new_lines(agent);
            let had_new_data = !events.is_empty();
            all_events.extend(events);

            // If the agent is Active but hasn't received new JSONL data,
            // check file modification time to detect stopped sessions
            if !had_new_data && agent.status == AgentStatus::Active {
                let is_stale = fs::metadata(&agent.jsonl_file)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|mtime| mtime.elapsed().ok())
                    .is_some_and(|age| age.as_secs() >= crate::constants::STALE_ACTIVE_TIMEOUT_SECS);

                if is_stale {
                    agent.active_tool_ids.clear();
                    agent.active_tool_statuses.clear();
                    agent.active_tool_names.clear();
                    agent.had_tools_in_turn = false;
                    agent.status = AgentStatus::Idle;

                    all_events.push(AgentEvent::StatusChange {
                        agent_id: agent.id,
                        status: AgentStatus::Idle,
                    });
                }
            }
        }
    }

    all_events
}
