use std::collections::HashMap;

use enum_iterator::Sequence;

use crate::celldata::CellState;
use crate::celldata::CellStateData;
use crate::celldata::CellStateVariant;
use crate::make_world;

pub(crate) type ResourceValue = i32;
pub(crate) type ResourceStockpile = ResourceContainer<ResourceData>;
pub(crate) type ResourcePacket = ResourceContainer<ResourceValue>;
pub(crate) type PacketMap = HashMap<ResourceType, i32>;
pub(crate) type ResourceContainer<T> = [T; ResourceType::CARDINALITY as usize];

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Sequence)]
pub(crate) enum ResourceType {
    LogisticsPoints = 0,
    Wood,
    Builders,
    BuildTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ResourceData {
    pub(crate) current: ResourceValue,
    pub(crate) max: ResourceValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Resource {
    Pure(ResourceStockpile),
    WithVariant(ResourceStockpile, CellStateVariant),
}

pub(crate) fn new_hub() -> CellState {
    let cv = CellStateVariant::Hub;
    let mut r: ResourceStockpile = empty_stockpile(cv);
    r = set_to_full(ResourceType::Builders, max(cv, ResourceType::Builders), r);
    r = set_to_empty(ResourceType::Wood, max(cv, ResourceType::Wood), r);
    r = set_to_full(
        ResourceType::LogisticsPoints,
        max(cv, ResourceType::LogisticsPoints),
        r,
    );
    stockpile_to_cell(CellStateVariant::Hub, r)
}

pub(crate) fn new_pure_stockpile(
    cv: CellStateVariant,
    data: HashMap<ResourceType, ResourceValue>,
) -> CellState {
    stockpile_to_cell(cv, map_to_stockpile(cv, data))
}

pub(crate) fn new_stockpile(
    cv: CellStateVariant,
    data: HashMap<ResourceType, ResourceValue>,
    to: CellStateVariant,
) -> CellState {
    stockpile_to_cell_with_extra_variant(cv, map_to_stockpile(cv, data), to)
}

fn map_to_stockpile(
    cv: CellStateVariant,
    data: HashMap<ResourceType, ResourceValue>,
) -> ResourceStockpile {
    let stockpile = data.into_iter().fold(empty_stockpile(cv), |acc, (t, d)| {
        set(
            t,
            ResourceData {
                current: d,
                max: max(cv, t),
            },
            acc,
        )
    });
    stockpile
}

fn stockpile_to_cell_with_extra_variant(
    cv: CellStateVariant,
    s: ResourceStockpile,
    to: CellStateVariant,
) -> CellState {
    CellState {
        variant: cv,
        data: CellStateData::Resource(Resource::WithVariant(s, to)),
    }
}

fn stockpile_to_cell(cv: CellStateVariant, s: ResourceStockpile) -> CellState {
    CellState {
        variant: cv,
        data: CellStateData::Resource(Resource::Pure(s)),
    }
}

pub(crate) fn new_packet(builders: i32, lp: i32) -> ResourcePacket {
    let mut ret = empty_packet();
    ret = set(ResourceType::Builders, builders, ret);
    ret = set(ResourceType::LogisticsPoints, lp, ret);
    ret
}

pub(crate) fn get(t: ResourceType, r: ResourceStockpile) -> i32 {
    r[t as usize].current
}
pub(crate) fn has_capacity(t: ResourceType, r: ResourceStockpile, min_capacity: i32) -> bool {
    let new_value = r[t as usize].current + min_capacity;
    new_value <= r[t as usize].max && new_value >= 0
}

pub(crate) fn has_resources(req: ResourcePacket, r: ResourceStockpile) -> bool {
    all_resourcetypes().all(|i| req[i as usize] <= r[i as usize].current)
}

pub(crate) fn add_packet_to_packet(mut p1: ResourcePacket, p2: ResourcePacket) -> ResourcePacket {
    for index in all_resourcetypes() {
        let i = index as usize;
        p1[i] = p1[i] + p2[i]
    }
    p1
}

pub(crate) fn neg_packet(mut p1: ResourcePacket) -> ResourcePacket {
    for index in all_resourcetypes() {
        let i = index as usize;
        p1[i] = p1[i] * -1;
    }
    p1
}

pub(crate) fn add_to_packet(t: ResourceType, to_add: i32, mut p: ResourcePacket) -> ResourcePacket {
    p[t as usize] = p[t as usize] + to_add;
    p
}

pub(crate) fn add_packet(p: ResourcePacket, c: CellState) -> Option<CellState> {
    match c {
        CellState {
            variant: cv,
            data: CellStateData::Resource(Resource::Pure(resources)),
        } => {
            if let Some(s) = add_packet_stockpile(p, resources) {
                Some(stockpile_to_cell(cv, s))
            } else {
                None
            }
        }
        CellState {
            variant: cv,
            data: CellStateData::Resource(Resource::WithVariant(resources, cv2)),
        } => {
            if let Some(s) = add_packet_stockpile(p, resources) {
                Some(stockpile_to_cell_with_extra_variant(cv, s, cv2))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn add(t: ResourceType, c: CellState, to_add: i32) -> Option<CellState> {
    let mut p = empty_packet();
    p = set(t, to_add, p);
    add_packet(p, c)
}

pub(crate) fn to_key_value_display_amounts(
    cv: CellStateVariant,
    r: ResourceStockpile,
) -> PacketMap {
    let mut ret = HashMap::new();
    for i in all_resourcetypes() {
        let value0 = get(i, r);
        let value = scale(cv, i, value0);
        if value > 0 {
            ret.insert(i, value);
        }
    }
    ret
}

pub(crate) fn to_key_value(r: ResourceStockpile) -> PacketMap {
    let mut ret = HashMap::new();
    for i in all_resourcetypes() {
        let value = get(i, r);
        if value > 0 {
            ret.insert(i, value);
        }
    }
    ret
}

fn scale(cv: CellStateVariant, _t: ResourceType, value: i32) -> i32 {
    let factor = match cv {
        CellStateVariant::Hidden => make_world::SCALING_WOOD,
        _ => 1,
    };
    value / factor
}

pub(crate) fn from_key_value(map: PacketMap) -> ResourcePacket {
    let mut ret = empty_packet();
    for (t, v) in map {
        ret = set(t, v, ret)
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

fn max(cv: CellStateVariant, t: ResourceType) -> i32 {
    match (cv, t) {
        (CellStateVariant::Hub, ResourceType::LogisticsPoints) => 18,
        (CellStateVariant::Hub, ResourceType::Wood) => 50,
        (CellStateVariant::Hub, ResourceType::Builders) => 3,
        (CellStateVariant::Building, ResourceType::BuildTime) => 10,
        (CellStateVariant::Building, ResourceType::Builders) => 2,
        (CellStateVariant::Hidden, ResourceType::Wood) => make_world::MAX_WOOD_RANGE,
        (CellStateVariant::Unused, ResourceType::Wood) => make_world::MAX_WOOD_RANGE,
        _ => 0,
    }
}

fn all_resourcetypes() -> impl Iterator<Item = ResourceType> {
    enum_iterator::all::<ResourceType>()
}

pub(crate) fn empty_packet() -> ResourcePacket {
    let nothing = 0;
    [nothing; ResourceType::CARDINALITY as usize]
}

pub(crate) fn empty_stockpile(cv: CellStateVariant) -> ResourceStockpile {
    let nothing = ResourceData { current: 0, max: 0 };
    let mut ret = [nothing; ResourceType::CARDINALITY as usize];
    for i in all_resourcetypes() {
        ret[i as usize].max = max(cv, i);
    }
    ret
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
