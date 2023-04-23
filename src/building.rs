use std::collections::HashSet;

use crate::{
    actionmachine,
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
    Available { locations: HashSet<hexgrid::Pos> },
}

pub fn new_plane(xmax: usize, ymax: usize) -> LogisticsPlane {
    vec![vec![LogisticsState::None; xmax]; ymax]
}

pub fn has_actions(
    pos: hexgrid::Pos,
    c: celldata::CellState,
    g: &GameState,
) -> Option<Vec<CellStateVariant>> {
    if has_logistics(pos, g) {
        match c.variant {
            CellStateVariant::Hidden => Some(explore_able()),
            CellStateVariant::Unused => Some(buildable()),
            CellStateVariant::Industry => Some(industry()),
            CellStateVariant::Infrastructure => Some(infrastructure()),
            CellStateVariant::Extract => Some(extract()),
            _ => None,
        }
    } else {
        None
    }
}

fn extract() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::WoodCutter,
        CellStateVariant::Seller,
        CellStateVariant::Back,
    ]
}

fn industry() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Hot,
        CellStateVariant::Insulation,
        CellStateVariant::Feeder,
        CellStateVariant::Back,
    ]
}

fn infrastructure() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Road,
        CellStateVariant::Hub,
        CellStateVariant::Back,
    ]
}

pub fn buildable() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Industry,
        CellStateVariant::Extract,
        CellStateVariant::Infrastructure,
    ]
}

fn menu_variant_transition(cv0: CellStateVariant) -> Option<CellState> {
    match cv0 {
        CellStateVariant::Industry
        | CellStateVariant::Infrastructure
        | CellStateVariant::Extract => Some(celldata::unit_state(cv0)),
        CellStateVariant::Back => Some(celldata::unit_state(CellStateVariant::Unused)),
        _ => None,
    }
}

pub fn explore_able() -> Vec<CellStateVariant> {
    vec![CellStateVariant::Unused]
}

fn has_buildtime() -> Vec<CellStateVariant> {
    let mut ret = industry();
    ret.append(&mut infrastructure());
    ret.append(&mut extract());
    ret
}

fn buildtime(cv: CellStateVariant) -> actionmachine::InProgressWait {
    match cv {
        CellStateVariant::Unused => 2,
        CellStateVariant::Hot => 4,
        CellStateVariant::Feeder => 1,
        CellStateVariant::Seller => 1,
        CellStateVariant::Insulation => 2,
        CellStateVariant::WoodCutter => 3,
        CellStateVariant::Road => 1,
        CellStateVariant::Hub => 10,
        _ => unimplemented!("{:?}", cv),
    }
}

pub fn max_buildtime() -> actionmachine::InProgressWait {
    has_buildtime().into_iter().map(buildtime).max().unwrap()
}

fn has_logistics(pos: hexgrid::Pos, g: &GameState) -> bool {
    match hexgrid::get(pos, &g.matrix) {
        CellState {
            variant: celldata::CellStateVariant::Hub,
            data: CellStateData::Resource { left, .. },
        } => left > 0,
        _ => match hexgrid::get(pos, &g.logistics_plane) {
            LogisticsState::None => false,
            LogisticsState::Available { locations } => {
                locations.into_iter().any(|i| has_logistics(i, g))
            }
            a => unimplemented!("{:?}", a),
        },
    }
}

pub fn build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    if let Some(new_cell) = menu_variant_transition(cv) {
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g
    } else {
        let new_cell = celldata::CellState {
            variant: CellStateVariant::Building,
            data: celldata::CellStateData::InProgress {
                countdown: buildtime(cv),
                on_done_data: actionmachine::OnDoneData::CellStateVariant(cv),
            },
        };
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g = use_builder(pos, g);
        g.action_machine =
            actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
        g
    }
}

fn use_builder(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.matrix = find_logistcs_node(
        pos,
        |cs| match cs {
            CellState {
                variant: celldata::CellStateVariant::Hub,
                data: CellStateData::Resource { left, .. },
            } => left > 0,
            _ => false,
        },
        |i| match i {
            CellState {
                variant,
                data: CellStateData::Resource { left, total },
            } => CellState {
                variant,
                data: CellStateData::Resource {
                    left: left - 1,
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

fn return_builder(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.matrix = find_logistcs_node(
        pos,
        |i| match i {
            CellState {
                variant: celldata::CellStateVariant::Hub,
                data: CellStateData::Resource { left, total },
            } => left < total,
            _ => false,
        },
        |i| match i {
            CellState {
                variant,
                data: CellStateData::Resource { left, total },
            } => CellState {
                variant,
                data: CellStateData::Resource {
                    left: left + 1,
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
    pos: Pos,
    cond: fn(CellState) -> bool,
    update: fn(CellState) -> CellState,
    mut b: hexgrid::Board,
    b2: &LogisticsPlane,
) -> Option<hexgrid::Board> {
    let cs = hexgrid::get(pos, &b);
    let ls = hexgrid::get(pos, &b2);
    if cond(cs) {
        let c1 = update(cs);
        hexgrid::set(pos, c1, &mut b);
        Some(b)
    } else {
        match ls {
            LogisticsState::Available { locations } => {
                for i in locations {
                    match find_logistcs_node(i, cond, update, b.clone(), b2) {
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

fn max_builders() -> i32 {
    3
}

pub fn statespace() -> celldata::Statespace {
    let cv = celldata::CellStateVariant::Building;
    let mut to_build = has_buildtime();
    to_build.append(&mut explore_able());
    let mut ret = vec![];
    for j in 1..actionmachine::in_progress_max(cv) + 1 {
        for b in to_build.clone() {
            ret.push(celldata::CellState {
                variant: cv,
                data: celldata::CellStateData::InProgress {
                    countdown: j,
                    on_done_data: actionmachine::OnDoneData::CellStateVariant(b),
                },
            })
        }
    }
    for i in 0..max_builders() + 1 {
        ret.push(CellState {
            variant: CellStateVariant::Hub,
            data: CellStateData::Resource {
                left: i,
                total: max_builders(),
            },
        })
    }
    ret.push(celldata::unit_state(CellStateVariant::Road));
    ret
}

pub fn finalize_build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g = return_builder(pos, g);
    do_build(cv, pos, g)
}

pub fn do_build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.action_machine = actionmachine::remove(g.action_machine, pos, CellStateVariant::Building);
    g.action_machine = actionmachine::maybe_insert(g.action_machine, pos, cv);
    let new_cell = match cv {
        a @ (CellStateVariant::Insulation
        | CellStateVariant::Feeder
        | CellStateVariant::Unused
        | CellStateVariant::Seller
        | CellStateVariant::Road) => celldata::unit_state(a),
        a @ CellStateVariant::Hot => CellState {
            variant: a,
            data: celldata::CellStateData::Slot {
                slot: celldata::Slot::Empty,
            },
        },
        a @ CellStateVariant::WoodCutter => CellState {
            variant: a,
            data: celldata::CellStateData::InProgress {
                countdown: 3,
                on_done_data: actionmachine::OnDoneData::Nothing,
            },
        },
        a @ CellStateVariant::Hub => {
            let builders = max_builders();
            let new_ls_cell = LogisticsState::Source;
            hexgrid::set(pos, new_ls_cell, &mut g.logistics_plane);

            CellState {
                variant: a,
                data: celldata::CellStateData::Resource {
                    left: builders,
                    total: builders,
                },
            }
        }
        _ => {
            println!("unexpected {:?}", cv);
            unimplemented!()
        }
    };
    hexgrid::set(pos, new_cell, &mut g.matrix);
    if cv == CellStateVariant::Hub {
        g = update_logistics(pos, true, g);
    }
    if cv == CellStateVariant::Road {
        g = update_logistics(pos, false, g);
    }
    if let Some(new_delta) = celldata::leak_delta(cv, pos, &g.matrix) {
        g.resources.leak = g.resources.leak + new_delta;
        g.resources.heat_efficency = g.resources.tiles as f64 / g.resources.leak as f64;
    }
    if celldata::is_hot_v(cv) {
        g.resources.tiles = g.resources.tiles + 1;
    }
    g
}

fn update_logistics(pos: hexgrid::Pos, is_hub: bool, mut g: GameState) -> GameState {
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
