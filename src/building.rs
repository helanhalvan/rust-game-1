use crate::{
    actionmachine,
    celldata::{self, CellState, CellStateVariant},
    hexgrid, GameState,
};

// Cell content initalizer/constructor
pub fn build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    let new_cell = CellState::InProgress {
        variant: CellStateVariant::Building,
        countdown: actionmachine::in_progress_max(CellStateVariant::Building),
        on_done_data: actionmachine::OnDoneData::CellStateVariant(cv),
    };
    hexgrid::set(pos, new_cell, &mut g.matrix);

    g.resources.actions = g.resources.actions - 1;
    g.action_machine =
        actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
    g
}

pub fn statespace() -> celldata::Statespace {
    let cv = celldata::CellStateVariant::Building;
    let mut to_build = buildable();
    to_build.append(&mut explore_able());
    let mut ret = vec![];
    let mut cv_buff = vec![];
    for j in 1..actionmachine::in_progress_max(cv) + 1 {
        for b in to_build.clone() {
            cv_buff.push(celldata::CellState::InProgress {
                variant: cv,
                countdown: j,
                on_done_data: actionmachine::OnDoneData::CellStateVariant(b),
            })
        }
    }
    ret.push((cv, cv_buff));

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
    g.action_machine = actionmachine::remove(g.action_machine, pos, CellStateVariant::Building);
    g.action_machine = actionmachine::maybe_insert(g.action_machine, pos, cv);
    let c = match cv {
        a @ (CellStateVariant::Insulation
        | CellStateVariant::Feeder
        | CellStateVariant::Unused
        | CellStateVariant::Seller) => CellState::Unit { variant: a },
        a @ CellStateVariant::Hot => CellState::Slot {
            variant: a,
            slot: celldata::Slot::Empty,
        },
        a @ CellStateVariant::ActionMachine => CellState::InProgress {
            variant: a,
            countdown: 3,
            on_done_data: actionmachine::OnDoneData::Nothing,
        },
        _ => {
            println!("unexpected {:?}", cv);
            unimplemented!()
        }
    };
    if let Some(new_delta) = celldata::leak_delta(cv, pos, &g.matrix) {
        g.resources.leak = g.resources.leak + new_delta;
        g.resources.heat_efficency = g.resources.tiles as f64 / g.resources.leak as f64;
    }
    if celldata::is_tile(cv) {
        g.resources.tiles = g.resources.tiles + 1;
    }
    (c, g)
}
