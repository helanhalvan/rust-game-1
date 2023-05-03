use std::{
    collections::{HashMap, HashSet},
    ops::{Add, Mul},
};

use itertools::Itertools;

use crate::{celldata, make_world};
use std::hash::Hash;

pub const CHUNK_SIZE: usize = 0x10;
const INDEX_MASK: i32 = 0xF;
const CHUNK_MASK: i32 = !(0 ^ INDEX_MASK);

#[derive(Debug, Clone)]
pub struct Hexgrid<T: CellGen<GenContext = C>, C: Clone> {
    chunks: HashMap<XYCont<i32>, Chunk<T>>,
    gen_context: C,
    out_of_bounds: T,
}

pub trait CellGen {
    type GenContext;
    fn new_chunk(p: Pos, c: &mut Self::GenContext) -> Chunk<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub enum EmptyContext {
    None,
}

pub type Matrix<T> = Vec<Vec<T>>;

//Could be array if generalized array initalization was easy
type Chunk<T> = Matrix<T>;

pub type Board = Hexgrid<celldata::CellState, make_world::GenContext>;
pub type Pos = XYCont<i32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct XYCont<C> {
    pub x: C,
    pub y: C,
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

pub fn new<T: Clone + CellGen<GenContext = C>, C: Clone>(
    gen_context: C,
    out_of_bounds: T,
) -> Hexgrid<T, C> {
    Hexgrid {
        chunks: HashMap::new(),
        gen_context: gen_context,
        out_of_bounds,
    }
}

pub fn chunk_from_example<T: Clone + CellGen<GenContext = C>, C: Clone>(example: T) -> Chunk<T> {
    let mut ret = vec![];
    for _ in 0..CHUNK_SIZE {
        let mut row = vec![];
        for _ in 0..CHUNK_SIZE {
            row.push(example.clone())
        }
        ret.push(row)
    }
    ret
}

pub fn touch_all_chunks<T: Clone + CellGen<GenContext = C>, C: Clone>(
    source: &mut Hexgrid<T, C>,
    XYCont { x, y }: XYCont<i32>,
    height_extra: i32,
    width_extra: i32,
) -> Matrix<T> {
    let mut ret = vec![];
    for dx in 0..(height_extra + 1) {
        let mut y_buff = vec![];
        for dy in 0..(width_extra + 1) {
            let new = get(
                XYCont {
                    x: x + dx,
                    y: y + dy,
                },
                source,
            );
            y_buff.push(new);
        }
        ret.push(y_buff)
    }
    ret
}

pub fn view_port<T: Clone + CellGen<GenContext = C>, C: Clone>(
    source: &Hexgrid<T, C>,
    XYCont { x, y }: XYCont<i32>,
    height_extra: i32,
    width_extra: i32,
) -> Matrix<T> {
    let mut ret = vec![];
    for dx in 0..(height_extra + 1) {
        let mut y_buff = vec![];
        for dy in 0..(width_extra + 1) {
            let new = unsafe_get(
                XYCont {
                    x: x + dx,
                    y: y + dy,
                },
                source,
            );
            y_buff.push(new);
        }
        ret.push(y_buff)
    }
    ret
}

pub fn pos_iter_to_cells<'a, T: Clone + CellGen<GenContext = C>, C: Clone>(
    pos: impl IntoIterator<Item = Pos> + 'a,
    m: &'a mut Hexgrid<T, C>,
) -> impl Iterator<Item = (Pos, T)> + 'a {
    let ret = pos.into_iter().map(|p| (p, get(p, m)));
    return ret;
}

pub fn get_connected<
    T: Clone + std::cmp::Eq + std::hash::Hash + CellGen<GenContext = C>,
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

pub fn within<'a, T: Clone + CellGen<GenContext = C>, C: Clone>(
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

pub fn neighbors<'a, T: Clone + CellGen<GenContext = C>, C: Clone>(
    p: Pos,
    m: &'a mut Hexgrid<T, C>,
) -> impl Iterator<Item = (Pos, T)> + 'a {
    within(p, m, 1)
}

pub fn set<T: Clone + CellGen<GenContext = C>, C: Clone>(
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
    chunk[in_chunk_key.x][in_chunk_key.y] = new_cell;
    m.chunks.insert(chunk_key, chunk);
}

pub fn get<T: Clone + CellGen<GenContext = C>, C: Clone>(p: Pos, m: &mut Hexgrid<T, C>) -> T {
    let (chunk_key, in_chunk_key) = to_chunk_keys(p);
    let chunk = if let Some(c) = m.chunks.get(&chunk_key) {
        c.clone()
    } else {
        let chunk = T::new_chunk(chunk_key, &mut m.gen_context);
        m.chunks.insert(chunk_key, chunk.clone());
        chunk
    };
    chunk[in_chunk_key.x][in_chunk_key.y].clone()
}

// this version of get does not persist values pulled out of Hexgrid
// so repeated get-calls might result in different returned values
// when pulling data from un-initalized chunks
pub fn unsafe_get<T: Clone + CellGen<GenContext = C>, C: Clone>(p: Pos, m: &Hexgrid<T, C>) -> T {
    let (chunk_key, in_chunk_key) = to_chunk_keys(p);
    if let Some(chunk) = m.chunks.get(&chunk_key) {
        chunk[in_chunk_key.x][in_chunk_key.y].clone()
    } else {
        m.out_of_bounds.clone()
    }
}

pub fn to_chunk_keys(Pos { x, y }: Pos) -> (XYCont<i32>, XYCont<usize>) {
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
    let r = y - (x + (x & 1)) / 2;
    XYZCont {
        x: q,
        y: r,
        z: (-q - r),
    }
}
