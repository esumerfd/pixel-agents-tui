#![allow(dead_code)]

use crate::types::*;
use ratatui::style::Color;

/// Appearance: (skin, hair) — unique per person
const APPEARANCES: [(Color, Color); 8] = [
    (Color::Rgb(255, 213, 170), Color::Rgb(80, 50, 30)),    // light/peach, brown hair
    (Color::Rgb(101, 67, 33),   Color::Rgb(20, 15, 10)),    // dark brown, black hair
    (Color::Rgb(255, 224, 189), Color::Rgb(180, 120, 60)),   // fair/rosy, auburn hair
    (Color::Rgb(195, 150, 100), Color::Rgb(40, 25, 15)),    // medium/olive, dark hair
    (Color::Rgb(130, 80, 45),   Color::Rgb(15, 10, 8)),     // deep brown, black hair
    (Color::Rgb(225, 185, 145), Color::Rgb(60, 40, 25)),    // warm/tan, dark brown hair
    (Color::Rgb(175, 125, 80),  Color::Rgb(50, 30, 18)),    // caramel, espresso hair
    (Color::Rgb(240, 200, 160), Color::Rgb(140, 90, 40)),   // golden, chestnut hair
];

/// Team colors: (shirt, pants, shoes) — shared by agent + subagents
const TEAM_COLORS: [(Color, Color, Color); 6] = [
    (Color::Rgb(70, 130, 200),  Color::Rgb(50, 50, 80),  Color::Rgb(60, 40, 30)),   // blue
    (Color::Rgb(60, 160, 80),   Color::Rgb(60, 60, 70),  Color::Rgb(50, 35, 25)),   // green
    (Color::Rgb(200, 60, 60),   Color::Rgb(40, 40, 60),  Color::Rgb(70, 45, 35)),   // red
    (Color::Rgb(140, 70, 180),  Color::Rgb(50, 45, 65),  Color::Rgb(55, 38, 28)),   // purple
    (Color::Rgb(220, 140, 50),  Color::Rgb(55, 50, 70),  Color::Rgb(65, 42, 32)),   // orange
    (Color::Rgb(50, 170, 170),  Color::Rgb(45, 55, 75),  Color::Rgb(58, 40, 30)),   // teal
];

const EYES: Color = Color::Rgb(40, 40, 40);

const T: Option<Color> = None; // transparent

/// Get the shirt color for a palette index (used as agent identifier in status panels).
pub fn palette_color(palette: u8) -> Color {
    TEAM_COLORS[palette as usize % TEAM_COLORS.len()].0
}

/// A sprite row: each cell is (character, foreground, optional background).
/// Width = CHARACTER_WIDTH, Height = CHARACTER_HEIGHT.
pub const CHARACTER_WIDTH: u16 = 5;
pub const CHARACTER_HEIGHT: u16 = 5;

pub struct SpriteCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Option<Color>,
}

/// Get a 5x5 character sprite grid for the given character state.
pub fn get_character_grid(ch: &Character) -> Vec<Vec<SpriteCell>> {
    let (skin, hair) = APPEARANCES[ch.person as usize % APPEARANCES.len()];
    let (shirt, pants, shoes) = TEAM_COLORS[ch.palette as usize % TEAM_COLORS.len()];

    match ch.state {
        CharacterState::Typing => typing_sprite(ch.frame, skin, hair, shirt, pants, EYES),
        CharacterState::Reading => reading_sprite(ch.frame, skin, hair, shirt, pants, EYES),
        CharacterState::Walking => walking_sprite(ch.frame, skin, hair, shirt, pants, shoes, EYES),
        CharacterState::Idle => idle_sprite(skin, hair, shirt, pants, shoes, EYES),
        CharacterState::Sitting => sitting_sprite(skin, hair, shirt, EYES, pants),
    }
}

fn c(ch: char, fg: Color) -> SpriteCell {
    SpriteCell { ch, fg, bg: None }
}

fn cb(ch: char, fg: Color, bg: Color) -> SpriteCell {
    SpriteCell { ch, fg, bg: Some(bg) }
}

fn empty() -> SpriteCell {
    SpriteCell { ch: ' ', fg: Color::Reset, bg: None }
}

fn typing_sprite(frame: u8, skin: Color, hair: Color, shirt: Color, pants: Color, eyes: Color) -> Vec<Vec<SpriteCell>> {
    let f = frame % 2;
    vec![
        //       0              1              2              3              4
        vec![ empty(),      c('\u{2584}', hair), c('\u{2584}', hair), c('\u{2584}', hair), empty()       ],  // hair
        vec![ empty(),      cb('\u{25CF}', eyes, skin), c('\u{2588}', skin), cb('\u{25CF}', eyes, skin), empty()       ],  // face
        vec![ if f==0 {c('\u{2500}', shirt)} else {c('\u{2571}', shirt)},
                            c('\u{2588}', shirt), c('\u{2588}', shirt), c('\u{2588}', shirt),
              if f==0 {c('\u{2500}', shirt)} else {c('\u{2572}', shirt)} ],  // torso+arms
        vec![ empty(),      c('\u{2588}', pants), c('\u{2588}', pants), c('\u{2588}', pants), empty()       ],  // pants
        vec![ empty(),      c('\u{2584}', pants), empty(),              c('\u{2584}', pants), empty()       ],  // feet
    ]
}

fn reading_sprite(_frame: u8, skin: Color, hair: Color, shirt: Color, pants: Color, eyes: Color) -> Vec<Vec<SpriteCell>> {
    let book = Color::Rgb(200, 200, 220);
    vec![
        vec![ empty(),      c('\u{2584}', hair), c('\u{2584}', hair), c('\u{2584}', hair), empty()       ],
        vec![ empty(),      cb('\u{25CF}', eyes, skin), c('\u{2588}', skin), cb('\u{25CF}', eyes, skin), empty() ],
        vec![ empty(),      c('\u{2588}', shirt), c('\u{2588}', shirt), c('\u{2588}', shirt), c('\u{2590}', book) ],
        vec![ empty(),      c('\u{2588}', pants), c('\u{2588}', pants), c('\u{2588}', pants), empty()       ],
        vec![ empty(),      c('\u{2584}', pants), empty(),              c('\u{2584}', pants), empty()       ],
    ]
}

fn walking_sprite(frame: u8, skin: Color, hair: Color, shirt: Color, pants: Color, shoes: Color, eyes: Color) -> Vec<Vec<SpriteCell>> {
    let f = frame % 4;
    let (left_leg, right_leg) = match f {
        0 => (c('\u{2588}', pants), c('\u{2588}', pants)),
        1 => (c('\u{2571}', pants), c('\u{2588}', pants)),
        2 => (c('\u{2588}', pants), c('\u{2588}', pants)),
        _ => (c('\u{2588}', pants), c('\u{2572}', pants)),
    };
    let (left_foot, right_foot) = match f {
        1 => (c('\u{2584}', shoes), c('\u{2584}', pants)),
        3 => (c('\u{2584}', pants), c('\u{2584}', shoes)),
        _ => (c('\u{2584}', shoes), c('\u{2584}', shoes)),
    };
    vec![
        vec![ empty(),      c('\u{2584}', hair), c('\u{2584}', hair), c('\u{2584}', hair), empty()       ],
        vec![ empty(),      cb('\u{25CF}', eyes, skin), c('\u{2588}', skin), cb('\u{25CF}', eyes, skin), empty()       ],
        vec![ empty(),      c('\u{2588}', shirt), c('\u{2588}', shirt), c('\u{2588}', shirt), empty()       ],
        vec![ empty(),      left_leg,             empty(),              right_leg,            empty()       ],
        vec![ empty(),      left_foot,            empty(),              right_foot,           empty()       ],
    ]
}

fn idle_sprite(skin: Color, hair: Color, shirt: Color, pants: Color, shoes: Color, eyes: Color) -> Vec<Vec<SpriteCell>> {
    vec![
        vec![ empty(),      c('\u{2584}', hair), c('\u{2584}', hair), c('\u{2584}', hair), empty()       ],
        vec![ empty(),      cb('\u{25CF}', eyes, skin), c('\u{2588}', skin), cb('\u{25CF}', eyes, skin), empty()       ],
        vec![ empty(),      c('\u{2588}', shirt), c('\u{2588}', shirt), c('\u{2588}', shirt), empty()       ],
        vec![ empty(),      c('\u{2588}', pants), empty(),              c('\u{2588}', pants), empty()       ],
        vec![ empty(),      c('\u{2584}', shoes), empty(),              c('\u{2584}', shoes), empty()       ],
    ]
}

fn sitting_sprite(skin: Color, hair: Color, shirt: Color, eyes: Color, _pants: Color) -> Vec<Vec<SpriteCell>> {
    vec![
        vec![ empty(),      c('\u{2584}', hair), c('\u{2584}', hair), c('\u{2584}', hair), empty()       ],
        vec![ empty(),      cb('\u{25CF}', eyes, skin), c('\u{2588}', skin), cb('\u{25CF}', eyes, skin), empty()       ],
        vec![ empty(),      c('\u{2588}', shirt), c('\u{2588}', shirt), c('\u{2588}', shirt), empty()       ],
    ]
}

/// Subagent sprite dimensions (3x4 smaller people)
pub const SUBAGENT_WIDTH: u16 = 3;
pub const SUBAGENT_HEIGHT: u16 = 4;

/// Get a 3x4 subagent sprite grid.
pub fn get_subagent_grid(ch: &Character) -> Vec<Vec<SpriteCell>> {
    let (skin, hair) = APPEARANCES[ch.person as usize % APPEARANCES.len()];
    let (shirt, pants, shoes) = TEAM_COLORS[ch.palette as usize % TEAM_COLORS.len()];
    let e = EYES;

    match ch.state {
        CharacterState::Typing | CharacterState::Reading => {
            let f = ch.frame % 2;
            vec![
                vec![ empty(), c('\u{2584}', hair), empty() ],
                vec![ empty(), cb('\u{25CF}', e, skin), empty() ],
                vec![ if f==0 {c('\u{2500}', shirt)} else {c('\u{2571}', shirt)},
                      c('\u{2588}', shirt),
                      if f==0 {c('\u{2500}', shirt)} else {c('\u{2572}', shirt)} ],
                vec![ empty(), c('\u{2588}', pants), empty() ],
            ]
        }
        CharacterState::Walking => {
            let f = ch.frame % 4;
            let (ll, rl) = match f {
                1 => (c('\u{2584}', shoes), c('\u{2584}', pants)),
                3 => (c('\u{2584}', pants), c('\u{2584}', shoes)),
                _ => (c('\u{2584}', shoes), c('\u{2584}', shoes)),
            };
            vec![
                vec![ empty(), c('\u{2584}', hair), empty() ],
                vec![ empty(), cb('\u{25CF}', e, skin), empty() ],
                vec![ empty(), c('\u{2588}', shirt), empty() ],
                vec![ ll, empty(), rl ],
            ]
        }
        CharacterState::Idle => {
            vec![
                vec![ empty(), c('\u{2584}', hair), empty() ],
                vec![ empty(), cb('\u{25CF}', e, skin), empty() ],
                vec![ empty(), c('\u{2588}', shirt), empty() ],
                vec![ empty(), c('\u{2588}', pants), empty() ],
            ]
        }
        CharacterState::Sitting => {
            vec![
                vec![ empty(), c('\u{2584}', hair), empty() ],
                vec![ empty(), cb('\u{25CF}', e, skin), empty() ],
                vec![ empty(), c('\u{2588}', shirt), empty() ],
            ]
        }
    }
}

/// Desk sprite: 8 wide x 3 tall
pub const DESK_WIDTH: u16 = 8;
pub const DESK_HEIGHT: u16 = 3;

pub fn get_desk_grid() -> Vec<Vec<SpriteCell>> {
    let w = Color::Rgb(139, 105, 20);   // wood
    let d = Color::Rgb(107, 78, 10);    // dark wood
    vec![
        // top surface
        vec![ c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w) ],
        // front face
        vec![ c('\u{2588}', w), c('\u{2588}', w), c('\u{2588}', w), c('\u{2588}', w), c('\u{2588}', w), c('\u{2588}', w), c('\u{2588}', w), c('\u{2588}', w) ],
        // legs
        vec![ c('\u{2588}', d), c('\u{2584}', d), empty(), empty(), empty(), empty(), c('\u{2584}', d), c('\u{2588}', d) ],
    ]
}

/// Get furniture sprite grid by kind.
pub fn get_furniture_grid(kind: crate::layout::FurnitureKind) -> Vec<Vec<SpriteCell>> {
    use crate::layout::FurnitureKind::*;
    match kind {
        Desk => get_desk_grid(),
        Chair => chair_grid(),
        Bookshelf => bookshelf_grid(),
        Plant => plant_grid(),
        WaterCooler => cooler_grid(),
        Whiteboard => whiteboard_grid(),
        RoundTable => round_table_grid(),
        Couch => couch_grid(),
        VendingMachine => vending_grid(),
        Counter => counter_grid(),
        Lamp => lamp_grid(),
        Monitor => monitor_grid(),
    }
}

fn chair_grid() -> Vec<Vec<SpriteCell>> {
    let w = Color::Rgb(80, 80, 100);
    vec![
        vec![ c('\u{250C}', w), c('\u{2500}', w), c('\u{2510}', w) ],
    ]
}

fn bookshelf_grid() -> Vec<Vec<SpriteCell>> {
    let w = Color::Rgb(120, 85, 20);
    let b1 = Color::Rgb(180, 60, 60);
    let b2 = Color::Rgb(60, 100, 160);
    let b3 = Color::Rgb(60, 140, 80);
    vec![
        vec![ c('\u{2554}', w), c('\u{2550}', w), c('\u{2550}', w), c('\u{2557}', w) ],
        vec![ c('\u{2551}', w), c('\u{2588}', b1), c('\u{2588}', b2), c('\u{2551}', w) ],
        vec![ c('\u{2551}', w), c('\u{2588}', b3), c('\u{2588}', b1), c('\u{2551}', w) ],
    ]
}

fn plant_grid() -> Vec<Vec<SpriteCell>> {
    let g = Color::Rgb(60, 140, 55);
    let p = Color::Rgb(160, 90, 50);
    vec![
        vec![ c('\u{2663}', g), c('\u{2663}', g) ],
        vec![ c('\u{2590}', p), c('\u{258C}', p) ],
    ]
}

fn cooler_grid() -> Vec<Vec<SpriteCell>> {
    let b = Color::Rgb(140, 170, 200);
    let d = Color::Rgb(100, 120, 150);
    vec![
        vec![ c('\u{250C}', b), c('\u{2500}', b), c('\u{2510}', b) ],
        vec![ c('\u{2502}', d), c('\u{2592}', b), c('\u{2502}', d) ],
    ]
}

fn whiteboard_grid() -> Vec<Vec<SpriteCell>> {
    let f = Color::Rgb(180, 180, 190);  // frame
    let w = Color::Rgb(220, 220, 230);  // white surface
    let t = Color::Rgb(80, 120, 180);   // text marks
    vec![
        vec![ c('\u{250C}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2510}', f) ],
        vec![ c('\u{2502}', f), cb('\u{2500}', t, w), cb(' ', t, w), cb('\u{2500}', t, w), cb(' ', t, w), cb('\u{2500}', t, w), cb('\u{2500}', t, w), cb(' ', t, w), cb('\u{2500}', t, w), c('\u{2502}', f) ],
    ]
}

fn round_table_grid() -> Vec<Vec<SpriteCell>> {
    let w = Color::Rgb(120, 100, 70);
    let s = Color::Rgb(140, 115, 80);
    vec![
        vec![ c('\u{256D}', w), c('\u{2500}', s), c('\u{2500}', s), c('\u{2500}', s), c('\u{2500}', s), c('\u{256E}', w) ],
        vec![ c('\u{2570}', w), c('\u{2500}', s), c('\u{2500}', s), c('\u{2500}', s), c('\u{2500}', s), c('\u{256F}', w) ],
    ]
}

fn couch_grid() -> Vec<Vec<SpriteCell>> {
    let c1 = Color::Rgb(100, 60, 120);  // purple couch
    let c2 = Color::Rgb(80, 45, 95);
    vec![
        vec![ c('\u{2588}', c1), c('\u{2584}', c1), c('\u{2584}', c1), c('\u{2584}', c1), c('\u{2584}', c1), c('\u{2584}', c1), c('\u{2584}', c1), c('\u{2588}', c1) ],
        vec![ c('\u{2588}', c2), c('\u{2588}', c1), c('\u{2588}', c1), c('\u{2588}', c1), c('\u{2588}', c1), c('\u{2588}', c1), c('\u{2588}', c1), c('\u{2588}', c2) ],
    ]
}

fn vending_grid() -> Vec<Vec<SpriteCell>> {
    let f = Color::Rgb(60, 80, 120);
    let g = Color::Rgb(80, 180, 100);
    let d = Color::Rgb(40, 55, 85);
    vec![
        vec![ c('\u{250C}', f), c('\u{2500}', f), c('\u{2500}', f), c('\u{2510}', f) ],
        vec![ c('\u{2502}', f), cb('\u{2592}', g, d), cb('\u{2592}', g, d), c('\u{2502}', f) ],
        vec![ c('\u{2514}', f), c('\u{2500}', d), c('\u{2500}', d), c('\u{2518}', f) ],
    ]
}

fn counter_grid() -> Vec<Vec<SpriteCell>> {
    let w = Color::Rgb(150, 130, 100);
    let d = Color::Rgb(110, 95, 70);
    vec![
        vec![ c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w), c('\u{2584}', w) ],
        vec![ c('\u{2588}', d), c('\u{2592}', w), c('\u{2592}', w), c('\u{2592}', w), c('\u{2592}', w), c('\u{2592}', w), c('\u{2592}', w), c('\u{2588}', d) ],
    ]
}

fn lamp_grid() -> Vec<Vec<SpriteCell>> {
    let l = Color::Rgb(220, 200, 100);
    let s = Color::Rgb(120, 110, 80);
    vec![
        vec![ c('\u{25CB}', l) ],
        vec![ c('\u{2502}', s) ],
    ]
}

fn monitor_grid() -> Vec<Vec<SpriteCell>> {
    let m = Color::Rgb(100, 140, 180);
    let b = Color::Rgb(60, 60, 80);
    vec![
        vec![ cb('\u{2588}', m, b), cb('\u{2588}', m, b) ],
    ]
}

/// Floor tile characters (kept for compatibility but layout now handles floor rendering)
pub fn floor_char(col: u16, row: u16) -> (char, Color) {
    if (col + row) % 2 == 0 {
        ('\u{2591}', Color::Rgb(60, 60, 80))
    } else {
        ('\u{2591}', Color::Rgb(50, 50, 70))
    }
}
