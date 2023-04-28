use std::collections::{HashMap, HashSet};

use crate::{
    celldata::{self, CellState, CellStateData, CellStateVariant},
    hexgrid::{self, Pos},
    resource, GameState,
};

// mirror of the main board (hexgrid::Board) in size
// for use of the building subsytem
// need to keep "available logistics" somewhere
pub type LogisticsPlane = hexgrid::Hexgrid<LogisticsState>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogisticsState {
    None,
    Source,
    Available {
        locations: HashSet<hexgrid::Pos>,
        borrows: HashMap<Pos, resource::ResourcePacket>,
    },
}

pub fn new_plane(xmax: usize, ymax: usize) -> LogisticsPlane {
    vec![vec![LogisticsState::None; xmax]; ymax]
}

pub fn has_worker(pos: hexgrid::Pos, g: &GameState) -> bool {
    connected_sources(pos, g)
        .into_iter()
        .any(|i| can_use(pos, i, hexgrid::get(i, &g.matrix)))
}

pub fn use_builder(pos: Pos, g: GameState) -> GameState {
    let p = resource::new_packet(-1, 0);
    try_borrow_resources(pos, p, g).unwrap()
}

fn can_use(user: Pos, target: Pos, c: CellState) -> bool {
    match c.data {
        CellStateData::Resource { resources, .. } => {
            let cmp = resource::new_packet(1, hexgrid::distance(user, target));
            resource::has_resources(cmp, resources)
        }
        _ => false,
    }
}

pub fn return_borrows(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    match hexgrid::get(pos, &g.logistics_plane) {
        LogisticsState::Available { borrows, locations } => {
            g = borrows.iter().fold(g, |mut acc: GameState, (p, b)| {
                let c0 = hexgrid::get(*p, &acc.matrix);
                let b1 = resource::neg_packet(*b);
                let c1 = resource::add_packet(b1, c0).unwrap();
                hexgrid::set(*p, c1, &mut acc.matrix);
                acc
            });
            hexgrid::set(
                pos,
                LogisticsState::Available {
                    locations,
                    borrows: HashMap::new(),
                },
                &mut g.logistics_plane,
            );
            g
        }
        a => {
            dbg!(a);
            unimplemented!()
        }
    }
}

fn try_borrow_resources(
    src: Pos,
    p: resource::ResourcePacket,
    mut g: GameState,
) -> Option<GameState> {
    match hexgrid::get(src, &g.logistics_plane) {
        LogisticsState::Available { locations, borrows } => {
            let mut vec: Vec<_> = locations
                .iter()
                .map(|i| (hexgrid::distance(src, *i), i))
                .collect();
            vec.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (distance, target) in vec {
                let p1 =
                    resource::add_to_packet(resource::ResouceType::LogisticsPoints, -distance, p);
                let target_cell = hexgrid::get(*target, &g.matrix);
                if let Some(new) = resource::add_packet(p1, target_cell) {
                    hexgrid::set(*target, new, &mut g.matrix);
                    let lp1 = update_borrows(locations.clone(), borrows, p1, *target);
                    hexgrid::set(src, lp1, &mut g.logistics_plane);
                    return Some(g);
                }
            }
            None
        }
        a => {
            dbg!(a);
            None
        }
    }
}

fn update_borrows(
    locations: HashSet<hexgrid::Pos>,
    mut borrows: HashMap<Pos, resource::ResourcePacket>,
    p: resource::ResourcePacket,
    target: Pos,
) -> LogisticsState {
    match borrows.get(&target) {
        None => {
            borrows.insert(target, p);
        }
        Some(p0) => {
            let p3 = resource::add_packet_to_packet(*p0, p);
            borrows.insert(target, p3);
        }
    }
    LogisticsState::Available { locations, borrows }
}

fn connected_sources(pos: hexgrid::Pos, g: &GameState) -> Vec<hexgrid::Pos> {
    match hexgrid::get(pos, &g.logistics_plane) {
        LogisticsState::None => vec![],
        LogisticsState::Available { locations, .. } => {
            if let Some(ret) = locations
                .into_iter()
                .map(|i| connected_sources(i, g))
                .reduce(|mut acc, mut i| {
                    acc.append(&mut i);
                    acc
                })
            {
                ret
            } else {
                vec![]
            }
        }
        LogisticsState::Source => {
            vec![pos]
        }
    }
}

pub fn update_logistics(pos: hexgrid::Pos, is_hub: bool, mut g: GameState) -> GameState {
    let other_hubs = find_connected_hubs(pos, &g);
    let connected_hubs = if is_hub {
        let mut tv: HashSet<_> = other_hubs.collect();
        tv.insert(pos);
        tv
    } else {
        other_hubs.collect()
    };
    let new_network = {
        let mut other_roads: HashSet<_> = find_connected_roads(pos, &g).collect();
        other_roads.insert(pos);
        other_roads
    };
    g.logistics_plane = add_to_neighbors(new_network, connected_hubs, g.logistics_plane);
    g
}

fn find_connected_roads(pos: hexgrid::Pos, g: &GameState) -> impl Iterator<Item = hexgrid::Pos> {
    hexgrid::get_connected(
        pos,
        |i| match i.into() {
            CellStateVariant::Road => true,
            _ => false,
        },
        &g.matrix,
    )
    .into_iter()
    .map(|(p, _)| p)
}

fn find_connected_hubs(pos: hexgrid::Pos, g: &GameState) -> impl Iterator<Item = hexgrid::Pos> {
    hexgrid::get_connected(
        pos,
        |i| match i.into() {
            CellStateVariant::Hub => true,
            CellStateVariant::Road => true,
            _ => false,
        },
        &g.matrix,
    )
    .into_iter()
    .filter(|(_p, c)| match (*c).into() {
        CellStateVariant::Hub => true,
        _ => false,
    })
    .map(|(p, _)| p)
}

fn add_to_neighbors(
    src: impl IntoIterator<Item = hexgrid::Pos>,
    to_add: impl IntoIterator<Item = hexgrid::Pos>,
    lp: LogisticsPlane,
) -> LogisticsPlane {
    let new_subset: HashSet<_> = to_add.into_iter().collect();
    if new_subset.is_empty() {
        lp
    } else {
        src.into_iter().fold(lp, |b, src_item| {
            hexgrid::neighbors(src_item, &(b.clone()))
                .filter_map(|i| i)
                .fold(b, |mut acc, (pn, c)| match c {
                    LogisticsState::None => {
                        let new_cell = LogisticsState::Available {
                            locations: new_subset.clone(),
                            borrows: HashMap::new(),
                        };
                        hexgrid::set(pn, new_cell, &mut acc);
                        acc
                    }
                    LogisticsState::Source { .. } => acc,
                    LogisticsState::Available {
                        mut locations,
                        borrows,
                    } => {
                        locations = locations.union(&new_subset).cloned().collect();
                        let new_cell = LogisticsState::Available { locations, borrows };
                        hexgrid::set(pn, new_cell, &mut acc);
                        acc
                    }
                })
        })
    }
}
