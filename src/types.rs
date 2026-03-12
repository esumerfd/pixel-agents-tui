#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Active,
    Idle,
    WaitingPermission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterState {
    Typing,
    Reading,
    Walking,
    Idle,
    Sitting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct AgentState {
    pub id: u32,
    pub project_dir: PathBuf,
    pub jsonl_file: PathBuf,
    pub file_offset: u64,
    pub line_buffer: String,
    pub active_tool_ids: HashSet<String>,
    pub active_tool_statuses: HashMap<String, String>,
    pub active_tool_names: HashMap<String, String>,
    pub is_waiting: bool,
    pub permission_sent: bool,
    pub had_tools_in_turn: bool,
    pub folder_name: Option<String>,
    pub status: AgentStatus,
    pub parent_id: Option<u32>,
    pub is_subagent: bool,
    pub last_status_text: String,
    pub last_status_time: std::time::Instant,
}

impl AgentState {
    pub fn new(id: u32, project_dir: PathBuf, jsonl_file: PathBuf, folder_name: Option<String>) -> Self {
        Self {
            id,
            project_dir,
            jsonl_file,
            file_offset: 0,
            line_buffer: String::new(),
            active_tool_ids: HashSet::new(),
            active_tool_statuses: HashMap::new(),
            active_tool_names: HashMap::new(),
            is_waiting: false,
            permission_sent: false,
            had_tools_in_turn: false,
            folder_name,
            status: AgentStatus::Idle,
            parent_id: None,
            is_subagent: false,
            last_status_text: String::new(),
            last_status_time: std::time::Instant::now(),
        }
    }

    pub fn new_subagent(id: u32, project_dir: PathBuf, jsonl_file: PathBuf, folder_name: Option<String>, parent_id: u32) -> Self {
        let mut agent = Self::new(id, project_dir, jsonl_file, folder_name);
        agent.parent_id = Some(parent_id);
        agent.is_subagent = true;
        agent
    }

    pub fn clear_activity(&mut self) {
        self.active_tool_ids.clear();
        self.active_tool_statuses.clear();
        self.active_tool_names.clear();
        self.is_waiting = false;
        self.permission_sent = false;
        self.status = AgentStatus::Idle;
        self.last_status_text.clear();
    }

    pub fn current_status_text(&self) -> String {
        // Priority 1: actively running tool — show its specific status
        if let Some(status) = self.active_tool_statuses.values().next() {
            return status.clone();
        }

        match self.status {
            AgentStatus::Active => {
                // Between tool calls: show last tool status if we have one,
                // otherwise "Thinking..." (LLM is generating the next action)
                if !self.last_status_text.is_empty() {
                    self.last_status_text.clone()
                } else {
                    "Thinking...".to_string()
                }
            }
            AgentStatus::Idle => "Idle".to_string(),
            AgentStatus::WaitingPermission => "Waiting for permission".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Character {
    pub id: u32,
    pub state: CharacterState,
    pub dir: Direction,
    pub col: u16,
    pub row: u16,
    pub path: Vec<(u16, u16)>,
    pub move_progress: f64,
    pub frame: u8,
    pub frame_timer: f64,
    pub wander_timer: f64,
    pub wander_count: u32,
    pub wander_limit: u32,
    pub is_active: bool,
    pub seat_col: u16,
    pub seat_row: u16,
    pub desk_col: u16,
    pub desk_row: u16,
    pub seat_timer: f64,
    pub palette: u8,
    pub person: u8,
    pub current_tool: Option<String>,
    pub status_text: String,
    pub is_subagent: bool,
    pub lounge_seat: Option<usize>,
}

impl Character {
    pub fn new(id: u32, seat_col: u16, seat_row: u16, palette: u8, person: u8) -> Self {
        Self {
            id,
            state: CharacterState::Idle,
            dir: Direction::Down,
            col: seat_col,
            row: seat_row,
            path: Vec::new(),
            move_progress: 0.0,
            frame: 0,
            frame_timer: 0.0,
            wander_timer: 3.0,
            wander_count: 0,
            wander_limit: 4,
            is_active: false,
            seat_col,
            seat_row,
            desk_col: seat_col,
            desk_row: seat_row.saturating_sub(3),
            seat_timer: 0.0,
            palette,
            person,
            current_tool: None,
            status_text: "Idle".to_string(),
            is_subagent: false,
            lounge_seat: None,
        }
    }
}

/// Events emitted by the transcript parser
#[derive(Debug, Clone)]
pub enum AgentEvent {
    ToolStart { agent_id: u32, tool_id: String, tool_name: String, status: String },
    ToolDone { agent_id: u32, tool_id: String },
    StatusChange { agent_id: u32, status: AgentStatus },
    ClearActivity { agent_id: u32 },
}
