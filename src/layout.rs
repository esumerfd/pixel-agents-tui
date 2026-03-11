#![allow(dead_code)]

/// Tile types for the office grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Void,      // outside building
    Wall,      // impassable wall
    Floor,     // standard walkable floor
    Corridor,  // walkable corridor (different color)
    Carpet,    // lounge floor
    TileFloor, // snack bar tiled floor
}

impl Tile {
    pub fn is_walkable(self) -> bool {
        matches!(self, Tile::Floor | Tile::Corridor | Tile::Carpet | Tile::TileFloor)
    }
}

/// Furniture placed in the office
#[derive(Debug, Clone)]
pub struct Furniture {
    pub kind: FurnitureKind,
    pub col: u16,
    pub row: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnitureKind {
    Desk,
    Chair,
    Bookshelf,
    Plant,
    WaterCooler,
    Whiteboard,
    RoundTable,
    Couch,
    VendingMachine,
    Counter,
    Lamp,
    Monitor,
}

impl FurnitureKind {
    /// Width x Height in terminal cells
    pub fn size(self) -> (u16, u16) {
        match self {
            FurnitureKind::Desk => (8, 3),
            FurnitureKind::Chair => (3, 1),
            FurnitureKind::Bookshelf => (4, 3),
            FurnitureKind::Plant => (2, 2),
            FurnitureKind::WaterCooler => (3, 2),
            FurnitureKind::Whiteboard => (10, 2),
            FurnitureKind::RoundTable => (6, 2),
            FurnitureKind::Couch => (8, 2),
            FurnitureKind::VendingMachine => (4, 3),
            FurnitureKind::Counter => (8, 2),
            FurnitureKind::Lamp => (1, 2),
            FurnitureKind::Monitor => (2, 1),
        }
    }

    pub fn is_desk(self) -> bool {
        matches!(self, FurnitureKind::Desk)
    }
}

/// Seat position where an agent can sit at a desk
#[derive(Debug, Clone)]
pub struct Seat {
    pub col: u16,
    pub row: u16,
    pub desk_col: u16,
    pub desk_row: u16,
}

/// A chair in the lounge where idle agents can sit
#[derive(Debug, Clone)]
pub struct LoungeSeat {
    pub col: u16,
    pub row: u16,
}

/// The complete office layout
pub struct OfficeLayout {
    pub cols: u16,
    pub rows: u16,
    pub tiles: Vec<Vec<Tile>>,
    pub furniture: Vec<Furniture>,
    pub seats: Vec<Seat>,
    pub lounge_seats: Vec<LoungeSeat>,
    pub blocked: std::collections::HashSet<(u16, u16)>,
    pub room_labels: Vec<(u16, u16, &'static str)>,
}

impl OfficeLayout {
    pub fn get_tile(&self, col: u16, row: u16) -> Tile {
        if row < self.rows && col < self.cols {
            self.tiles[row as usize][col as usize]
        } else {
            Tile::Void
        }
    }

    pub fn is_walkable(&self, col: u16, row: u16) -> bool {
        self.get_tile(col, row).is_walkable() && !self.blocked.contains(&(col, row))
    }
}

/// Build the office layout with dynamic desk count.
///
/// 3-room layout (86x38):
///   Work Room (left, Floor) — desks scale to agent count
///   Snack Bar (top-right, TileFloor)
///   Lounge (bottom, Carpet) — chairs for idle sitting
pub fn build_office(agent_count: usize) -> OfficeLayout {
    let cols: u16 = 86;
    let rows: u16 = 38;

    let mut tiles = vec![vec![Tile::Void; cols as usize]; rows as usize];

    let fill = |tiles: &mut Vec<Vec<Tile>>, c1: u16, r1: u16, c2: u16, r2: u16, tile: Tile| {
        for r in r1..=r2 {
            for c in c1..=c2 {
                tiles[r as usize][c as usize] = tile;
            }
        }
    };

    // ── Outer walls ──
    fill(&mut tiles, 0, 0, cols - 1, 0, Tile::Wall);
    fill(&mut tiles, 0, rows - 1, cols - 1, rows - 1, Tile::Wall);
    fill(&mut tiles, 0, 0, 0, rows - 1, Tile::Wall);
    fill(&mut tiles, cols - 1, 0, cols - 1, rows - 1, Tile::Wall);

    // ── Work Room: cols 1-50, rows 1-20 ──
    fill(&mut tiles, 1, 1, 50, 20, Tile::Floor);
    fill(&mut tiles, 51, 1, 51, 21, Tile::Wall);   // right wall
    fill(&mut tiles, 1, 21, 50, 21, Tile::Wall);    // bottom wall
    // door Work Room <-> Snack Bar (col 51, rows 7-8)
    tiles[7][51] = Tile::Floor;
    tiles[8][51] = Tile::Floor;
    // door Work Room <-> Lounge (row 21, cols 24-25)
    tiles[21][24] = Tile::Floor;
    tiles[21][25] = Tile::Floor;

    // ── Snack Bar: cols 52-84, rows 1-14 ──
    fill(&mut tiles, 52, 1, cols - 2, 14, Tile::TileFloor);
    fill(&mut tiles, 52, 15, cols - 2, 15, Tile::Wall); // bottom wall
    // door Snack Bar <-> Lounge (row 15, cols 68-69)
    tiles[15][68] = Tile::TileFloor;
    tiles[15][69] = Tile::TileFloor;

    // ── Lounge: cols 1-84, rows 22-36 (full width) ──
    fill(&mut tiles, 1, 22, cols - 2, rows - 2, Tile::Carpet);
    // extend wall between snack bar and lounge (col 51 from row 15 down to row 21)
    // (already covered by work room right wall)
    // lounge needs access from snack bar side too
    fill(&mut tiles, 52, 16, cols - 2, 21, Tile::Carpet);
    // wall segment between snack bar and lounge right side
    // (row 15 wall already placed, door at 68-69 provides access)

    // ── Furniture ──
    let mut furniture = Vec::new();

    // Work Room: dynamic desks
    let desk_positions = place_desks(agent_count);
    for (dc, dr) in &desk_positions {
        furniture.push(Furniture { kind: FurnitureKind::Desk, col: *dc, row: *dr });
    }
    // Work Room decorations
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 1, row: 1 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 49, row: 1 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 1, row: 19 });

    // Snack Bar furniture
    furniture.push(Furniture { kind: FurnitureKind::VendingMachine, col: 54, row: 1 });
    furniture.push(Furniture { kind: FurnitureKind::VendingMachine, col: 60, row: 1 });
    furniture.push(Furniture { kind: FurnitureKind::Counter, col: 70, row: 1 });
    furniture.push(Furniture { kind: FurnitureKind::RoundTable, col: 56, row: 7 });
    furniture.push(Furniture { kind: FurnitureKind::RoundTable, col: 70, row: 7 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 82, row: 1 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 54, row: 12 });

    // Lounge furniture — couches and chairs spread across the full width
    furniture.push(Furniture { kind: FurnitureKind::Couch, col: 4, row: 23 });
    furniture.push(Furniture { kind: FurnitureKind::Couch, col: 4, row: 28 });
    furniture.push(Furniture { kind: FurnitureKind::Couch, col: 4, row: 33 });
    furniture.push(Furniture { kind: FurnitureKind::Couch, col: 30, row: 23 });
    furniture.push(Furniture { kind: FurnitureKind::Couch, col: 56, row: 23 });
    furniture.push(Furniture { kind: FurnitureKind::RoundTable, col: 16, row: 25 });
    furniture.push(Furniture { kind: FurnitureKind::RoundTable, col: 42, row: 25 });
    furniture.push(Furniture { kind: FurnitureKind::RoundTable, col: 68, row: 25 });
    furniture.push(Furniture { kind: FurnitureKind::Bookshelf, col: 78, row: 22 });
    furniture.push(Furniture { kind: FurnitureKind::WaterCooler, col: 50, row: 22 });
    furniture.push(Furniture { kind: FurnitureKind::Lamp, col: 14, row: 22 });
    furniture.push(Furniture { kind: FurnitureKind::Lamp, col: 40, row: 22 });
    furniture.push(Furniture { kind: FurnitureKind::Lamp, col: 66, row: 22 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 1, row: 35 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 82, row: 35 });
    furniture.push(Furniture { kind: FurnitureKind::Plant, col: 82, row: 22 });

    // ── Build blocked tile set from furniture ──
    let mut blocked = std::collections::HashSet::new();
    for f in &furniture {
        let (w, h) = f.kind.size();
        for dr in 0..h {
            for dc in 0..w {
                blocked.insert((f.col + dc, f.row + dr));
            }
        }
    }

    // ── Build desk seats ──
    let mut seats = Vec::new();
    for f in &furniture {
        if f.kind.is_desk() {
            let (w, _h) = f.kind.size();
            let seat_col = f.col + w / 2;
            let seat_row = f.row + 3 + 1;
            if seat_row < rows && tiles[seat_row as usize][seat_col as usize].is_walkable()
                && !blocked.contains(&(seat_col, seat_row))
            {
                seats.push(Seat {
                    col: seat_col,
                    row: seat_row,
                    desk_col: f.col,
                    desk_row: f.row,
                });
            }
        }
    }

    // ── Lounge seats (walkable spots near couches) ──
    let lounge_seats = vec![
        // Near couch row 1 (row 23)
        LoungeSeat { col: 14, row: 24 },
        LoungeSeat { col: 16, row: 24 },
        LoungeSeat { col: 38, row: 24 },
        LoungeSeat { col: 40, row: 24 },
        LoungeSeat { col: 64, row: 24 },
        LoungeSeat { col: 66, row: 24 },
        // Near couch row 2 (row 28)
        LoungeSeat { col: 14, row: 29 },
        LoungeSeat { col: 16, row: 29 },
        // Near couch row 3 (row 33)
        LoungeSeat { col: 14, row: 34 },
        LoungeSeat { col: 16, row: 34 },
        // Scattered seats in the lounge
        LoungeSeat { col: 30, row: 30 },
        LoungeSeat { col: 50, row: 30 },
        LoungeSeat { col: 70, row: 30 },
        LoungeSeat { col: 30, row: 34 },
        LoungeSeat { col: 60, row: 34 },
    ];

    // ── Room labels ──
    let room_labels = vec![
        (18, 0, "WORK ROOM"),
        (63, 0, "SNACK BAR"),
        (38, 21, "LOUNGE"),
    ];

    OfficeLayout {
        cols,
        rows,
        tiles,
        furniture,
        seats,
        lounge_seats,
        blocked,
        room_labels,
    }
}

/// Calculate desk positions in the work room based on agent count.
/// Work room spans cols 1-50, rows 1-20.
/// Desks are 8x3, laid out in 2 columns x 3 rows.
fn place_desks(agent_count: usize) -> Vec<(u16, u16)> {
    let positions = [
        (4, 2),   (24, 2),   // row 1
        (4, 8),   (24, 8),   // row 2
        (4, 14),  (24, 14),  // row 3
    ];
    let count = agent_count.max(2).min(positions.len());
    positions[..count].to_vec()
}
