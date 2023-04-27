use std::collections::HashMap;

use enum_iterator::Sequence;

use crate::celldata;
use crate::celldata::CellState;
use crate::celldata::CellStateData;
use crate::celldata::CellStateVariant;

type ResourceValue = i32;
pub type ResourceStockpile = ResourceContainer<ResourceData>;
pub type ResourcePacket = ResourceContainer<ResourceValue>;
pub type ResourceContainer<T> = [T; ResouceType::CARDINALITY as usize];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
pub enum ResouceType {
    LogisticsPoints,
    Wood,
    Builders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceData {
    current: ResourceValue,
    max: ResourceValue,
}

pub fn new_hub() -> CellState {
    new_stockpile(max_builders(), max_lp())
}

fn new_stockpile(b: i32, lp: i32) -> CellState {
    let mut r: ResourceStockpile = empty_stockpile();
    r = set_to_full(ResouceType::Builders, b, r);
    r = set_to_full(ResouceType::LogisticsPoints, lp, r);
    stockpile_to_cell(r)
}

fn stockpile_to_cell(s: ResourceStockpile) -> CellState {
    CellState {
        variant: CellStateVariant::Hub,
        data: CellStateData::Resource { resources: s },
    }
}

pub fn new_packet(builders: i32, lp: i32) -> ResourcePacket {
    let mut ret = empty_packet();
    ret = set(ResouceType::Builders, builders, ret);
    ret = set(ResouceType::LogisticsPoints, lp, ret);
    ret
}

pub fn get(t: ResouceType, r: ResourceStockpile) -> i32 {
    r[t as usize].current
}
pub fn has_capacity(t: ResouceType, r: ResourceStockpile, min_capacity: i32) -> bool {
    let new_value = r[t as usize].current + min_capacity;
    new_value <= r[t as usize].max && new_value >= 0
}

pub fn has_resources(req: ResourcePacket, r: ResourceStockpile) -> bool {
    all_resourcetypes().all(|i| req[i as usize] <= r[i as usize].current)
}

pub fn add_packet(p: ResourcePacket, c: CellState) -> Option<CellState> {
    match c {
        CellState {
            variant: CellStateVariant::Hub,
            data: CellStateData::Resource { resources },
        } => {
            if let Some(s) = add_packet_stockpile(p, resources) {
                Some(stockpile_to_cell(s))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn add(t: ResouceType, c: CellState, to_add: i32) -> Option<CellState> {
    let mut p = empty_packet();
    p = set(t, to_add, p);
    add_packet(p, c)
}

pub fn to_key_value(r: ResourceStockpile) -> HashMap<ResouceType, i32> {
    let mut ret = HashMap::new();
    for i in all_resourcetypes() {
        let value = get(i, r);
        if value > 0 {
            ret.insert(i, value);
        }
    }
    ret
}

fn add_packet_stockpile(p: ResourcePacket, mut s: ResourceStockpile) -> Option<ResourceStockpile> {
    for i in all_resourcetypes() {
        let index = i as usize;
        let delta = p[index];
        if delta == 0 {
            continue;
        }
        if has_capacity(i, s, delta) {
            s[index].current = s[index].current + delta;
        } else {
            return None;
        }
    }
    return Some(s);
}

fn max_builders() -> i32 {
    3
}

fn max_lp() -> i32 {
    9
}

pub fn statespace() -> celldata::Statespace {
    let mut ret = vec![];
    let mut s0 = new_stockpile(max_builders(), max_lp());
    ret.push(s0);
    for _ in 0..max_builders() {
        s0 = add(ResouceType::Builders, s0, -1).unwrap();
        for _ in 0..max_lp() {
            s0 = add(ResouceType::LogisticsPoints, s0, -1).unwrap();
            ret.push(s0);
        }
        s0 = add(ResouceType::LogisticsPoints, s0, max_lp()).unwrap();
    }
    ret
}

fn all_resourcetypes() -> impl Iterator<Item = ResouceType> {
    enum_iterator::all::<ResouceType>()
}

fn empty_packet() -> ResourcePacket {
    let nothing = 0;
    [nothing; ResouceType::CARDINALITY as usize]
}

fn empty_stockpile() -> ResourceStockpile {
    let nothing = ResourceData { current: 0, max: 0 };
    [nothing; ResouceType::CARDINALITY as usize]
}

fn set_to_full(t: ResouceType, new: i32, r: ResourceStockpile) -> ResourceStockpile {
    let new = ResourceData {
        current: new,
        max: new,
    };
    set(t, new, r)
}

fn set<I>(t: ResouceType, new: I, mut r: ResourceContainer<I>) -> ResourceContainer<I> {
    r[t as usize] = new;
    r
}
