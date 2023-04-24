use std::collections::HashMap;

use crate::celldata;
use crate::celldata::CellState;
use crate::celldata::CellStateData;
use crate::celldata::CellStateVariant;

type ResourceValue = i32;
pub type ResourceStockpile = ResourceContainer<ResourceData>;
pub type ResourcePacket = ResourceContainer<ResourceValue>;
pub type ResourceContainer<T> = [T; ResouceType::Last as usize];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResouceType {
    LogisticsPoints,
    Wood,
    Builders,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceData {
    current: ResourceValue,
    max: ResourceValue,
}

pub fn new_stockpile(cv: CellStateVariant, builders: i32, lp: i32) -> CellState {
    let mut r: ResourceStockpile = empty_stockpile();
    r = set_to_full(ResouceType::Builders, builders, r);
    r = set_to_full(ResouceType::LogisticsPoints, lp, r);
    CellState {
        variant: cv,
        data: CellStateData::Resource { resources: r },
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
    all_resourcetypes().all(|i| req[i] <= r[i].current)
}

pub fn add(t: ResouceType, mut r: ResourceStockpile, to_add: i32) -> Option<ResourceStockpile> {
    if has_capacity(t, r, to_add) {
        r[t as usize].current = r[t as usize].current + to_add;
        Some(r)
    } else {
        None
    }
}

fn all_resourcetypes() -> std::ops::Range<usize> {
    0..ResouceType::Last as usize
}

fn empty_packet() -> ResourcePacket {
    let nothing = 0;
    [nothing; ResouceType::Last as usize]
}

fn empty_stockpile() -> ResourceStockpile {
    let nothing = ResourceData { current: 0, max: 0 };
    [nothing; ResouceType::Last as usize]
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
