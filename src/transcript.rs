use std::path::Path;

use crate::constants::*;
use crate::types::*;

/// Format a tool invocation into a human-readable status string.
pub fn format_tool_status(tool_name: &str, input: &serde_json::Value) -> String {
    let basename = |key: &str| -> String {
        input
            .get(key)
            .and_then(|v| v.as_str())
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.to_string())
            })
            .unwrap_or_default()
    };

    match tool_name {
        "Read" => format!("Reading {}", basename("file_path")),
        "Edit" => format!("Editing {}", basename("file_path")),
        "Write" => format!("Writing {}", basename("file_path")),
        "Bash" => {
            let cmd = input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if cmd.len() > BASH_COMMAND_DISPLAY_MAX_LENGTH {
                format!("Running: {}\u{2026}", &cmd[..BASH_COMMAND_DISPLAY_MAX_LENGTH])
            } else {
                format!("Running: {}", cmd)
            }
        }
        "Glob" => "Searching files".to_string(),
        "Grep" => "Searching code".to_string(),
        "WebFetch" => "Fetching web content".to_string(),
        "WebSearch" => "Searching the web".to_string(),
        "Task" | "Agent" => {
            let desc = input
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if desc.is_empty() {
                "Running subtask".to_string()
            } else if desc.len() > TASK_DESCRIPTION_DISPLAY_MAX_LENGTH {
                format!(
                    "Subtask: {}\u{2026}",
                    &desc[..TASK_DESCRIPTION_DISPLAY_MAX_LENGTH]
                )
            } else {
                format!("Subtask: {}", desc)
            }
        }
        "AskUserQuestion" => "Waiting for your answer".to_string(),
        "EnterPlanMode" => "Planning".to_string(),
        "NotebookEdit" => "Editing notebook".to_string(),
        _ => format!("Using {}", tool_name),
    }
}

/// Returns true if this tool triggers a reading animation rather than typing.
pub fn is_reading_tool(tool_name: &str) -> bool {
    matches!(tool_name, "Read" | "Grep" | "Glob" | "WebFetch" | "WebSearch")
}

/// Process a single JSONL transcript line and update the agent state.
/// Returns a list of events to be handled by the UI.
pub fn process_transcript_line(line: &str, agent: &mut AgentState) -> Vec<AgentEvent> {
    let mut events = Vec::new();

    let record: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return events,
    };

    let record_type = record.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match record_type {
        "assistant" => {
            let blocks = record
                .pointer("/message/content")
                .and_then(|v| v.as_array());

            if let Some(blocks) = blocks {
                let has_tool_use = blocks.iter().any(|b| {
                    b.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                });

                if has_tool_use {
                    agent.is_waiting = false;
                    agent.had_tools_in_turn = true;
                    agent.status = AgentStatus::Active;
                    events.push(AgentEvent::StatusChange {
                        agent_id: agent.id,
                        status: AgentStatus::Active,
                    });

                    for block in blocks {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                            let tool_id = block
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let tool_name = block
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let input = block.get("input").cloned().unwrap_or(serde_json::Value::Object(Default::default()));
                            let status = format_tool_status(&tool_name, &input);

                            agent.active_tool_ids.insert(tool_id.clone());
                            agent.active_tool_statuses.insert(tool_id.clone(), status.clone());
                            agent.active_tool_names.insert(tool_id.clone(), tool_name.clone());
                            agent.last_status_text = status.clone();
                            agent.last_status_time = std::time::Instant::now();

                            events.push(AgentEvent::ToolStart {
                                agent_id: agent.id,
                                tool_id,
                                tool_name,
                                status,
                            });
                        }
                    }
                } else {
                    let has_text = blocks.iter().any(|b| {
                        b.get("type").and_then(|t| t.as_str()) == Some("text")
                    });
                    if has_text && !agent.had_tools_in_turn {
                        // Text-only response — will go idle after TEXT_IDLE_DELAY_MS
                        // (handled by the watcher timer logic)
                    }
                }
            }
        }
        "user" => {
            let content = record.pointer("/message/content");
            if let Some(content) = content {
                if let Some(blocks) = content.as_array() {
                    let has_tool_result = blocks.iter().any(|b| {
                        b.get("type").and_then(|t| t.as_str()) == Some("tool_result")
                    });

                    if has_tool_result {
                        for block in blocks {
                            if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                                if let Some(tool_use_id) = block.get("tool_use_id").and_then(|v| v.as_str()) {
                                    let tool_id = tool_use_id.to_string();
                                    agent.active_tool_ids.remove(&tool_id);
                                    agent.active_tool_statuses.remove(&tool_id);
                                    agent.active_tool_names.remove(&tool_id);

                                    events.push(AgentEvent::ToolDone {
                                        agent_id: agent.id,
                                        tool_id,
                                    });
                                }
                            }
                        }
                        if agent.active_tool_ids.is_empty() {
                            agent.had_tools_in_turn = false;
                        }
                    } else {
                        // New user text prompt — new turn
                        agent.clear_activity();
                        agent.had_tools_in_turn = false;
                        events.push(AgentEvent::ClearActivity {
                            agent_id: agent.id,
                        });
                    }
                } else if content.is_string() {
                    // New user text prompt — new turn
                    agent.clear_activity();
                    agent.had_tools_in_turn = false;
                    events.push(AgentEvent::ClearActivity {
                        agent_id: agent.id,
                    });
                }
            }
        }
        "system" => {
            let subtype = record.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
            if subtype == "turn_duration" {
                agent.active_tool_ids.clear();
                agent.active_tool_statuses.clear();
                agent.active_tool_names.clear();
                agent.is_waiting = true;
                agent.permission_sent = false;
                agent.had_tools_in_turn = false;
                agent.status = AgentStatus::Idle;

                events.push(AgentEvent::StatusChange {
                    agent_id: agent.id,
                    status: AgentStatus::Idle,
                });
            }
        }
        _ => {}
    }

    events
}
