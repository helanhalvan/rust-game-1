use crate::{
    actionmachine,
    celldata::{self, CellState, CellStateVariant},
    hexgrid::{self, Pos},
    GameState,
};

// mirror of the main board (hexgrid::Board) in size
// for use of the building subsytem
// need to keep "available logistics" somewhere
pub type Board = hexgrid::Hexgrid<LogisticsState>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogisticsState {
    None,
    Source { used: i32, total: i32 }, // TODO, remove source and use celldata values instead
    Available { locations: Vec<hexgrid::Pos> },
}

pub fn new_plane(xmax: usize, ymax: usize) -> Board {
    vec![vec![LogisticsState::None; xmax]; ymax]
}

pub fn has_actions(pos: hexgrid::Pos, g: &GameState) -> bool {
    match hexgrid::get(pos, &g.logistics_plane) {
        LogisticsState::None => false,
        LogisticsState::Source { used, total } => total > used,
        LogisticsState::Available { locations } => locations.into_iter().any(|i| has_actions(i, g)),
    }
}

pub fn build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    let new_cell = celldata::CellState {
        variant: CellStateVariant::Building,
        data: celldata::CellStateData::InProgress {
            countdown: buildtime(cv),
            on_done_data: actionmachine::OnDoneData::CellStateVariant(cv),
        },
    };
    hexgrid::set(pos, new_cell, &mut g.matrix);
    g = use_builder(pos, g);
    g.resources.build_in_progress = g.resources.build_in_progress + 1;
    g.action_machine =
        actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
    g
}

fn use_builder(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.logistics_plane = find_logistcs_node(
        pos,
        |i| match i {
            LogisticsState::Source { used, total } => total > used,
            _ => false,
        },
        |i| match i {
            LogisticsState::Source { used, total } => LogisticsState::Source {
                used: used + 1,
                total: total,
            },
            _ => unimplemented!(),
        },
        g.logistics_plane,
    )
    .unwrap();
    g
}

fn return_builder(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.logistics_plane = find_logistcs_node(
        pos,
        |i| match i {
            LogisticsState::Source { used, total } => (used > 0) && (used <= total),
            _ => false,
        },
        |i| match i {
            LogisticsState::Source { used, total } => LogisticsState::Source {
                used: used - 1,
                total: total,
            },
            _ => unimplemented!(),
        },
        g.logistics_plane,
    )
    .unwrap();
    g
}

fn find_logistcs_node(
    pos: Pos,
    cond: fn(LogisticsState) -> bool,
    update: fn(LogisticsState) -> LogisticsState,
    mut board: Board,
) -> Option<Board> {
    let c = hexgrid::get(pos, &board);
    if cond(c.clone()) {
        let c1 = update(c);
        hexgrid::set(pos, c1, &mut board);
        Some(board)
    } else {
        match c {
            LogisticsState::Available { locations } => {
                for i in locations {
                    match find_logistcs_node(i, cond, update, board.clone()) {
                        a @ Some(..) => {
                            return a;
                        }
                        None => {}
                    }
                }
                None
            }
            _ => {
                dbg!(c);
                None
            }
        }
    }
}

fn buildtime(cv: CellStateVariant) -> actionmachine::InProgressWait {
    match cv {
        CellStateVariant::Unused => 2,
        CellStateVariant::Hot => 4,
        CellStateVariant::Feeder => 1,
        CellStateVariant::Seller => 1,
        CellStateVariant::Insulation => 2,
        CellStateVariant::ActionMachine => 3,
        _ => 4,
    }
}

pub fn statespace() -> celldata::Statespace {
    let cv = celldata::CellStateVariant::Building;
    let mut to_build = buildable();
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
    ret
}

pub fn buildable() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Hot,
        CellStateVariant::Insulation,
        CellStateVariant::Feeder,
        CellStateVariant::ActionMachine,
        CellStateVariant::Seller,
    ]
}

pub fn explore_able() -> Vec<CellStateVariant> {
    vec![CellStateVariant::Unused]
}

pub fn finalize_build(
    cv: CellStateVariant,
    pos: hexgrid::Pos,
    mut g: GameState,
) -> (CellState, GameState) {
    g = return_builder(pos, g);
    do_build(cv, pos, g)
}

pub fn do_build(
    cv: CellStateVariant,
    pos: hexgrid::Pos,
    mut g: GameState,
) -> (CellState, GameState) {
    g.action_machine = actionmachine::remove(g.action_machine, pos, CellStateVariant::Building);
    g.action_machine = actionmachine::maybe_insert(g.action_machine, pos, cv);
    let c = match cv {
        a @ (CellStateVariant::Insulation
        | CellStateVariant::Feeder
        | CellStateVariant::Unused
        | CellStateVariant::Seller) => CellState {
            variant: a,
            data: celldata::CellStateData::Unit,
        },
        a @ CellStateVariant::Hot => CellState {
            variant: a,
            data: celldata::CellStateData::Slot {
                slot: celldata::Slot::Empty,
            },
        },
        a @ CellStateVariant::ActionMachine => CellState {
            variant: a,
            data: celldata::CellStateData::InProgress {
                countdown: 3,
                on_done_data: actionmachine::OnDoneData::Nothing,
            },
        },
        a @ CellStateVariant::Hub => {
            let builders = 3;
            g.resources.build_points = g.resources.build_points + builders;
            let new_cell = LogisticsState::Source { used: 0, total: 3 };
            hexgrid::set(pos, new_cell, &mut g.logistics_plane);
            g.logistics_plane = add_to_neighbors(pos, g.logistics_plane);
            CellState {
                variant: a,
                data: celldata::CellStateData::Resource { slot: builders },
            }
        }
        _ => {
            println!("unexpected {:?}", cv);
            unimplemented!()
        }
    };
    if let Some(new_delta) = celldata::leak_delta(cv, pos, &g.matrix) {
        g.resources.leak = g.resources.leak + new_delta;
        g.resources.heat_efficency = g.resources.tiles as f64 / g.resources.leak as f64;
    }
    if celldata::is_hot_v(cv) {
        g.resources.tiles = g.resources.tiles + 1;
    }
    (c, g)
}

fn add_to_neighbors(pos: hexgrid::Pos, mut b: Board) -> Board {
    b = hexgrid::neighbors(pos, &(b.clone()))
        .filter_map(|i| i)
        .fold(b, |mut acc, (pn, c)| match c {
            LogisticsState::None => {
                let new_cell = LogisticsState::Available {
                    locations: vec![pos],
                };
                hexgrid::set(pn, new_cell, &mut acc);
                acc
            }
            LogisticsState::Source { .. } => acc,
            LogisticsState::Available { mut locations } => {
                locations.push(pos);
                let new_cell = LogisticsState::Available { locations };
                hexgrid::set(pn, new_cell, &mut acc);
                acc
            }
        });
    b
}
