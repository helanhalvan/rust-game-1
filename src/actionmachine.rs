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

pub fn run(
    mut r: GameResources,
    m: ActionMachine,
    mut b: hexgrid::Board,
) -> (GameResources, hexgrid::Board) {
    r.wood = r.wood - r.leak;
    for v in m {
        for i in hexgrid::pos_iter_to_cells(v, &b) {
            match i {
                Some((x, y, celldata::CellState::ActionMachine(0))) => {
                    r.actions = r.actions + 1;
                    b[x][y] = celldata::CellState::ActionMachine(3);
                }
                Some((x, y, celldata::CellState::ActionMachine(count))) => {
                    b[x][y] = celldata::CellState::ActionMachine(count - 1);
                }
                Some((x, y, celldata::CellState::Feeder)) => {
                    let con: Vec<(usize, usize, celldata::CellState)> =
                        hexgrid::get_connected(x, y, celldata::CellStateVariant::Hot, &b)
                            .into_iter()
                            .filter(|(_x, _y, i)| match i {
                                celldata::CellState::Hot {
                                    slot: celldata::Slot::Empty,
                                    ..
                                } => true,
                                _ => false,
                            })
                            .collect();
                    match con.get(0) {
                        Some((
                            hx,
                            hy,
                            celldata::CellState::Hot {
                                slot: celldata::Slot::Empty,
                            },
                        )) => {
                            b[*hx][*hy] = celldata::CellState::Hot {
                                slot: celldata::Slot::Progress(5),
                            }
                        }
                        _ => {}
                    }
                }
                Some((_x, _y, celldata::CellState::Hot { slot: s }))
                    if s == celldata::Slot::Done || s == celldata::Slot::Empty => {}
                Some((
                    x,
                    y,
                    celldata::CellState::Hot {
                        slot: celldata::Slot::Progress(1),
                    },
                )) => {
                    b[x][y] = celldata::CellState::Hot {
                        slot: celldata::Slot::Done,
                    };
                }
                Some((
                    x,
                    y,
                    celldata::CellState::Hot {
                        slot: celldata::Slot::Progress(p),
                    },
                )) => {
                    b[x][y] = celldata::CellState::Hot {
                        slot: celldata::Slot::Progress(p - 1),
                    };
                }
                Some((x, y, celldata::CellState::Seller)) => {
                    let con: Vec<(usize, usize, celldata::CellState)> =
                        hexgrid::get_connected(x, y, celldata::CellStateVariant::Hot, &b)
                            .into_iter()
                            .filter(|(_x, _y, i)| match i {
                                celldata::CellState::Hot {
                                    slot: celldata::Slot::Done,
                                    ..
                                } => true,
                                _ => false,
                            })
                            .collect();
                    match con.get(0) {
                        Some((
                            hx,
                            hy,
                            celldata::CellState::Hot {
                                slot: celldata::Slot::Done,
                            },
                        )) => {
                            b[*hx][*hy] = celldata::CellState::Hot {
                                slot: celldata::Slot::Empty,
                            };
                            r.actions = r.actions + 1;
                            r.wood = r.wood + 40;
                        }
                        _ => {}
                    }
                }
                None => {
                    println!("unexpected NONE");
                    unimplemented!()
                }
                Some((x, y, a)) => {
                    println!("unexpected {:?}{:?}{:?}", x, y, a);
                    unimplemented!()
                }
            }
        }
    }
    return (r, b);
}
