use core::fmt;
use std::collections::HashMap;

use crate::{actionmachine, building, hexgrid};

//Data in a cell (position) on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellState {
    Unit {
        variant: CellStateVariant,
    },
    Slot {
        variant: CellStateVariant,
        slot: Slot,
    },
    InProgress {
        variant: CellStateVariant,
        countdown: actionmachine::InProgressWait,
        on_done_data: actionmachine::OnDoneData,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Empty,
    Done,
}

//all possible cellstates, grouped by CellStateVariant
pub type Statespace = Vec<(CellStateVariant, Vec<CellState>)>;

//I'll need a build-statespace for the things with buttons later
// all possible non-interactive cellstates, used by the UI
pub fn non_interactive_statespace() -> Statespace {
    let mut ret = vec![
        //(CellStateVariant::Hidden, vec![CellState::Hidden]),
        //(CellStateVariant::Unused, vec![CellState::Unused]),
        (
            CellStateVariant::Insulation,
            vec![CellState::Unit {
                variant: CellStateVariant::Insulation,
            }],
        ),
        (
            CellStateVariant::Feeder,
            vec![CellState::Unit {
                variant: CellStateVariant::Feeder,
            }],
        ),
        (
            CellStateVariant::Seller,
            vec![CellState::Unit {
                variant: CellStateVariant::Seller,
            }],
        ),
        (
            CellStateVariant::Hot,
            vec![
                CellState::Slot {
                    variant: CellStateVariant::Hot,
                    slot: Slot::Empty,
                },
                CellState::Slot {
                    variant: CellStateVariant::Hot,
                    slot: Slot::Done,
                },
            ],
        ),
    ];
    ret.append(&mut actionmachine::statespace());
    ret.append(&mut building::statespace());
    ret
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
    Building,
}

impl Into<CellStateVariant> for CellState {
    fn into(self) -> CellStateVariant {
        match self {
            CellState::Unit { variant } => variant,
            CellState::Slot { variant, .. } => variant,
            CellState::InProgress { variant, .. } => variant,
        }
    }
}

impl fmt::Display for CellStateVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn is_hot(c: CellState) -> bool {
    let cv: CellStateVariant = c.into();
    cv == CellStateVariant::Hot
}

pub fn is_tile(cv: CellStateVariant) -> bool {
    match cv {
        CellStateVariant::Hot => true,
        _ => false,
    }
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
            .map(|i| match i {
                Some((_, cc)) => {
                    let ct: CellStateVariant = (cc).into();
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
