use std::collections::{HashSet, VecDeque};

use crate::layout::OfficeLayout;

/// BFS pathfinding on the tile grid.
/// Returns path as Vec of (col, row) steps, excluding start, including end.
/// Returns empty vec if no path exists or start == end.
pub fn find_path(
    start_col: u16,
    start_row: u16,
    end_col: u16,
    end_row: u16,
    layout: &OfficeLayout,
) -> Vec<(u16, u16)> {
    if start_col == end_col && start_row == end_row {
        return Vec::new();
    }

    if !layout.is_walkable(end_col, end_row) {
        return Vec::new();
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut came_from = std::collections::HashMap::new();

    let start = (start_col, start_row);
    let end = (end_col, end_row);

    visited.insert(start);
    queue.push_back(start);

    let directions: [(i16, i16); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

    while let Some(current) = queue.pop_front() {
        if current == end {
            // Reconstruct path
            let mut path = Vec::new();
            let mut node = end;
            while node != start {
                path.push(node);
                node = came_from[&node];
            }
            path.reverse();
            return path;
        }

        for (dc, dr) in &directions {
            let nc = current.0 as i16 + dc;
            let nr = current.1 as i16 + dr;

            if nc < 0 || nr < 0 || nc >= layout.cols as i16 || nr >= layout.rows as i16 {
                continue;
            }

            let next = (nc as u16, nr as u16);
            if visited.contains(&next) {
                continue;
            }

            if !layout.is_walkable(next.0, next.1) {
                continue;
            }

            visited.insert(next);
            came_from.insert(next, current);
            queue.push_back(next);
        }
    }

    Vec::new() // no path found
}

/// Get all walkable tiles in the layout, excluding positions near edges
/// where a character sprite would be clipped.
/// A 5x5 sprite centered on (col, row) occupies cols [col-2, col+2] and rows [row-2, row+2].
/// We need margin of half_w+1 on each side to keep the sprite fully inside the grid.
pub fn get_walkable_tiles(layout: &OfficeLayout) -> Vec<(u16, u16)> {
    let sprite_margin_top: u16 = 3;
    let sprite_margin_bottom: u16 = 3;
    let sprite_margin_left: u16 = 3;
    let sprite_margin_right: u16 = 3;

    let mut tiles = Vec::new();
    for row in 0..layout.rows {
        for col in 0..layout.cols {
            if layout.is_walkable(col, row)
                && row >= sprite_margin_top
                && row + sprite_margin_bottom < layout.rows
                && col >= sprite_margin_left
                && col + sprite_margin_right < layout.cols
            {
                tiles.push((col, row));
            }
        }
    }
    tiles
}
