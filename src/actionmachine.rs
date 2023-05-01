use std::collections::HashSet;

use crate::{
    building,
    celldata::{self, CellStateVariant},
    hexgrid, GameState,
};

//crontab but for game triggers
pub type ActionMachine = Vec<HashSet<hexgrid::Pos>>;

pub type Prio = usize;
pub static ACTION_MAX_PRIO: Prio = (*(&CellStateVariant::Last)) as Prio;

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
    return vec![HashSet::new(); ACTION_MAX_PRIO];
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

pub type InProgressWait = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnDoneData {
    Nothing,
    CellStateVariant(celldata::CellStateVariant),
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
            ret.push(celldata::CellState {
                variant: cv,
                data: celldata::CellStateData::InProgress {
                    countdown: j,
                    on_done_data: OnDoneData::Nothing,
                },
            });
        }
    }
    ret
}

fn do_progress_done(
    p @ hexgrid::Pos { x, y }: hexgrid::Pos,
    cv: celldata::CellStateVariant,
    on_done_data: OnDoneData,
    mut g: GameState,
) -> GameState {
    g = match (cv, on_done_data) {
        (celldata::CellStateVariant::WoodCutter, _) => {
            g.resources.wood = g.resources.wood + 1;
            let new_cell = celldata::CellState {
                variant: cv,
                data: celldata::CellStateData::InProgress {
                    countdown: in_progress_max(cv),
                    on_done_data: OnDoneData::Nothing,
                },
            };
            hexgrid::set(p, new_cell, &mut g.matrix);
            g
        }
        (celldata::CellStateVariant::Hot, _) => {
            let new_cell = celldata::CellState {
                variant: cv,
                data: celldata::CellStateData::Slot {
                    slot: celldata::Slot::Done,
                },
            };
            hexgrid::set(p, new_cell, &mut g.matrix);
            g
        }
        (celldata::CellStateVariant::Building, OnDoneData::CellStateVariant(new_cv)) => {
            building::finalize_build(new_cv, p, g)
        }
        a => {
            println!("unexpected {:?}{:?}{:?}", x, y, a);
            unimplemented!()
        }
    };
    g
}

fn do_tick(
    p @ hexgrid::Pos { x, y }: hexgrid::Pos,
    c: celldata::CellState,
    mut g: GameState,
) -> GameState {
    match c {
        celldata::CellState {
            variant,
            data:
                celldata::CellStateData::InProgress {
                    countdown: 1,
                    on_done_data,
                },
        } => {
            g = do_progress_done(p, variant, on_done_data, g);
        }
        celldata::CellState {
            variant,
            data:
                celldata::CellStateData::InProgress {
                    countdown,
                    on_done_data,
                },
        } => {
            let new_cell = celldata::CellState {
                variant,
                data: celldata::CellStateData::InProgress {
                    countdown: countdown - 1,
                    on_done_data,
                },
            };
            hexgrid::set(p, new_cell, &mut g.matrix);
        }
        celldata::CellState {
            variant: celldata::CellStateVariant::Feeder,
            ..
        } => {
            let con: Vec<(hexgrid::Pos, celldata::CellState)> =
                hexgrid::get_connected(p, celldata::is_hot, &g.matrix)
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
                    let new_cell = celldata::CellState {
                        variant: celldata::CellStateVariant::Hot,
                        data: celldata::CellStateData::InProgress {
                            countdown: in_progress_max(celldata::CellStateVariant::Hot),
                            on_done_data: OnDoneData::Nothing,
                        },
                    };
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
                hexgrid::get_connected(p, celldata::is_hot, &g.matrix)
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
                    g.resources.wood = g.resources.wood + 40;
                }
                _ => {}
            }
        }
        celldata::CellState {
            variant: celldata::CellStateVariant::Hot,
            ..
        } => {}
        celldata::CellState {
            variant: celldata::CellStateVariant::Building,
            data: celldata::CellStateData::Resource(r),
        } => match r {
            celldata::Resource::WithVariant(_, _) => todo!(),
            _ => todo!(),
        },
        //TODO handle building + resource
        a => {
            println!("unexpected {:?}{:?}{:?}", x, y, a);
            unimplemented!()
        }
    };
    g
}

pub fn run(mut g: GameState) -> GameState {
    g.resources.wood = g.resources.wood - g.resources.leak;
    let old_acton_machine = g.action_machine.clone();
    for v in old_acton_machine {
        g = v.into_iter().fold(g, |acc, pos| {
            let cell = hexgrid::get(pos, &acc.matrix);
            do_tick(pos, cell, acc)
        })
    }
    g
}
