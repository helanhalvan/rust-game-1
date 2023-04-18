use std::collections::HashSet;

use crate::celldata;

pub type Board = Vec<Vec<celldata::CellState>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

pub fn pos_iter_to_cells(
    pos: impl IntoIterator<Item = Pos>,
    m: &Board,
) -> Vec<Option<(Pos, celldata::CellState)>> {
    let ret = pos
        .into_iter()
        .map(|p @ Pos { x, y }| match m.get(x) {
            Some(v) => match v.get(y) {
                None => None,
                Some(&a) => Some((p, a)),
            },
            None => None,
        })
        .collect();
    return ret;
}

pub fn get_connected(
    p: Pos,
    t: fn(celldata::CellState) -> bool,
    m: &Board,
) -> impl IntoIterator<Item = (Pos, celldata::CellState)> {
    let mut set_size = 0;
    let mut connected: HashSet<(Pos, celldata::CellState)> = neighbors(p, m)
        .iter()
        .filter_map(|&i| match i {
            Some((_x, a)) => {
                if t(a) {
                    i
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    while connected.len() > set_size {
        set_size = connected.len();
        let new_connected = connected
            .iter()
            .flat_map(|(p, _)| neighbors(*p, m))
            .filter_map(|i| match i {
                Some((_x, a)) if t(a) => i,
                _ => None,
            })
            .collect();
        connected = connected.union(&new_connected).map(|i| i.clone()).collect();
    }
    return connected;
}

pub fn neighbors(Pos { x: x0, y: y0 }: Pos, m: &Board) -> Vec<Option<(Pos, celldata::CellState)>> {
    let x = x0 as i64;
    let y = y0 as i64;
    let hard_neighbors = if (x % 2) == 0 {
        [(x + 1, y + 1), (x - 1, y + 1)]
    } else {
        [(x + 1, y - 1), (x - 1, y - 1)]
    };
    let mut neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)].to_vec();
    neighbors.append(&mut hard_neighbors.to_vec());
    let pos_iter = neighbors
        .into_iter()
        .map(|(x, y)| match (x.try_into(), y.try_into()) {
            (Ok(x1), Ok(y1)) => Pos { x: x1, y: y1 },
            _ => Pos {
                x: usize::MAX,
                y: usize::MAX,
            },
        });
    let ret = pos_iter_to_cells(pos_iter, m);
    return ret;
}

pub fn set(Pos { x, y }: Pos, new_cell: celldata::CellState, m: &mut Board) {
    m[x][y] = new_cell;
}
