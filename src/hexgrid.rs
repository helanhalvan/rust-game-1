use std::{collections::HashSet};

use crate::celldata;
use std::hash::Hash;

pub type Hexgrid<T> = Vec<Vec<T>>;
pub type Board = Hexgrid<celldata::CellState>;

pub type Pos = XYCont<usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XYCont<C> {
    pub x: C,
    pub y: C,
}

pub fn sub_matrix<T: Clone>(
    source: &Hexgrid<T>,
    _center @ XYCont { x, y }: XYCont<i32>,
    height_extra: i32,
    width_extra: i32,
    default: T,
) -> Hexgrid<T> {
    let mut ret = vec![];
    for dx in 0..(height_extra + 1) {
        let mut y_buff = vec![];
        for dy in 0..(width_extra + 1) {
            let new = if let Some((_, new)) = to_pos_cell(
                XYCont {
                    x: x + dx,
                    y: y + dy,
                },
                source,
            ) {
                new
            } else {
                default.clone()
            };
            y_buff.push(new);
        }
        ret.push(y_buff)
    }
    ret
}

pub fn to_pos_cell<T: Clone, C: TryInto<usize>>(
    XYCont { x: raw_x, y: raw_y }: XYCont<C>,
    source: &Hexgrid<T>,
) -> Option<(Pos, T)> {
    match raw_x.try_into() {
        Ok(x1) => match raw_y.try_into() {
            Ok(y1) => {
                let x: usize = x1;
                let y: usize = y1;
                match source.get(x) {
                    Some(v) => match v.get(y) {
                        Some(i) => Some((Pos { x, y }, i.clone())),
                        None => None,
                    },
                    None => None,
                }
            }
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub fn pos_iter_to_cells<'a, T: Clone>(
    pos: impl IntoIterator<Item = Pos> + 'a,
    m: &'a Hexgrid<T>,
) -> impl Iterator<Item = Option<(Pos, T)>> + 'a {
    let ret = pos.into_iter().map(|p @ Pos { x, y }| match m.get(x) {
        Some(v) => match v.get(y) {
            None => None,
            Some(a) => Some((p, a.clone())),
        },
        None => None,
    });
    return ret;
}

pub fn get_connected<T: Clone + std::cmp::Eq + std::hash::Hash>(
    p: Pos,
    t: fn(T) -> bool,
    m: &Hexgrid<T>,
) -> impl IntoIterator<Item = (Pos, T)> {
    let mut set_size = 0;
    let mut connected: HashSet<(Pos, _)> = neighbors(p, m)
        .filter_map(|i| match i.clone() {
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
            .filter_map(|i| match i.clone() {
                Some((_x, a)) if t(a.clone()) => i,
                _ => None,
            })
            .collect();
        connected = connected.union(&new_connected).map(|i| i.clone()).collect();
    }
    return connected;
}

pub fn neighbors<'a, T: Clone>(
    Pos { x: x0, y: y0 }: Pos,
    m: &'a Hexgrid<T>,
) -> impl Iterator<Item = Option<(Pos, T)>> + 'a {
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

pub fn set<T>(Pos { x, y }: Pos, new_cell: T, m: &mut Hexgrid<T>) {
    m[x][y] = new_cell;
}

pub fn get<T: Clone>(Pos { x, y }: Pos, m: &Hexgrid<T>) -> T {
    m[x][y].clone()
}

// is it really this easy to do distance in a hexgrid?
pub fn distance<C: TryInto<i32>>(
    XYCont {
        x: from_x,
        y: from_y,
    }: XYCont<C>,
    XYCont { x: to_x, y: to_y }: XYCont<C>,
) -> i32
where
    <C as TryInto<i32>>::Error: std::fmt::Debug,
{
    let abs_y = i32::abs(from_y.try_into().unwrap() - to_y.try_into().unwrap());
    let abs_x = i32::abs(from_x.try_into().unwrap() - to_x.try_into().unwrap());
    if abs_x > abs_y {
        abs_x
    } else {
        abs_y
    }
}
