use core::fmt;
use std::collections::HashMap;

use crate::hexgrid;

//Data in a cell (position) on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellState {
    Hidden,
    Unused,
    Hot(bool),
    Insulation,
    Feeder,
    ActionMachine(i32),
}

//no data variant of CellState, ECS type key if this was a proper ECS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellStateVariant {
    Hidden,
    Unused,
    Hot,
    Insulation,
    Feeder,
    ActionMachine,
}

impl fmt::Display for CellStateVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<CellStateVariant> for CellState {
    fn into(self) -> CellStateVariant {
        match self {
            CellState::Hidden => CellStateVariant::Hidden,
            CellState::Unused => CellStateVariant::Unused,
            CellState::Hot(_) => CellStateVariant::Hot,
            CellState::Insulation => CellStateVariant::Insulation,
            CellState::Feeder => CellStateVariant::Feeder,
            CellState::ActionMachine(_) => CellStateVariant::ActionMachine,
        }
    }
}
// Cell content initalizer/constructor
impl Into<CellState> for CellStateVariant {
    fn into(self) -> CellState {
        match self {
            CellStateVariant::Hidden => CellState::Hidden,
            CellStateVariant::Unused => CellState::Unused,
            CellStateVariant::Hot => CellState::Hot(false),
            CellStateVariant::Insulation => CellState::Insulation,
            CellStateVariant::Feeder => CellState::Feeder,
            CellStateVariant::ActionMachine => CellState::ActionMachine(3),
        }
    }
}
// needs to run code periodically
pub fn is_action_machine(cv: CellStateVariant) -> bool {
    match cv {
        CellStateVariant::Feeder => true,
        CellStateVariant::ActionMachine => true,
        _ => false,
    }
}
pub fn is_tile(cv: CellStateVariant) -> bool {
    match cv {
        CellStateVariant::Hot => true,
        _ => false,
    }
}
pub fn buildable() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Hot,
        CellStateVariant::Insulation,
        CellStateVariant::Feeder,
        CellStateVariant::ActionMachine,
    ]
}

pub fn leak_delta(cv: CellStateVariant, (x, y): (usize, usize), m: &hexgrid::Board) -> Option<i32> {
    if let Some((base, n_effects)) = match cv {
        CellStateVariant::Insulation => Some((0, HashMap::from([(CellStateVariant::Hot, -1)]))),
        CellStateVariant::Hot => Some((
            12,
            HashMap::from([
                (CellStateVariant::Hot, -2),
                (CellStateVariant::Insulation, -1),
            ]),
        )),
        _ => None,
    } {
        let n_effects_applied: i32 = hexgrid::neighbors(x, y, &m)
            .iter()
            .map(|i| match i {
                Some((_, _, cc)) => {
                    let ct: CellStateVariant = (*cc).into();
                    if let Some(d) = n_effects.get(&ct) {
                        *d
                    } else {
                        0
                    }
                }
                _ => 0,
            })
            .sum();
        Some(base + n_effects_applied)
    } else {
        None
    }
}
