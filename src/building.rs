use crate::{
    actionmachine,
    celldata::{self, CellState, CellStateVariant},
    hexgrid, GameState,
};

// mirror of the main board (hexgrid::Board) in size
// for use of the building subsytem
// need to keep "available logistics" somewhere
pub type Board = hexgrid::Hexgrid<LogisticsState>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogisticsState {
    None,
    Source { used: i32, total: i32 },
    Available { localtions: Vec<hexgrid::Pos> },
}

pub fn new_plane(xmax: usize, ymax: usize) -> Board {
    vec![vec![LogisticsState::None; xmax]; ymax]
}

pub fn has_actions(_pos: hexgrid::Pos, g: &GameState) -> bool {
    g.resources.build_points > g.resources.build_in_progress
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

    g.resources.build_in_progress = g.resources.build_in_progress + 1;
    g.action_machine =
        actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
    g
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
    pos @ hexgrid::Pos { x, y }: hexgrid::Pos,
    mut g: GameState,
) -> (CellState, GameState) {
    g.resources.build_in_progress = g.resources.build_in_progress - 1;
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
            g.logistics_plane[x][y] = LogisticsState::Source { used: 0, total: 3 };

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
