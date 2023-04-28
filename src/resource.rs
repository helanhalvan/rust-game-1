use std::collections::HashMap;

use enum_iterator::Sequence;

use crate::celldata;
use crate::celldata::CellState;
use crate::celldata::CellStateData;
use crate::celldata::CellStateVariant;
use itertools::Itertools;

type ResourceValue = i32;
pub type ResourceStockpile = ResourceContainer<ResourceData>;
pub type ResourcePacket = ResourceContainer<ResourceValue>;
pub type ResourceContainer<T> = [T; ResourceType::CARDINALITY as usize];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
pub enum ResourceType {
    LogisticsPoints = 0,
    Wood,
    Builders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceData {
    current: ResourceValue,
    max: ResourceValue,
}

pub fn new_hub() -> CellState {
    let mut r: ResourceStockpile = empty_stockpile();
    r = set_to_full(ResourceType::Builders, max(ResourceType::Builders), r);
    r = set_to_empty(ResourceType::Wood, max(ResourceType::Wood), r);
    r = set_to_full(
        ResourceType::LogisticsPoints,
        max(ResourceType::LogisticsPoints),
        r,
    );
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
    ret = set(ResourceType::Builders, builders, ret);
    ret = set(ResourceType::LogisticsPoints, lp, ret);
    ret
}

pub fn get(t: ResourceType, r: ResourceStockpile) -> i32 {
    r[t as usize].current
}
pub fn has_capacity(t: ResourceType, r: ResourceStockpile, min_capacity: i32) -> bool {
    let new_value = r[t as usize].current + min_capacity;
    new_value <= r[t as usize].max && new_value >= 0
}

pub fn has_resources(req: ResourcePacket, r: ResourceStockpile) -> bool {
    all_resourcetypes().all(|i| req[i as usize] <= r[i as usize].current)
}

pub fn add_packet_to_packet(mut p1: ResourcePacket, p2: ResourcePacket) -> ResourcePacket {
    for index in all_resourcetypes() {
        let i = index as usize;
        p1[i] = p1[i] + p2[i]
    }
    p1
}

pub fn neg_packet(mut p1: ResourcePacket) -> ResourcePacket {
    for index in all_resourcetypes() {
        let i = index as usize;
        p1[i] = p1[i] * -1;
    }
    p1
}

pub fn add_to_packet(t: ResourceType, to_add: i32, mut p: ResourcePacket) -> ResourcePacket {
    p[t as usize] = p[t as usize] + to_add;
    p
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

pub fn add(t: ResourceType, c: CellState, to_add: i32) -> Option<CellState> {
    let mut p = empty_packet();
    p = set(t, to_add, p);
    add_packet(p, c)
}

pub fn to_key_value(r: ResourceStockpile) -> HashMap<ResourceType, i32> {
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

fn max(t: ResourceType) -> i32 {
    match t {
        ResourceType::LogisticsPoints => 9,
        ResourceType::Wood => 100,
        ResourceType::Builders => 3,
    }
}

pub fn statespace() -> celldata::Statespace {
    let mut ret = vec![];
    let resource_space: Vec<_> = all_resourcetypes()
        .map(|i| 0..(max(i) + 1))
        .multi_cartesian_product()
        .collect();
    for s in resource_space {
        let mut s0 = empty_stockpile();
        for t in all_resourcetypes() {
            s0 = set(
                t,
                ResourceData {
                    max: max(t),
                    current: s[t as usize],
                },
                s0,
            );
        }
        ret.push(stockpile_to_cell(s0));
    }
    dbg!(ret.clone());
    ret
}

fn all_resourcetypes() -> impl Iterator<Item = ResourceType> {
    enum_iterator::all::<ResourceType>()
}

pub fn empty_packet() -> ResourcePacket {
    let nothing = 0;
    [nothing; ResourceType::CARDINALITY as usize]
}

fn empty_stockpile() -> ResourceStockpile {
    let nothing = ResourceData { current: 0, max: 0 };
    [nothing; ResourceType::CARDINALITY as usize]
}

fn set_to_empty(t: ResourceType, new: i32, r: ResourceStockpile) -> ResourceStockpile {
    let new = ResourceData {
        current: 0,
        max: new,
    };
    set(t, new, r)
}

fn set_to_full(t: ResourceType, new: i32, r: ResourceStockpile) -> ResourceStockpile {
    let new = ResourceData {
        current: new,
        max: new,
    };
    set(t, new, r)
}

fn set<I>(t: ResourceType, new: I, mut r: ResourceContainer<I>) -> ResourceContainer<I> {
    r[t as usize] = new;
    r
}
