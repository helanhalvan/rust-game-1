use std::collections::{HashMap, HashSet};

use crate::{
    building,
    celldata::{self, CellState, CellStateData, CellStateVariant},
    hexgrid, logistics_plane, resource, GameState,
};

//crontab but for game triggers
pub type ActionMachine = [HashSet<hexgrid::Pos>; ACTION_MAX_PRIO];

pub type Prio = usize;
pub const ACTION_MAX_PRIO: Prio = (*(&CellStateVariant::Last)) as Prio;

pub type InProgressWait = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnDoneData {
    Nothing,
    CellStateVariant(celldata::CellStateVariant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InProgress {
    Pure(InProgressWait),
    WithVariant(InProgressWait, CellStateVariant),
}

// for now the main point of prio is to ensure
// all CellStateVariants which are the same are executed after eachother
// to limit effects of order added on order executed
pub fn prio(cv: celldata::CellStateVariant) -> Option<Prio> {
    match cv {
        CellStateVariant::WoodCutter
        | CellStateVariant::Hot
        | CellStateVariant::Feeder
        | CellStateVariant::Seller
        | CellStateVariant::Building => Some(*(&cv) as usize),
        _ => None,
    }
}

pub fn new() -> ActionMachine {
    Default::default()
}

pub fn new_in_progress(cv: CellStateVariant, wait: InProgressWait) -> CellState {
    CellState {
        variant: cv,
        data: CellStateData::InProgress(InProgress::Pure(wait)),
    }
}

pub fn new_in_progress_with_variant(
    cv: CellStateVariant,
    wait: InProgressWait,
    cv2: CellStateVariant,
) -> CellState {
    CellState {
        variant: cv,
        data: CellStateData::InProgress(InProgress::WithVariant(wait, cv2)),
    }
}

pub fn maybe_insert(
    mut m: ActionMachine,
    pos: hexgrid::Pos,
    cv: celldata::CellStateVariant,
) -> ActionMachine {
    if let Some(p) = prio(cv) {
        m[p].insert(pos);
    }
    m
}
pub fn remove(
    mut m: ActionMachine,
    pos: hexgrid::Pos,
    cv: celldata::CellStateVariant,
) -> ActionMachine {
    if let Some(p) = prio(cv) {
        m[p].remove(&pos);
    }
    m
}

pub fn in_progress_variants() -> [celldata::CellStateVariant; 2] {
    [
        celldata::CellStateVariant::WoodCutter,
        celldata::CellStateVariant::Hot,
    ]
}

pub fn in_progress_max(cv: celldata::CellStateVariant) -> InProgressWait {
    match cv {
        celldata::CellStateVariant::WoodCutter => 3,
        celldata::CellStateVariant::Hot => 5,
        celldata::CellStateVariant::Building => building::max_buildtime(),
        a => {
            println!("unexpected {:?}", a);
            unimplemented!()
        }
    }
}

pub fn statespace() -> celldata::Statespace {
    let mut ret = vec![];
    for cv in in_progress_variants() {
        for j in 1..in_progress_max(cv) + 1 {
            ret.push(new_in_progress(cv, j));
        }
    }
    ret
}

fn do_in_progress(
    p: hexgrid::Pos,
    cv: CellStateVariant,
    ip: InProgress,
    mut g: GameState,
) -> GameState {
    match ip {
        InProgress::Pure(1) => do_pure_progress_done(p, cv, g),
        InProgress::WithVariant(1, cv2) => do_progress_done_extra_variant(p, cv, cv2, g),
        InProgress::Pure(x) => {
            hexgrid::set(p, new_in_progress(cv, x - 1), &mut g.matrix);
            g
        }
        InProgress::WithVariant(x, cv2) => {
            hexgrid::set(
                p,
                new_in_progress_with_variant(cv, x - 1, cv2),
                &mut g.matrix,
            );
            g
        }
    }
}

fn do_pure_progress_done(p: hexgrid::Pos, cv: CellStateVariant, mut g: GameState) -> GameState {
    match cv {
        celldata::CellStateVariant::WoodCutter => {
            g = logistics_plane::return_lp(p, g);
            let packet =
                resource::from_key_value(HashMap::from([(resource::ResourceType::Wood, -10)]));
            if let Some(g1) = logistics_plane::try_take_resources(p, packet, &mut g) {
                g = g1;
                let new_cell = new_in_progress(cv, in_progress_max(cv));
                hexgrid::set(p, new_cell, &mut g.matrix);
            }
            g
        }
        celldata::CellStateVariant::Hot => {
            let new_cell = celldata::CellState {
                variant: cv,
                data: celldata::CellStateData::Slot {
                    slot: celldata::Slot::Done,
                },
            };
            hexgrid::set(p, new_cell, &mut g.matrix);
            g
        }
        _a => {
            println!("unexpected {:?}{:?}{:?}", g, p, cv);
            unimplemented!()
        }
    }
}

fn do_progress_done_extra_variant(
    p: hexgrid::Pos,
    cv: celldata::CellStateVariant,
    cv2: celldata::CellStateVariant,
    mut g: GameState,
) -> GameState {
    g = match (cv, cv2) {
        (celldata::CellStateVariant::Building, new_cv) => building::finalize_build(new_cv, p, g),
        _ => {
            println!("unexpected {:?}{:?}{:?}{:?}", g, p, cv, cv2);
            unimplemented!()
        }
    };
    g
}

fn do_tick(p: hexgrid::Pos, c: celldata::CellState, mut g: GameState) -> GameState {
    match c {
        celldata::CellState {
            variant,
            data: celldata::CellStateData::InProgress(in_progress),
        } => {
            g = do_in_progress(p, variant, in_progress, g);
        }
        celldata::CellState {
            variant: celldata::CellStateVariant::Feeder,
            ..
        } => {
            let con: Vec<(hexgrid::Pos, celldata::CellState)> =
                hexgrid::get_connected(p, celldata::is_hot, &mut g.matrix)
                    .into_iter()
                    .filter(|(_p, i)| match i {
                        celldata::CellState {
                            variant: celldata::CellStateVariant::Hot,
                            data:
                                celldata::CellStateData::Slot {
                                    slot: celldata::Slot::Empty,
                                },
                            ..
                        } => true,
                        _ => false,
                    })
                    .collect();
            match con.get(0) {
                Some((
                    hp,
                    celldata::CellState {
                        variant: celldata::CellStateVariant::Hot,
                        data:
                            celldata::CellStateData::Slot {
                                slot: celldata::Slot::Empty,
                            },
                        ..
                    },
                )) => {
                    let cv = celldata::CellStateVariant::Hot;
                    let new_cell = new_in_progress(cv, in_progress_max(cv));
                    hexgrid::set(*hp, new_cell, &mut g.matrix);
                }
                _ => {}
            }
        }
        celldata::CellState {
            variant: celldata::CellStateVariant::Seller,
            ..
        } => {
            let con: Vec<(hexgrid::Pos, celldata::CellState)> =
                hexgrid::get_connected(p, celldata::is_hot, &mut g.matrix)
                    .into_iter()
                    .filter(|(_p, i)| match i {
                        celldata::CellState {
                            variant: celldata::CellStateVariant::Hot,
                            data:
                                celldata::CellStateData::Slot {
                                    slot: celldata::Slot::Done,
                                },
                            ..
                        } => true,
                        _ => false,
                    })
                    .collect();
            match con.get(0) {
                Some((
                    hp,
                    celldata::CellState {
                        variant: celldata::CellStateVariant::Hot,
                        data:
                            celldata::CellStateData::Slot {
                                slot: celldata::Slot::Done,
                            },
                    },
                )) => {
                    let new_cell = celldata::CellState {
                        variant: celldata::CellStateVariant::Hot,
                        data: celldata::CellStateData::Slot {
                            slot: celldata::Slot::Empty,
                        },
                    };
                    hexgrid::set(*hp, new_cell, &mut g.matrix);
                    // TODO selling should make gold or something
                }
                _ => {}
            }
        }
        celldata::CellState {
            variant: celldata::CellStateVariant::Hot,
            ..
        } => {}
        c @ celldata::CellState {
            variant: celldata::CellStateVariant::Building,
            data:
                celldata::CellStateData::Resource(resource::Resource::WithVariant(resources, goal_cv)),
        } => g = building::do_build_progress(c, p, resources, goal_cv, g),
        a => {
            println!("unexpected {:?}{:?}{:?}", g, p, a);
            unimplemented!()
        }
    };
    g
}

pub fn run(mut g: GameState) -> GameState {
    let old_acton_machine = g.action_machine.clone();
    for v in old_acton_machine {
        g = v.into_iter().fold(g, |mut acc, pos| {
            let cell = hexgrid::get(pos, &mut acc.matrix);
            do_tick(pos, cell, acc)
        })
    }
    g
}
