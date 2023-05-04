use std::collections::{HashMap, HashSet};

use crate::{
    building,
    celldata::{self, CellState, CellStateData, CellStateVariant},
    hexgrid, logistics_plane, resource, GameState,
};

//crontab but for game triggers
pub(crate) type ActionMachine = [HashSet<hexgrid::Pos>; ACTION_MAX_PRIO];

pub(crate) type Prio = usize;
pub(crate) const ACTION_MAX_PRIO: Prio = (*(&CellStateVariant::Last)) as Prio;

pub(crate) type InProgressWait = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum InProgress {
    Pure(InProgressWait),
    WithOther(InProgressWait, Other),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Other {
    CellStateVariant(CellStateVariant),
    CvAndRS(CellStateVariant, resource::ResourceStockpile),
}

// for now the main point of prio is to ensure
// all CellStateVariants which are the same are executed after eachother
// to limit effects of order added on order executed
pub(crate) fn prio(cv: celldata::CellStateVariant) -> Option<Prio> {
    match cv {
        CellStateVariant::WoodFarm
        | CellStateVariant::Hot
        | CellStateVariant::Feeder
        | CellStateVariant::Seller
        | CellStateVariant::Building => Some(*(&cv) as usize),
        _ => None,
    }
}

pub(crate) fn new() -> ActionMachine {
    Default::default()
}

pub(crate) fn new_in_progress(cv: CellStateVariant, wait: InProgressWait) -> CellState {
    CellState {
        variant: cv,
        data: CellStateData::InProgress(InProgress::Pure(wait)),
    }
}

pub(crate) fn new_in_progress_with_variant(
    cv: CellStateVariant,
    wait: InProgressWait,
    cv2: CellStateVariant,
) -> CellState {
    new_in_progress_with_other(cv, wait, Other::CellStateVariant(cv2))
}

pub(crate) fn new_in_progress_with_variant_and_resource(
    cv: CellStateVariant,
    wait: InProgressWait,
    cv2: CellStateVariant,
    res: resource::ResourceStockpile,
) -> CellState {
    new_in_progress_with_other(cv, wait, Other::CvAndRS(cv2, res))
}

fn new_in_progress_with_other(cv: CellStateVariant, wait: InProgressWait, oth: Other) -> CellState {
    CellState {
        variant: cv,
        data: CellStateData::InProgress(InProgress::WithOther(wait, oth)),
    }
}

pub(crate) fn maybe_insert(
    mut m: ActionMachine,
    pos: hexgrid::Pos,
    cv: celldata::CellStateVariant,
) -> ActionMachine {
    if let Some(p) = prio(cv) {
        m[p].insert(pos);
    }
    m
}
pub(crate) fn remove(
    mut m: ActionMachine,
    pos: hexgrid::Pos,
    cv: celldata::CellStateVariant,
) -> ActionMachine {
    if let Some(p) = prio(cv) {
        m[p].remove(&pos);
    }
    m
}

pub(crate) fn in_progress_max(cv: celldata::CellStateVariant) -> InProgressWait {
    match cv {
        celldata::CellStateVariant::WoodFarm => 3,
        celldata::CellStateVariant::Hot => 5,
        celldata::CellStateVariant::Building => building::max_buildtime(),
        a => {
            println!("unexpected {:?}", a);
            unimplemented!()
        }
    }
}

fn do_in_progress(
    p: hexgrid::Pos,
    cv: CellStateVariant,
    ip: InProgress,
    mut g: GameState,
) -> GameState {
    match ip {
        InProgress::Pure(1) => do_pure_progress_done(p, cv, g),
        InProgress::WithOther(1, cv2) => do_progress_done_other(p, cv, cv2, g),
        InProgress::Pure(x) => {
            hexgrid::set(p, new_in_progress(cv, x - 1), &mut g.matrix);
            g
        }
        InProgress::WithOther(x, cv2) => {
            hexgrid::set(p, new_in_progress_with_other(cv, x - 1, cv2), &mut g.matrix);
            g
        }
    }
}

fn do_pure_progress_done(p: hexgrid::Pos, cv: CellStateVariant, mut g: GameState) -> GameState {
    match cv {
        celldata::CellStateVariant::WoodFarm => {
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
            println!("unexpected {:?}{:?}", p, cv);
            unimplemented!()
        }
    }
}

fn do_progress_done_other(
    p: hexgrid::Pos,
    cv: celldata::CellStateVariant,
    oth: Other,
    mut g: GameState,
) -> GameState {
    g = match cv {
        celldata::CellStateVariant::Building => building::finalize_build(oth, p, g),
        _ => {
            println!("unexpected {:?}{:?}{:?}", p, cv, oth);
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
            println!("unexpected {:?}{:?}", p, a);
            unimplemented!()
        }
    };
    g
}

pub(crate) fn run(mut g: GameState) -> GameState {
    let old_acton_machine = g.action_machine.clone();
    for v in old_acton_machine {
        g = v.into_iter().fold(g, |mut acc, pos| {
            let cell = hexgrid::get(pos, &mut acc.matrix);
            do_tick(pos, cell, acc)
        })
    }
    g
}
