use core::fmt;
use std::collections::HashMap;

use enum_iterator::Sequence;

use crate::{actionmachine, building, hexgrid, resource};

// Data in a cell (position) on the board
// The two-field struct makes a bunch of impossible stuff representable
// Converting back to single enum might be possible
// but implementing into for CellState -> CellstateVariant
// and CellState -> CellStateData
// Seems very boilerplatey
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellState {
    pub variant: CellStateVariant,
    pub data: CellStateData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Pure(resource::ResourceStockpile),
    WithVariant(resource::ResourceStockpile, CellStateVariant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellStateData {
    Unit,
    Slot {
        slot: Slot,
    },
    InProgress {
        countdown: actionmachine::InProgressWait,
        on_done_data: actionmachine::OnDoneData,
    },
    Resource(Resource),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Empty,
    Done,
}

//all possible cellstates, grouped by CellStateVariant
pub type Statespace = Vec<CellState>;

//I'll need a build-statespace for the things with buttons later
// all possible non-interactive cellstates, used by the UI
pub fn non_interactive_statespace() -> Statespace {
    let mut ret = vec![
        unit_state(CellStateVariant::Insulation),
        unit_state(CellStateVariant::Feeder),
        unit_state(CellStateVariant::Seller),
        unit_state(CellStateVariant::Hidden),
        unit_state(CellStateVariant::Unused),
        CellState {
            variant: CellStateVariant::Hot,
            data: CellStateData::Slot { slot: Slot::Empty },
        },
        CellState {
            variant: CellStateVariant::Hot,
            data: CellStateData::Slot { slot: Slot::Done },
        },
    ];
    ret.append(&mut actionmachine::statespace());
    ret.append(&mut building::statespace());
    ret.append(&mut resource::statespace());
    ret
}

pub fn unit_state(cv: CellStateVariant) -> CellState {
    CellState {
        data: CellStateData::Unit,
        variant: cv,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
pub enum CellStateVariant {
    Hidden,
    Unused,
    Hot,
    Insulation,
    Feeder,
    WoodCutter,
    Seller,
    InProgress,
    Building,
    Hub,
    Road,
    OutOfBounds,
    Industry,
    Infrastructure,
    Extract,
    Back,
    Last, //NEEDS TO EXIST AND BE LAST
}

impl Into<CellStateVariant> for CellState {
    fn into(self) -> CellStateVariant {
        match self {
            CellState { variant, .. } => variant,
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
    is_hot_v(cv)
}

pub fn is_hot_v(cv: CellStateVariant) -> bool {
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
