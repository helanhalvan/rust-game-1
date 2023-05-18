use std::{
    collections::{HashMap, HashSet},
    ops::{Add, Mul},
};

use itertools::Itertools;

use crate::{
    celldata,
    make_world::{self},
};
use std::hash::Hash;

pub(crate) mod matrix;

pub(crate) const CHUNK_SIZE: usize = 0x100;
const INDEX_MASK: i32 = CHUNK_SIZE as i32 - 1;
const CHUNK_MASK: i32 = !(0 ^ INDEX_MASK);

#[derive(Debug, Clone)]
pub(crate) struct Hexgrid<T: CellGen<GenContext = C>, C: Clone> {
    chunks: HashMap<XYCont<i32>, Chunk<T>>,
    gen_context: C,
    out_of_bounds: T,
}

pub(crate) trait CellGen {
    type GenContext;
    fn new_chunk(p: Pos, c: &mut Self::GenContext) -> Chunk<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub(crate) enum EmptyContext {
    None,
}

//Could be array if generalized array initalization was easy
pub(crate) type Chunk<T> = matrix::Matrix<T>;

pub(crate) type Board = Hexgrid<celldata::CellState, make_world::GenContext>;
pub(crate) type Pos = XYCont<i32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct XYCont<C> {
    pub(crate) x: C,
    pub(crate) y: C,
}

#[derive(Debug, Clone)]
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

pub(crate) fn new<T: Clone + CellGen<GenContext = C>, C: Clone>(
    gen_context: C,
    out_of_bounds: T,
) -> Hexgrid<T, C> {
    Hexgrid {
        chunks: HashMap::new(),
        gen_context: gen_context,
        out_of_bounds,
    }
}

pub(crate) fn chunk_from_example<T: Clone + CellGen<GenContext = C>, C: Clone>(
    example: T,
) -> Chunk<T> {
    let mut ret = vec![];
    for _ in 0..CHUNK_SIZE {
        let mut row = vec![];
        for _ in 0..CHUNK_SIZE {
            row.push(example.clone())
        }
        ret.push(row)
    }
    matrix::new(CHUNK_SIZE, CHUNK_SIZE, ret)
}

pub(crate) fn touch_all_chunks<
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    source: &mut Hexgrid<T, C>,
    XYCont { x: x0, y: y0 }: XYCont<i32>,
    height_extra: i32,
    width_extra: i32,
) {
    for dx in 0..(height_extra + 1) {
        for dy in 0..(width_extra + 1) {
            let c = XYCont {
                x: x0 + dx,
                y: y0 + dy,
            };
            if is_chunk_corner(c) {
                get(c, source);
            }
        }
    }
}

fn is_chunk_corner(XYCont { x: x0, y: y0 }: XYCont<i32>) -> bool {
    let x = x0 & INDEX_MASK;
    let y = y0 & INDEX_MASK;
    ((x == 0) || (x == INDEX_MASK)) && (y == 0 || (y == INDEX_MASK))
}

pub(crate) struct PortIterator<'a, T: CellGen<GenContext = C>, C: Clone> {
    x: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
    source: &'a Hexgrid<T, C>,
}

pub(crate) struct RowIterator<'a, T: CellGen<GenContext = C>, C: Clone> {
    x: i32,
    y: i32,
    y_max: i32,
    source: &'a Hexgrid<T, C>,
}

impl<'a, T: CellGen<GenContext = C>, C: Clone> Iterator for PortIterator<'a, T, C> {
    type Item = (i32, RowIterator<'a, T, C>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.x_max {
            let ret = Some((
                self.x,
                RowIterator {
                    x: self.x,
                    y_max: self.y_max,
                    y: self.y_min,
                    source: self.source,
                },
            ));
            self.x = self.x + 1;
            ret
        } else {
            None
        }
    }
}

impl<T: CellGen<GenContext = C> + Clone + std::cmp::PartialEq + std::fmt::Debug, C: Clone> Iterator
    for RowIterator<'_, T, C>
{
    type Item = (XYCont<i32>, T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y < self.y_max {
            let p = XYCont {
                x: self.x,
                y: self.y,
            };
            let ret = Some((p, unsafe_get(p, self.source)));
            self.y = self.y + 1;
            ret
        } else {
            None
        }
    }
}

pub(crate) fn view_port<
    'a,
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug + Sized,
    C: Clone,
>(
    source: &Hexgrid<T, C>,
    XYCont { x, y }: XYCont<i32>,
    height_extra: i32,
    width_extra: i32,
) -> PortIterator<T, C> {
    let p = PortIterator {
        x,
        y_min: y,
        x_max: x + height_extra,
        y_max: y + width_extra,
        source,
    };
    p
}

pub(crate) fn pos_iter_to_cells<
    'a,
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    pos: impl IntoIterator<Item = Pos> + 'a,
    m: &'a mut Hexgrid<T, C>,
) -> impl Iterator<Item = (Pos, T)> + 'a {
    let ret = pos.into_iter().map(|p| (p, get(p, m)));
    return ret;
}

pub(crate) fn get_connected<
    T: Clone
        + std::cmp::Eq
        + std::hash::Hash
        + CellGen<GenContext = C>
        + std::cmp::PartialEq
        + std::fmt::Debug,
    C: Clone,
>(
    p: Pos,
    t: fn(T) -> bool,
    m: &mut Hexgrid<T, C>,
) -> impl IntoIterator<Item = (Pos, T)> {
    let mut set_size = 0;
    let mut connected: HashSet<(Pos, _)> = neighbors(p, m).filter(|(_, i)| t(i.clone())).collect();
    while connected.len() > set_size {
        set_size = connected.len();
        let mut new_connected = HashSet::new();
        for (i, _) in connected.clone() {
            let batch = neighbors(i, m).filter(|(_, i)| t(i.clone())).collect();
            new_connected = new_connected.union(&batch).map(|i| i.clone()).collect();
        }
        connected = connected.union(&new_connected).map(|i| i.clone()).collect();
    }
    return connected;
}

pub(crate) fn within<
    'a,
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    Pos { x: x0, y: y0 }: Pos,
    m: &'a mut Hexgrid<T, C>,
    range: i32,
) -> impl Iterator<Item = (Pos, T)> + 'a {
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

pub(crate) fn neighbors<
    'a,
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    p: Pos,
    m: &'a mut Hexgrid<T, C>,
) -> impl Iterator<Item = (Pos, T)> + 'a {
    within(p, m, 1)
}

pub(crate) fn set<
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    p: Pos,
    new_cell: T,
    m: &mut Hexgrid<T, C>,
) {
    let (chunk_key, in_chunk_key) = to_chunk_keys(p);
    let mut chunk = if let Some(chunk) = m.chunks.get(&chunk_key) {
        chunk.clone()
    } else {
        T::new_chunk(chunk_key, &mut m.gen_context)
    };
    matrix::set(&mut chunk, new_cell, in_chunk_key);
    m.chunks.insert(chunk_key, chunk);
}

pub(crate) fn get<
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    p: Pos,
    m: &mut Hexgrid<T, C>,
) -> T {
    let (chunk_key, in_chunk_key) = to_chunk_keys(p);
    let chunk = if let Some(c) = m.chunks.get(&chunk_key) {
        c.clone()
    } else {
        let chunk = T::new_chunk(chunk_key, &mut m.gen_context);
        m.chunks.insert(chunk_key, chunk.clone());
        chunk
    };
    matrix::get(&chunk, in_chunk_key).unwrap().clone()
}

pub(crate) fn unsafe_get<
    T: Clone + CellGen<GenContext = C> + std::cmp::PartialEq + std::fmt::Debug,
    C: Clone,
>(
    p: Pos,
    m: &Hexgrid<T, C>,
) -> T {
    let (chunk_key, in_chunk_key) = to_chunk_keys(p);
    if let Some(chunk) = m.chunks.get(&chunk_key) {
        matrix::get(&chunk, in_chunk_key).unwrap().clone()
    } else {
        m.out_of_bounds.clone()
    }
}

pub(crate) fn to_chunk_keys(Pos { x, y }: Pos) -> (XYCont<i32>, XYCont<usize>) {
    let chunk_key = XYCont {
        x: x & CHUNK_MASK,
        y: y & CHUNK_MASK,
    };
    let in_chunk_key = XYCont {
        x: (x & INDEX_MASK) as usize,
        y: (y & INDEX_MASK) as usize,
    };
    (chunk_key, in_chunk_key)
}

// https://www.redblobgames.com/grids/hexagons/
pub(crate) fn distance<C: TryInto<i32>>(from: XYCont<C>, to: XYCont<C>) -> i32
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
    let r = y - (x + (x & 1)) / 2;
    XYZCont {
        x: q,
        y: r,
        z: (-q - r),
    }
}
