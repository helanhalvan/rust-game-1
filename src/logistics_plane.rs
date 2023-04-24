use std::collections::{HashMap, HashSet};

use crate::{
    celldata::{self, CellState, CellStateData, CellStateVariant},
    hexgrid::{self, Pos},
    GameState,
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
        //borrows: HashMap<Pos, Borrow>,
    },
}

//type Borrow = HashMap<ResouceType, ResourceData>;

pub fn new_plane(xmax: usize, ymax: usize) -> LogisticsPlane {
    vec![vec![LogisticsState::None; xmax]; ymax]
}

pub fn has_worker(pos: hexgrid::Pos, g: &GameState) -> bool {
    connected_sources(pos, g)
        .into_iter()
        .any(|i| can_use(pos, i, hexgrid::get(i, &g.matrix)))
}

pub fn use_builder(pos: Pos, mut g: GameState) -> GameState {
    g.matrix = find_logistcs_node(
        pos,
        pos,
        can_use,
        |_user, _target, i| match i {
            CellState {
                variant,
                data:
                    CellStateData::Resource2x {
                        left: [workers, lp],
                        total,
                    },
            } => CellState {
                variant,
                data: CellStateData::Resource2x {
                    left: [workers - 1, lp],
                    total,
                },
            },
            _ => unimplemented!(),
        },
        g.matrix,
        &g.logistics_plane,
    )
    .unwrap();
    g
}

fn can_use(user: Pos, target: Pos, c: CellState) -> bool {
    match c.data {
        CellStateData::Resource2x {
            left: [workers, lp],
            ..
        } => (workers > 0) && (lp >= hexgrid::distance(user, target)),
        _ => false,
    }
}

// its not like we check that we return it to the right place
pub fn return_builder(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.matrix = find_logistcs_node(
        pos,
        pos,
        |_, _, i| match i {
            CellState {
                variant: celldata::CellStateVariant::Hub,
                data:
                    CellStateData::Resource2x {
                        left: [workers, _],
                        total: [max_workers, _],
                    },
            } => workers < max_workers,
            _ => false,
        },
        |_, _, i| match i {
            CellState {
                variant,
                data:
                    CellStateData::Resource2x {
                        left: [workers, lp],
                        total,
                    },
            } => CellState {
                variant,
                data: CellStateData::Resource2x {
                    left: [workers + 1, lp],
                    total,
                },
            },
            _ => unimplemented!(),
        },
        g.matrix,
        &g.logistics_plane,
    )
    .unwrap();
    g
}

fn find_logistcs_node(
    src: Pos,
    pos: hexgrid::Pos,
    cond: fn(Pos, Pos, CellState) -> bool,
    update: fn(Pos, Pos, CellState) -> CellState,
    mut b: hexgrid::Board,
    b2: &LogisticsPlane,
) -> Option<hexgrid::Board> {
    let cs = hexgrid::get(pos, &b);
    let ls = hexgrid::get(pos, &b2);
    if cond(src, pos, cs) {
        let c1 = update(src, pos, cs);
        hexgrid::set(pos, c1, &mut b);
        Some(b)
    } else {
        match ls {
            LogisticsState::Available { locations } => {
                for i in locations {
                    match find_logistcs_node(src, i, cond, update, b.clone(), b2) {
                        a @ Some(..) => {
                            return a;
                        }
                        None => {}
                    }
                }
                None
            }
            _ => {
                dbg!((ls, cs));
                None
            }
        }
    }
}

fn connected_sources(pos: hexgrid::Pos, g: &GameState) -> Vec<hexgrid::Pos> {
    match hexgrid::get(pos, &g.logistics_plane) {
        LogisticsState::None => vec![],
        LogisticsState::Available { locations } => {
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
                        };
                        hexgrid::set(pn, new_cell, &mut acc);
                        acc
                    }
                    LogisticsState::Source { .. } => acc,
                    LogisticsState::Available { mut locations } => {
                        locations = locations.union(&new_subset).cloned().collect();
                        let new_cell = LogisticsState::Available { locations };
                        hexgrid::set(pn, new_cell, &mut acc);
                        acc
                    }
                })
        })
    }
}
