use core::fmt;
use std::collections::HashMap;

use crate::{actionmachine, hexgrid};

//Data in a cell (position) on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellState {
    Hidden,
    Unused,
    Hot {
        slot: Slot,
    },
    Insulation,
    Feeder,
    InProgress {
        variant: CellStateVariant,
        countdown: actionmachine::InProgressWait,
    },
    Seller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellStateVariant {
    Hidden,
    Unused,
    Hot,
    Insulation,
    Feeder,
    ActionMachine,
    Seller,
    InProgress,
}

impl Into<CellStateVariant> for CellState {
    fn into(self) -> CellStateVariant {
        match self {
            CellState::Hidden => CellStateVariant::Hidden,
            CellState::Unused => CellStateVariant::Unused,
            CellState::Hot { .. } => CellStateVariant::Hot,
            CellState::Insulation => CellStateVariant::Insulation,
            CellState::Feeder => CellStateVariant::Feeder,
            CellState::Seller => CellStateVariant::Seller,
            CellState::InProgress { .. } => CellStateVariant::InProgress,
        }
    }
}

impl fmt::Display for CellStateVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Empty,
    Done,
}

pub fn is_hot(c: CellState) -> bool {
    match c {
        CellState::Hot { .. } => true,
        CellState::InProgress { variant, .. } if variant == CellStateVariant::Hot => true,
        _ => false,
    }
}

// Cell content initalizer/constructor
pub fn build(cv: CellStateVariant) -> CellState {
    match cv {
        CellStateVariant::Unused => CellState::Unused,
        CellStateVariant::Hot => CellState::Hot { slot: Slot::Empty },
        CellStateVariant::Insulation => CellState::Insulation,
        CellStateVariant::Feeder => CellState::Feeder,
        CellStateVariant::ActionMachine => CellState::InProgress {
            variant: cv,
            countdown: 3,
        },
        CellStateVariant::Seller => CellState::Seller,
        _ => {
            println!("unexpected {:?}", cv);
            unimplemented!()
        }
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
        CellStateVariant::Seller,
    ]
}

pub fn leak_delta(cv: CellStateVariant, p: hexgrid::Pos, m: &hexgrid::Board) -> Option<i32> {
    if let Some((base, n_effects)) = match cv {
        CellStateVariant::Insulation => Some((0, HashMap::from([(CellStateVariant::Hot, -1)]))),
        CellStateVariant::Hot => Some((
            12,
            HashMap::from([
                (CellStateVariant::Hot, -4),
                (CellStateVariant::Insulation, -1),
            ]),
        )),
        _ => None,
    } {
        let n_effects_applied: i32 = hexgrid::neighbors(p, &m)
            .iter()
            .map(|i| match i {
                Some((_, cc)) => {
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
