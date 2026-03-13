#![allow(dead_code)]

// ── Timing ──────────────────────────────────────────────────
pub const FILE_POLL_INTERVAL_MS: u64 = 1000;
pub const TOOL_DONE_DELAY_MS: u64 = 300;
pub const PERMISSION_TIMER_DELAY_MS: u64 = 7000;
pub const TEXT_IDLE_DELAY_MS: u64 = 5000;
pub const STALE_ACTIVE_TIMEOUT_SECS: u64 = 30;
pub const STALE_AGENT_REMOVE_SECS: u64 = 300; // remove agents with no activity for 5 minutes

// ── Display Truncation ──────────────────────────────────────
pub const BASH_COMMAND_DISPLAY_MAX_LENGTH: usize = 30;
pub const TASK_DESCRIPTION_DISPLAY_MAX_LENGTH: usize = 40;

// ── Grid & Layout ───────────────────────────────────────────
pub const TILE_SIZE: u16 = 2; // terminal cells per tile
pub const DEFAULT_COLS: u16 = 30;
pub const DEFAULT_ROWS: u16 = 15;

// ── Character Animation ─────────────────────────────────────
pub const WALK_SPEED_TILES_PER_SEC: f64 = 3.0;
pub const WALK_FRAME_DURATION_SEC: f64 = 0.15;
pub const TYPE_FRAME_DURATION_SEC: f64 = 0.3;
pub const WANDER_PAUSE_MIN_SEC: f64 = 2.0;
pub const WANDER_PAUSE_MAX_SEC: f64 = 8.0;
pub const WANDER_MOVES_BEFORE_REST_MIN: u32 = 3;
pub const WANDER_MOVES_BEFORE_REST_MAX: u32 = 6;
pub const SEAT_REST_MIN_SEC: f64 = 10.0;
pub const SEAT_REST_MAX_SEC: f64 = 30.0;
pub const LOUNGE_SIT_CHANCE: f64 = 0.4;
pub const LOUNGE_SIT_MIN_SEC: f64 = 15.0;
pub const LOUNGE_SIT_MAX_SEC: f64 = 45.0;

// ── Rendering ───────────────────────────────────────────────
pub const TICK_RATE_MS: u64 = 100; // 10 FPS render loop

// ── Snack Bar Visit ────────────────────────────────────────
pub const SNACK_BAR_VISIT_CHANCE: f64 = 0.3;
pub const VENDING_USE_MIN_SEC: f64 = 4.0;
pub const VENDING_USE_MAX_SEC: f64 = 10.0;
pub const VENDING_FRAME_DURATION_SEC: f64 = 0.4;

// ── Permission-exempt tools ─────────────────────────────────
pub const PERMISSION_EXEMPT_TOOLS: &[&str] = &["Task", "Agent", "AskUserQuestion"];
