use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::{
    celldata::{CellState, CellStateData, CellStateVariant},
    hexgrid::{self, Pos},
    resource, GameState,
};

// mirror of the main board (hexgrid::Board) in size
// for use of the building subsytem
// need to keep "available logistics" somewhere
pub type LogisticsPlane = hexgrid::Hexgrid<LogisticsState, hexgrid::EmptyContext>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Available {
    pub locations: HashSet<hexgrid::Pos>,
    pub borrows: HashMap<Pos, resource::ResourcePacket>,
    pub taken_lp: HashMap<Pos, resource::ResourceValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogisticsState {
    None,
    Source,
    Available(Available),
}

impl hexgrid::CellGen for LogisticsState {
    type GenContext = hexgrid::EmptyContext;

    fn new_chunk(_p: Pos, _c: &mut Self::GenContext) -> hexgrid::Matrix<Self> {
        hexgrid::chunk_from_example(LogisticsState::None)
    }
}

pub fn new_plane() -> LogisticsPlane {
    hexgrid::new(hexgrid::EmptyContext::None, LogisticsState::None)
}

pub fn has_worker(pos: hexgrid::Pos, g: &GameState) -> bool {
    connected_sources(pos, g)
        .into_iter()
        .any(|i| can_use(pos, i, hexgrid::unsafe_get(i, &g.matrix)))
}

fn can_use(user: Pos, target: Pos, c: CellState) -> bool {
    match c.data {
        CellStateData::Resource(resource::Resource::Pure(resources)) => {
            let cmp = resource::new_packet(1, hexgrid::distance(user, target));
            resource::has_resources(cmp, resources)
        }
        _ => false,
    }
}

pub fn return_borrows(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    let a = get_available(pos, &mut g);
    g = a.borrows.iter().fold(g, |mut acc: GameState, (p, b)| {
        let c0 = hexgrid::get(*p, &mut acc.matrix);
        let b1 = resource::neg_packet(*b);
        let c1 = resource::add_packet(b1, c0).unwrap();
        hexgrid::set(*p, c1, &mut acc.matrix);
        acc
    });
    hexgrid::set(
        pos,
        LogisticsState::Available(Available {
            borrows: HashMap::new(),
            ..a
        }),
        &mut g.logistics_plane,
    );
    g
}

pub fn return_lp(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    let a = get_available(pos, &mut g);
    g = a.taken_lp.iter().fold(g, |mut acc: GameState, (p, b)| {
        let c0 = hexgrid::get(*p, &mut acc.matrix);
        let c1 = resource::add(resource::ResourceType::LogisticsPoints, c0, *b).unwrap();
        hexgrid::set(*p, c1, &mut acc.matrix);
        acc
    });
    hexgrid::set(
        pos,
        LogisticsState::Available(Available {
            taken_lp: HashMap::new(),
            ..a
        }),
        &mut g.logistics_plane,
    );
    g
}

pub fn try_take_resources(
    src: Pos,
    p: resource::ResourcePacket,
    g: &mut GameState,
) -> Option<GameState> {
    dbg!(p);
    try_resources(src, p, false, g)
}

pub fn try_borrow_resources(
    src: Pos,
    p: resource::ResourcePacket,
    g: &mut GameState,
) -> Option<GameState> {
    try_resources(src, p, true, g)
}

fn get_available(src: Pos, g: &mut GameState) -> Available {
    match hexgrid::get(src, &mut g.logistics_plane) {
        LogisticsState::Available(a) => a,
        a => {
            unimplemented!("{:?}", a)
        }
    }
}

fn try_resources(
    src: Pos,
    mut p: resource::ResourcePacket,
    is_borrow: bool,
    g: &mut GameState,
) -> Option<GameState> {
    p = resource::neg_packet(p);
    let a = get_available(src, g);
    let mut vec: Vec<_> = a
        .locations
        .clone()
        .into_iter()
        .map(|i| (hexgrid::distance(src, i), i))
        .collect();
    vec.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (distance, target) in vec {
        let p1 = resource::add_to_packet(resource::ResourceType::LogisticsPoints, -distance, p);
        let target_cell = hexgrid::get(target, &mut g.matrix);
        dbg!(distance, target, p1);
        if let Some(new) = resource::add_packet(p1, target_cell) {
            hexgrid::set(target, new, &mut g.matrix);
            if is_borrow {
                let lp1 = update_borrows(a, p1, target);
                hexgrid::set(src, lp1, &mut g.logistics_plane);
            } else {
                let lp1 = update_take(a, distance, target);
                hexgrid::set(src, lp1, &mut g.logistics_plane);
            }
            return Some(g.clone());
        }
    }
    None
}

fn update_take(a: Available, p: resource::ResourceValue, target: Pos) -> LogisticsState {
    let taken_lp = insert_or_join(a.taken_lp, |a, b| a + b, target, p);
    LogisticsState::Available(Available { taken_lp, ..a })
}

fn update_borrows(a: Available, p: resource::ResourcePacket, target: Pos) -> LogisticsState {
    let borrows = insert_or_join(
        a.borrows,
        |a, b| resource::add_packet_to_packet(a, b),
        target,
        p,
    );
    LogisticsState::Available(Available { borrows, ..a })
}

fn insert_or_join<K, V: Clone>(
    mut m: HashMap<K, V>,
    join: fn(V, V) -> V,
    k: K,
    v: V,
) -> HashMap<K, V>
where
    K: Hash + std::cmp::Eq + PartialEq,
{
    match m.get(&k) {
        None => {
            m.insert(k, v);
        }
        Some(v2) => {
            let v3 = join(v2.clone(), v);
            m.insert(k, v3);
        }
    }
    m
}

fn connected_sources(pos: hexgrid::Pos, g: &GameState) -> Vec<hexgrid::Pos> {
    match hexgrid::unsafe_get(pos, &g.logistics_plane) {
        LogisticsState::None => vec![],
        LogisticsState::Available(Available { locations, .. }) => locations.into_iter().collect(),
        LogisticsState::Source => {
            vec![pos]
        }
    }
}

pub fn update_logistics(pos: hexgrid::Pos, is_hub: bool, mut g: GameState) -> GameState {
    let other_hubs = find_connected_hubs(pos, &mut g);
    let connected_hubs = if is_hub {
        let mut tv: HashSet<_> = other_hubs.collect();
        tv.insert(pos);
        tv
    } else {
        other_hubs.collect()
    };
    let new_network = {
        let mut other_roads: HashSet<_> = find_connected_roads(pos, &mut g).collect();
        other_roads.insert(pos);
        other_roads
    };
    g.logistics_plane = add_to_close(new_network, connected_hubs, g.logistics_plane);
    g
}

fn find_connected_roads(
    pos: hexgrid::Pos,
    g: &mut GameState,
) -> impl Iterator<Item = hexgrid::Pos> {
    hexgrid::get_connected(
        pos,
        |i| match i.into() {
            CellStateVariant::Road => true,
            _ => false,
        },
        &mut g.matrix,
    )
    .into_iter()
    .map(|(p, _)| p)
}

fn find_connected_hubs(pos: hexgrid::Pos, g: &mut GameState) -> impl Iterator<Item = hexgrid::Pos> {
    hexgrid::get_connected(
        pos,
        |i| match i.into() {
            CellStateVariant::Hub => true,
            CellStateVariant::Road => true,
            _ => false,
        },
        &mut g.matrix,
    )
    .into_iter()
    .filter(|(_p, c)| match (*c).into() {
        CellStateVariant::Hub => true,
        _ => false,
    })
    .map(|(p, _)| p)
}

fn add_to_close(
    src: impl IntoIterator<Item = hexgrid::Pos>,
    to_add: impl IntoIterator<Item = hexgrid::Pos>,
    lp: LogisticsPlane,
) -> LogisticsPlane {
    let new_subset: HashSet<_> = to_add.into_iter().collect();
    if new_subset.is_empty() {
        lp
    } else {
        src.into_iter().fold(lp, |b, src_item| {
            hexgrid::within(src_item, &mut (b.clone()), 2).fold(b, |mut acc, (pn, c)| match c {
                LogisticsState::None => {
                    let new_cell = LogisticsState::Available(Available {
                        locations: new_subset.clone(),
                        borrows: HashMap::new(),
                        taken_lp: HashMap::new(),
                    });
                    hexgrid::set(pn, new_cell, &mut acc);
                    acc
                }
                LogisticsState::Source { .. } => acc,
                LogisticsState::Available(a) => {
                    let locations = new_subset.union(&a.locations).cloned().collect();
                    let new_cell = LogisticsState::Available(Available { locations, ..a });
                    hexgrid::set(pn, new_cell, &mut acc);
                    acc
                }
            })
        })
    }
}
