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
) -> Vec<Option<(usize, usize, celldata::CellState)>> {
    let ret = pos
        .into_iter()
        .map(|Pos { x, y }| {
            let ret = match (x.try_into(), y.try_into()) {
                (Ok(x1), Ok(y1)) => Pos { x: x1, y: y1 },
                _ => Pos {
                    x: usize::MAX,
                    y: usize::MAX,
                },
            };
            ret
        })
        .map(|Pos { x, y }| match m.get(x) {
            Some(v) => match v.get(y) {
                None => None,
                Some(&a) => Some((x, y, a)),
            },
            None => None,
        })
        .collect();
    return ret;
}

pub fn get_connected(
    x0: usize,
    y0: usize,
    t: celldata::CellStateVariant,
    m: &Board,
) -> impl IntoIterator<Item = (usize, usize, celldata::CellState)> {
    let mut set_size = 0;
    let mut connected: HashSet<(usize, usize, celldata::CellState)> = neighbors(x0, y0, m)
        .iter()
        .filter_map(|&i| match i {
            Some((_x, _y, a)) => {
                if t == a.into() {
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
            .flat_map(|(x, y, _)| neighbors(*x, *y, m))
            .filter_map(|i| match i {
                Some((_x, _y, a)) if t == a.into() => i,
                _ => None,
            })
            .collect();
        connected = connected.union(&new_connected).map(|i| i.clone()).collect();
    }
    return connected;
}

pub fn neighbors(
    x0: usize,
    y0: usize,
    m: &Board,
) -> Vec<Option<(usize, usize, celldata::CellState)>> {
    let x: i32 = x0.try_into().unwrap();
    let y: i32 = y0.try_into().unwrap();
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
