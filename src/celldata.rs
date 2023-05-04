use core::fmt;
use std::collections::HashMap;

use enum_iterator::Sequence;

use crate::{actionmachine, hexgrid, resource};

// Data in a cell (position) on the board
// The two-field struct makes a bunch of impossible stuff representable
// Converting back to single enum might be possible
// but implementing into for CellState -> CellstateVariant
// and CellState -> CellStateData
// Seems very boilerplatey
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CellState {
    pub(crate) variant: CellStateVariant,
    pub(crate) data: CellStateData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CellStateData {
    Unit,
    Slot { slot: Slot },
    InProgress(actionmachine::InProgress),
    Resource(resource::Resource),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Slot {
    Empty,
    Done,
}

pub(crate) fn unit_state(cv: CellStateVariant) -> CellState {
    CellState {
        data: CellStateData::Unit,
        variant: cv,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sequence)]
pub(crate) enum CellStateVariant {
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

pub(crate) fn is_hot(c: CellState) -> bool {
    let cv: CellStateVariant = c.into();
    is_hot_v(cv)
}

pub(crate) fn is_hot_v(cv: CellStateVariant) -> bool {
    match cv {
        CellStateVariant::Hot => true,
        _ => false,
    }
}

pub(crate) fn leak_delta(
    cv: CellStateVariant,
    p: hexgrid::Pos,
    m: &mut hexgrid::Board,
) -> Option<i32> {
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
        let n_effects_applied: i32 = hexgrid::neighbors(p, m)
            .map(|(_, i)| {
                let ct: CellStateVariant = i.into();
                if let Some(d) = n_effects.get(&ct) {
                    *d
                } else {
                    0
                }
            })
            .sum();
        Some(base + n_effects_applied)
    } else {
        None
    }
}
