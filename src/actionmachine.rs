use crate::{celldata, hexgrid, GameResources};

//crontab but for game triggers
pub type ActionMachine = Vec<Vec<hexgrid::Pos>>;

type ActionMachinePrio = usize;
pub static ACTION_MAX_PRIO: ActionMachinePrio = 4;

pub fn prio(cv: celldata::CellStateVariant) -> Option<ActionMachinePrio> {
    match cv {
        celldata::CellStateVariant::ActionMachine => Some(0),
        celldata::CellStateVariant::Seller => Some(1),
        celldata::CellStateVariant::Hot => Some(2),
        celldata::CellStateVariant::Feeder => Some(3),
        _ => None,
    }
}

pub fn new() -> ActionMachine {
    return vec![Vec::new(); ACTION_MAX_PRIO];
}

pub fn maybe_insert(
    mut m: ActionMachine,
    pos: hexgrid::Pos,
    cv: celldata::CellStateVariant,
) -> ActionMachine {
    if let Some(p) = prio(cv) {
        m[p].push(pos);
    }
    m
}

pub type InProgressWait = u32;

fn in_progress_max(cv: celldata::CellStateVariant) -> InProgressWait {
    match cv {
        celldata::CellStateVariant::ActionMachine => 3,
        celldata::CellStateVariant::Hot => 5,
        a => {
            println!("unexpected {:?}", a);
            unimplemented!()
        }
    }
}

fn do_progress_done(
    p @ hexgrid::Pos { x, y }: hexgrid::Pos,
    cv: celldata::CellStateVariant,
    mut r: GameResources,
    mut b: hexgrid::Board,
) -> (GameResources, hexgrid::Board) {
    let new_cell = match cv {
        celldata::CellStateVariant::ActionMachine => {
            r.actions = r.actions + 1;
            celldata::CellState::InProgress {
                variant: cv,
                countdown: in_progress_max(cv),
            }
        }
        celldata::CellStateVariant::Hot => celldata::CellState::Hot {
            slot: celldata::Slot::Done,
        },
        a => {
            println!("unexpected {:?}{:?}{:?}", x, y, a);
            unimplemented!()
        }
    };
    hexgrid::set(p, new_cell, &mut b);
    (r, b)
}

fn do_tick(
    p @ hexgrid::Pos { x, y }: hexgrid::Pos,
    c: celldata::CellState,
    mut r: GameResources,
    mut b: hexgrid::Board,
) -> (GameResources, hexgrid::Board) {
    match c {
        celldata::CellState::InProgress {
            variant,
            countdown: 0,
        } => {
            let (r1, b1) = do_progress_done(p, variant, r, b);
            r = r1;
            b = b1;
        }
        celldata::CellState::InProgress {
            variant, countdown, ..
        } => {
            let new_cell = celldata::CellState::InProgress {
                countdown: countdown - 1,
                variant,
            };
            hexgrid::set(p, new_cell, &mut b);
        }
        celldata::CellState::Feeder => {
            let con: Vec<(hexgrid::Pos, celldata::CellState)> =
                hexgrid::get_connected(p, celldata::is_hot, &b)
                    .into_iter()
                    .filter(|(_p, i)| match i {
                        celldata::CellState::Hot {
                            slot: celldata::Slot::Empty,
                            ..
                        } => true,
                        _ => false,
                    })
                    .collect();
            match con.get(0) {
                Some((
                    hp,
                    celldata::CellState::Hot {
                        slot: celldata::Slot::Empty,
                    },
                )) => {
                    let new_cell = celldata::CellState::InProgress {
                        variant: celldata::CellStateVariant::Hot,
                        countdown: in_progress_max(celldata::CellStateVariant::Hot),
                    };
                    hexgrid::set(*hp, new_cell, &mut b);
                }
                _ => {}
            }
        }
        celldata::CellState::Seller => {
            let con: Vec<(hexgrid::Pos, celldata::CellState)> =
                hexgrid::get_connected(p, celldata::is_hot, &b)
                    .into_iter()
                    .filter(|(_p, i)| match i {
                        celldata::CellState::Hot {
                            slot: celldata::Slot::Done,
                            ..
                        } => true,
                        _ => false,
                    })
                    .collect();
            match con.get(0) {
                Some((
                    hp,
                    celldata::CellState::Hot {
                        slot: celldata::Slot::Done,
                    },
                )) => {
                    let new_cell = celldata::CellState::Hot {
                        slot: celldata::Slot::Empty,
                    };
                    hexgrid::set(*hp, new_cell, &mut b);
                    r.actions = r.actions + 1;
                    r.wood = r.wood + 40;
                }
                _ => {}
            }
        }
        celldata::CellState::Hot { .. } => {}
        a => {
            println!("unexpected {:?}{:?}{:?}", x, y, a);
            unimplemented!()
        }
    }
    (r, b)
}

pub fn run(
    mut r: GameResources,
    m: ActionMachine,
    mut b: hexgrid::Board,
) -> (GameResources, hexgrid::Board) {
    r.wood = r.wood - r.leak;
    for v in m {
        for i in hexgrid::pos_iter_to_cells(v, &b) {
            match i {
                Some((pos, cell)) => {
                    let (r1, b1) = do_tick(pos, cell, r, b);
                    b = b1;
                    r = r1;
                }
                None => {
                    println!("unexpected NONE");
                    unimplemented!()
                }
            }
        }
    }
    return (r, b);
}
