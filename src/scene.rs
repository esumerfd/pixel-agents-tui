use rand::Rng;
use std::collections::HashMap;

use crate::constants::*;
use crate::layout::OfficeLayout;
use crate::pathfinding;
use crate::transcript;
use crate::types::*;

pub struct Scene {
    pub characters: HashMap<u32, Character>,
    pub layout: OfficeLayout,
    walkable_tiles: Vec<(u16, u16)>,
    lounge_seat_occupied: Vec<bool>,
    next_palette: u8,
    next_seat: usize,
}

impl Scene {
    pub fn new(layout: OfficeLayout) -> Self {
        let walkable_tiles = pathfinding::get_walkable_tiles(&layout);
        let lounge_seat_count = layout.lounge_seats.len();
        Self {
            characters: HashMap::new(),
            layout,
            walkable_tiles,
            lounge_seat_occupied: vec![false; lounge_seat_count],
            next_palette: 0,
            next_seat: 0,
        }
    }

    /// Assign the next available seat to a new agent.
    fn assign_seat(&mut self) -> Option<(u16, u16, u16, u16)> {
        if self.layout.seats.is_empty() {
            return None;
        }
        let idx = self.next_seat % self.layout.seats.len();
        self.next_seat += 1;
        let seat = &self.layout.seats[idx];
        Some((seat.col, seat.row, seat.desk_col, seat.desk_row))
    }

    /// Free a lounge seat if the character is occupying one.
    fn free_lounge_seat(&mut self, ch: &mut Character) {
        if let Some(idx) = ch.lounge_seat.take() {
            if idx < self.lounge_seat_occupied.len() {
                self.lounge_seat_occupied[idx] = false;
            }
        }
    }

    /// Pick a random unoccupied lounge seat.
    fn pick_lounge_seat(&self) -> Option<(usize, u16, u16)> {
        let available: Vec<usize> = self.lounge_seat_occupied.iter()
            .enumerate()
            .filter(|(_, occupied)| !*occupied)
            .map(|(i, _)| i)
            .collect();
        if available.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        let idx = available[rng.gen_range(0..available.len())];
        let seat = &self.layout.lounge_seats[idx];
        Some((idx, seat.col, seat.row))
    }

    pub fn ensure_character(&mut self, agent_id: u32, agents: &std::collections::HashMap<u32, crate::types::AgentState>) {
        if self.characters.contains_key(&agent_id) {
            return;
        }

        let agent = agents.get(&agent_id);
        let is_subagent = agent.map_or(false, |a| a.is_subagent);
        let parent_id = agent.and_then(|a| a.parent_id);

        if is_subagent {
            let (palette, seat_col, seat_row, desk_col, desk_row) = if let Some(pid) = parent_id {
                if let Some(parent_ch) = self.characters.get(&pid) {
                    let mut rng = rand::thread_rng();
                    let offset_col: i16 = rng.gen_range(-3..=3);
                    let offset_row: i16 = rng.gen_range(-2..=2);
                    let sc = (parent_ch.seat_col as i16 + offset_col).max(1) as u16;
                    let sr = (parent_ch.seat_row as i16 + offset_row).max(1) as u16;
                    (parent_ch.palette, sc, sr, parent_ch.desk_col, parent_ch.desk_row)
                } else {
                    let (sc, sr, dc, dr) = self.assign_seat().unwrap_or((5, 5, 5, 2));
                    (self.next_palette, sc, sr, dc, dr)
                }
            } else {
                let (sc, sr, dc, dr) = self.assign_seat().unwrap_or((5, 5, 5, 2));
                (self.next_palette, sc, sr, dc, dr)
            };

            let mut ch = Character::new(agent_id, seat_col, seat_row, palette);
            ch.desk_col = desk_col;
            ch.desk_row = desk_row;
            ch.is_subagent = true;
            self.characters.insert(agent_id, ch);
        } else {
            let palette = self.next_palette;
            self.next_palette = (self.next_palette + 1) % 6;

            let (seat_col, seat_row, desk_col, desk_row) = self.assign_seat()
                .unwrap_or((5, 5, 5, 2));

            let mut ch = Character::new(agent_id, seat_col, seat_row, palette);
            ch.desk_col = desk_col;
            ch.desk_row = desk_row;
            self.characters.insert(agent_id, ch);
        }
    }

    pub fn handle_events(&mut self, events: &[AgentEvent], agents: &std::collections::HashMap<u32, crate::types::AgentState>) {
        for event in events {
            match event {
                AgentEvent::ToolStart {
                    agent_id,
                    tool_name,
                    status,
                    ..
                } => {
                    self.ensure_character(*agent_id, agents);
                    if let Some(ch) = self.characters.get_mut(agent_id) {
                        ch.is_active = true;
                        ch.current_tool = Some(tool_name.clone());
                        ch.status_text = status.clone();

                        // Free lounge seat if sitting
                        if ch.state == CharacterState::Sitting {
                            if let Some(idx) = ch.lounge_seat.take() {
                                if idx < self.lounge_seat_occupied.len() {
                                    self.lounge_seat_occupied[idx] = false;
                                }
                            }
                        }

                        if ch.col != ch.seat_col || ch.row != ch.seat_row {
                            let path = pathfinding::find_path(
                                ch.col, ch.row, ch.seat_col, ch.seat_row, &self.layout,
                            );
                            if !path.is_empty() {
                                ch.path = path;
                                ch.state = CharacterState::Walking;
                                ch.move_progress = 0.0;
                            }
                        } else if transcript::is_reading_tool(tool_name) {
                            ch.state = CharacterState::Reading;
                        } else {
                            ch.state = CharacterState::Typing;
                        }
                    }
                }
                AgentEvent::ToolDone { .. } => {}
                AgentEvent::StatusChange { agent_id, status } => {
                    self.ensure_character(*agent_id, agents);
                    if let Some(ch) = self.characters.get_mut(agent_id) {
                        match status {
                            AgentStatus::Active => {
                                ch.is_active = true;

                                // Free lounge seat if sitting
                                if ch.state == CharacterState::Sitting {
                                    if let Some(idx) = ch.lounge_seat.take() {
                                        if idx < self.lounge_seat_occupied.len() {
                                            self.lounge_seat_occupied[idx] = false;
                                        }
                                    }
                                }

                                if ch.col != ch.seat_col || ch.row != ch.seat_row {
                                    let path = pathfinding::find_path(
                                        ch.col, ch.row, ch.seat_col, ch.seat_row, &self.layout,
                                    );
                                    if !path.is_empty() {
                                        ch.path = path;
                                        ch.state = CharacterState::Walking;
                                        ch.move_progress = 0.0;
                                    } else {
                                        ch.state = CharacterState::Typing;
                                    }
                                } else {
                                    ch.state = CharacterState::Typing;
                                }
                            }
                            AgentStatus::Idle => {
                                ch.is_active = false;
                                ch.current_tool = None;
                                ch.status_text = "Idle".to_string();
                            }
                            AgentStatus::WaitingPermission => {
                                ch.status_text = "Waiting for permission".to_string();
                            }
                        }
                    }
                }
                AgentEvent::ClearActivity { agent_id } => {
                    if let Some(ch) = self.characters.get_mut(agent_id) {
                        ch.is_active = false;
                        ch.current_tool = None;
                        ch.status_text = "Idle".to_string();
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f64) {
        let ids: Vec<u32> = self.characters.keys().copied().collect();
        for id in ids {
            if let Some(mut ch) = self.characters.remove(&id) {
                self.update_character(&mut ch, dt);
                self.characters.insert(id, ch);
            }
        }
    }

    fn update_character(&mut self, ch: &mut Character, dt: f64) {
        ch.frame_timer += dt;

        match ch.state {
            CharacterState::Typing | CharacterState::Reading => {
                if ch.frame_timer >= TYPE_FRAME_DURATION_SEC {
                    ch.frame_timer -= TYPE_FRAME_DURATION_SEC;
                    ch.frame = (ch.frame + 1) % 2;
                }
                if !ch.is_active {
                    ch.seat_timer -= dt;
                    if ch.seat_timer <= 0.0 {
                        ch.state = CharacterState::Idle;
                        ch.frame = 0;
                        ch.frame_timer = 0.0;
                        ch.wander_timer = random_range(WANDER_PAUSE_MIN_SEC, WANDER_PAUSE_MAX_SEC);
                        ch.wander_count = 0;
                        ch.wander_limit = random_int(WANDER_MOVES_BEFORE_REST_MIN, WANDER_MOVES_BEFORE_REST_MAX);
                    }
                }
            }
            CharacterState::Sitting => {
                ch.frame = 0;
                if ch.is_active {
                    // Got work — head back to desk
                    self.free_lounge_seat(ch);
                    let path = pathfinding::find_path(
                        ch.col, ch.row, ch.seat_col, ch.seat_row, &self.layout,
                    );
                    if !path.is_empty() {
                        ch.path = path;
                        ch.state = CharacterState::Walking;
                        ch.move_progress = 0.0;
                    } else {
                        ch.state = CharacterState::Typing;
                        ch.dir = Direction::Up;
                    }
                    return;
                }
                ch.seat_timer -= dt;
                if ch.seat_timer <= 0.0 {
                    // Done sitting — go idle and wander
                    self.free_lounge_seat(ch);
                    ch.state = CharacterState::Idle;
                    ch.frame = 0;
                    ch.frame_timer = 0.0;
                    ch.wander_timer = random_range(WANDER_PAUSE_MIN_SEC, WANDER_PAUSE_MAX_SEC);
                    ch.wander_count = 0;
                    ch.wander_limit = random_int(WANDER_MOVES_BEFORE_REST_MIN, WANDER_MOVES_BEFORE_REST_MAX);
                }
            }
            CharacterState::Idle => {
                ch.frame = 0;
                if ch.is_active {
                    self.free_lounge_seat(ch);
                    let path = pathfinding::find_path(
                        ch.col, ch.row, ch.seat_col, ch.seat_row, &self.layout,
                    );
                    if !path.is_empty() {
                        ch.path = path;
                        ch.state = CharacterState::Walking;
                        ch.move_progress = 0.0;
                    } else {
                        ch.state = CharacterState::Typing;
                        ch.dir = Direction::Up;
                    }
                    return;
                }

                ch.wander_timer -= dt;
                if ch.wander_timer <= 0.0 {
                    if ch.wander_count >= ch.wander_limit {
                        // Try to sit in the lounge
                        let mut rng = rand::thread_rng();
                        if rng.gen_bool(LOUNGE_SIT_CHANCE) {
                            if let Some((idx, col, row)) = self.pick_lounge_seat() {
                                let path = pathfinding::find_path(
                                    ch.col, ch.row, col, row, &self.layout,
                                );
                                if !path.is_empty() {
                                    self.lounge_seat_occupied[idx] = true;
                                    ch.lounge_seat = Some(idx);
                                    ch.path = path;
                                    ch.state = CharacterState::Walking;
                                    ch.move_progress = 0.0;
                                    ch.wander_count = 0;
                                    ch.wander_limit = random_int(WANDER_MOVES_BEFORE_REST_MIN, WANDER_MOVES_BEFORE_REST_MAX);
                                    return;
                                }
                            }
                        }
                        // Otherwise return to desk seat
                        let path = pathfinding::find_path(
                            ch.col, ch.row, ch.seat_col, ch.seat_row, &self.layout,
                        );
                        if !path.is_empty() {
                            ch.path = path;
                            ch.state = CharacterState::Walking;
                            ch.move_progress = 0.0;
                            ch.wander_count = 0;
                            ch.wander_limit = random_int(WANDER_MOVES_BEFORE_REST_MIN, WANDER_MOVES_BEFORE_REST_MAX);
                            return;
                        }
                    }

                    // Pick random walkable tile to wander to
                    if !self.walkable_tiles.is_empty() {
                        let mut rng = rand::thread_rng();
                        let target = self.walkable_tiles[rng.gen_range(0..self.walkable_tiles.len())];
                        let path = pathfinding::find_path(
                            ch.col, ch.row, target.0, target.1, &self.layout,
                        );
                        if !path.is_empty() && path.len() < 30 {
                            ch.path = path;
                            ch.state = CharacterState::Walking;
                            ch.move_progress = 0.0;
                            ch.wander_count += 1;
                        }
                    }
                    ch.wander_timer = random_range(WANDER_PAUSE_MIN_SEC, WANDER_PAUSE_MAX_SEC);
                }
            }
            CharacterState::Walking => {
                if ch.frame_timer >= WALK_FRAME_DURATION_SEC {
                    ch.frame_timer -= WALK_FRAME_DURATION_SEC;
                    ch.frame = (ch.frame + 1) % 4;
                }

                ch.move_progress += WALK_SPEED_TILES_PER_SEC * dt;

                if ch.path.is_empty() {
                    // Arrived — check if at a lounge seat
                    if ch.lounge_seat.is_some() {
                        ch.state = CharacterState::Sitting;
                        ch.dir = Direction::Down;
                        ch.seat_timer = random_range(LOUNGE_SIT_MIN_SEC, LOUNGE_SIT_MAX_SEC);
                    } else if ch.is_active && ch.col == ch.seat_col && ch.row == ch.seat_row {
                        ch.state = CharacterState::Typing;
                        ch.dir = Direction::Up;
                    } else if !ch.is_active && ch.col == ch.seat_col && ch.row == ch.seat_row {
                        ch.state = CharacterState::Typing;
                        ch.dir = Direction::Up;
                        ch.seat_timer = random_range(SEAT_REST_MIN_SEC, SEAT_REST_MAX_SEC);
                    } else {
                        ch.state = CharacterState::Idle;
                        ch.wander_timer = random_range(WANDER_PAUSE_MIN_SEC, WANDER_PAUSE_MAX_SEC);
                    }
                    ch.frame = 0;
                    ch.frame_timer = 0.0;
                    return;
                }

                if ch.move_progress >= 1.0 {
                    ch.move_progress -= 1.0;
                    let (next_col, next_row) = ch.path[0];

                    if next_col > ch.col {
                        ch.dir = Direction::Right;
                    } else if next_col < ch.col {
                        ch.dir = Direction::Left;
                    } else if next_row > ch.row {
                        ch.dir = Direction::Down;
                    } else if next_row < ch.row {
                        ch.dir = Direction::Up;
                    }

                    ch.col = next_col;
                    ch.row = next_row;
                    ch.path.remove(0);
                }

                // If became active while wandering, repath to seat
                if ch.is_active {
                    // Free lounge seat if heading there
                    self.free_lounge_seat(ch);

                    let last = ch.path.last().copied();
                    if last != Some((ch.seat_col, ch.seat_row)) {
                        let path = pathfinding::find_path(
                            ch.col, ch.row, ch.seat_col, ch.seat_row, &self.layout,
                        );
                        if !path.is_empty() {
                            ch.path = path;
                            ch.move_progress = 0.0;
                        }
                    }
                }
            }
        }
    }
}

fn random_range(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

fn random_int(min: u32, max: u32) -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}
