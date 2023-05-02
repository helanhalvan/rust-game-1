use std::{
    collections::HashSet,
    ops::{Add, Mul},
};

use itertools::Itertools;

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

struct XYZCont<C> {
    x: C,
    y: C,
    z: C,
}

impl Mul<XYCont<i32>> for XYCont<i32> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        XYCont {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Mul<i32> for XYCont<i32> {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        XYCont {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<XYCont<i32>> for i32 {
    type Output = XYCont<i32>;
    fn mul(self, rhs: XYCont<i32>) -> Self::Output {
        XYCont {
            x: rhs.x * self,
            y: rhs.y * self,
        }
    }
}

impl Add<XYCont<i32>> for XYCont<i32> {
    type Output = XYCont<i32>;
    fn add(self, rhs: XYCont<i32>) -> Self::Output {
        XYCont {
            x: rhs.x + self.x,
            y: rhs.y + self.y,
        }
    }
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

pub fn within<'a, T: Clone>(
    Pos { x: x0, y: y0 }: Pos,
    m: &'a Hexgrid<T>,
    range: i32,
) -> impl Iterator<Item = Option<(Pos, T)>> + 'a {
    let o_x = x0 as i32;
    let o_y = y0 as i32;
    let origin = XYCont { x: o_x, y: o_y };
    let v1 = XYCont { x: 1, y: 0 };
    let v2 = XYCont { x: 0, y: 1 };
    let unit_vectors = vec![v1, v2];

    let vcounts = -range..range + 1;

    let close_space: HashSet<_> = (0..2)
        .map(|_| vcounts.clone())
        .multi_cartesian_product()
        .map(|v| v_mul_reduce(&v, &unit_vectors) + origin)
        .filter(|v| distance(*v, XYCont { x: o_x, y: o_y }) <= range)
        .map(|XYCont { x, y }| match (x.try_into(), y.try_into()) {
            (Ok(x1), Ok(y1)) => Pos { x: x1, y: y1 },
            _ => Pos {
                x: usize::MAX,
                y: usize::MAX,
            },
        })
        .collect();

    let ret = pos_iter_to_cells(close_space, m);
    return ret;
}

fn v_mul_reduce(v1: &Vec<i32>, v2: &Vec<XYCont<i32>>) -> XYCont<i32> {
    let mut ret = XYCont { x: 0, y: 0 };
    for i in 0..v1.len() {
        ret = ret + (v1[i] * v2[i]);
    }
    ret
}

pub fn neighbors<'a, T: Clone>(
    p: Pos,
    m: &'a Hexgrid<T>,
) -> impl Iterator<Item = Option<(Pos, T)>> + 'a {
    within(p, m, 1)
}

pub fn set<T>(Pos { x, y }: Pos, new_cell: T, m: &mut Hexgrid<T>) {
    m[x][y] = new_cell;
}

pub fn get<T: Clone>(Pos { x, y }: Pos, m: &Hexgrid<T>) -> T {
    m[x][y].clone()
}

pub fn distance<C: TryInto<i32>>(from: XYCont<C>, to: XYCont<C>) -> i32
where
    <C as TryInto<i32>>::Error: std::fmt::Debug,
{
    let from_q = xy_to_cube(from);
    let to_q = xy_to_cube(to);
    cube_distance(from_q, to_q) / 2
}

fn cube_distance(from: XYZCont<i32>, to: XYZCont<i32>) -> i32 {
    (from.x - to.x).abs() + (from.y - to.y).abs() + (from.z - to.z).abs()
}

fn xy_to_cube<C: TryInto<i32>>(
    XYCont {
        x: from_x,
        y: from_y,
    }: XYCont<C>,
) -> XYZCont<i32>
where
    <C as TryInto<i32>>::Error: std::fmt::Debug,
{
    let x = from_x.try_into().unwrap();
    let y = from_y.try_into().unwrap();
    let q = x;
    let r = y - (x - ((x.abs() + 1) % 2)) / 2;
    XYZCont {
        x: q,
        y: r,
        z: (-q - r),
    }
}
