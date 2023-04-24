use std::collections::HashSet;

use crate::{
    actionmachine,
    celldata::{self, CellState, CellStateData, CellStateVariant},
    hexgrid::{self},
    logistics_plane::{self, LogisticsPlane, LogisticsState},
    resource, GameState,
};

pub fn has_actions(
    pos: hexgrid::Pos,
    c: celldata::CellState,
    g: &GameState,
) -> Option<Vec<CellStateVariant>> {
    if logistics_plane::has_worker(pos, g) {
        match c.variant {
            CellStateVariant::Hidden => Some(explore_able()),
            CellStateVariant::Unused => Some(buildable()),
            CellStateVariant::Industry => Some(industry()),
            CellStateVariant::Infrastructure => Some(infrastructure()),
            CellStateVariant::Extract => Some(extract()),
            _ => None,
        }
    } else {
        None
    }
}

fn extract() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::WoodCutter,
        CellStateVariant::Seller,
        CellStateVariant::Back,
    ]
}

fn industry() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Hot,
        CellStateVariant::Insulation,
        CellStateVariant::Feeder,
        CellStateVariant::Back,
    ]
}

fn infrastructure() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Road,
        CellStateVariant::Hub,
        CellStateVariant::Back,
    ]
}

pub fn buildable() -> Vec<CellStateVariant> {
    vec![
        CellStateVariant::Industry,
        CellStateVariant::Extract,
        CellStateVariant::Infrastructure,
    ]
}

fn menu_variant_transition(cv0: CellStateVariant) -> Option<CellState> {
    match cv0 {
        CellStateVariant::Industry
        | CellStateVariant::Infrastructure
        | CellStateVariant::Extract => Some(celldata::unit_state(cv0)),
        CellStateVariant::Back => Some(celldata::unit_state(CellStateVariant::Unused)),
        _ => None,
    }
}

pub fn explore_able() -> Vec<CellStateVariant> {
    vec![CellStateVariant::Unused]
}

fn has_buildtime() -> Vec<CellStateVariant> {
    let mut ret = industry();
    ret.append(&mut infrastructure());
    ret.append(&mut extract());
    ret
}

fn buildtime(cv: CellStateVariant) -> Option<actionmachine::InProgressWait> {
    match cv {
        CellStateVariant::Unused => Some(2),
        CellStateVariant::Hot => Some(4),
        CellStateVariant::Feeder => Some(1),
        CellStateVariant::Seller => Some(1),
        CellStateVariant::Insulation => Some(2),
        CellStateVariant::WoodCutter => Some(3),
        CellStateVariant::Road => Some(1),
        CellStateVariant::Hub => Some(10),
        _ => None,
    }
}

pub fn max_buildtime() -> actionmachine::InProgressWait {
    has_buildtime()
        .into_iter()
        .filter_map(buildtime)
        .max()
        .unwrap()
}

pub fn build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    if let Some(new_cell) = menu_variant_transition(cv) {
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g
    } else if let Some(b) = buildtime(cv) {
        let new_cell = celldata::CellState {
            variant: CellStateVariant::Building,
            data: celldata::CellStateData::InProgress {
                countdown: b,
                on_done_data: actionmachine::OnDoneData::CellStateVariant(cv),
            },
        };
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g = logistics_plane::use_builder(pos, g);
        g.action_machine =
            actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
        g
    } else {
        unimplemented!("{:?}", (cv, pos, g))
    }
}

fn max_builders() -> i32 {
    3
}

fn max_lp() -> i32 {
    9
}

pub fn statespace() -> celldata::Statespace {
    let cv = celldata::CellStateVariant::Building;
    let mut to_build = has_buildtime();
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
    for i in 0..max_builders() + 1 {
        for j in 0..max_lp() {
            ret.push(resource::new_stockpile(CellStateVariant::Hub, i, j))
        }
    }
    ret.push(celldata::unit_state(CellStateVariant::Road));
    ret
}

pub fn finalize_build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g = logistics_plane::return_builder(pos, g);
    do_build(cv, pos, g)
}

pub fn do_build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g.action_machine = actionmachine::remove(g.action_machine, pos, CellStateVariant::Building);
    g.action_machine = actionmachine::maybe_insert(g.action_machine, pos, cv);
    let new_cell = match cv {
        a @ (CellStateVariant::Insulation
        | CellStateVariant::Feeder
        | CellStateVariant::Unused
        | CellStateVariant::Seller
        | CellStateVariant::Road) => celldata::unit_state(a),
        a @ CellStateVariant::Hot => CellState {
            variant: a,
            data: celldata::CellStateData::Slot {
                slot: celldata::Slot::Empty,
            },
        },
        a @ CellStateVariant::WoodCutter => CellState {
            variant: a,
            data: celldata::CellStateData::InProgress {
                countdown: 3,
                on_done_data: actionmachine::OnDoneData::Nothing,
            },
        },
        a @ CellStateVariant::Hub => {
            let builders = max_builders();
            let logistics_points = max_lp();
            let new_ls_cell = LogisticsState::Source;
            hexgrid::set(pos, new_ls_cell, &mut g.logistics_plane);
            resource::new_stockpile(cv, builders, logistics_points)
        }
        _ => {
            println!("unexpected {:?}", cv);
            unimplemented!()
        }
    };
    hexgrid::set(pos, new_cell, &mut g.matrix);
    if cv == CellStateVariant::Hub {
        g = logistics_plane::update_logistics(pos, true, g);
    }
    if cv == CellStateVariant::Road {
        g = logistics_plane::update_logistics(pos, false, g);
    }
    if let Some(new_delta) = celldata::leak_delta(cv, pos, &g.matrix) {
        g.resources.leak = g.resources.leak + new_delta;
        g.resources.heat_efficency = g.resources.tiles as f64 / g.resources.leak as f64;
    }
    if celldata::is_hot_v(cv) {
        g.resources.tiles = g.resources.tiles + 1;
    }
    g
}
