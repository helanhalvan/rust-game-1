use std::collections::HashMap;

use crate::{
    actionmachine,
    celldata::{self, CellState, CellStateVariant},
    hexgrid::{self},
    logistics_plane::{self, LogisticsState},
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
    enum_iterator::all::<CellStateVariant>()
        .filter(|i| match buildtime(*i) {
            None => false,
            Some(_) => true,
        })
        .collect()
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
        _ => None,
    }
}

fn buildcost(cv: CellStateVariant) -> Option<CellState> {
    match cv {
        CellStateVariant::Hub => Some(resource::new_stockpile(
            CellStateVariant::Building,
            HashMap::from([(resource::ResourceType::Builders, 1)]),
        )),
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
    } else if let Some(new_cell) = buildcost(cv) {
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g = logistics_plane::use_builder(pos, g);
        g.action_machine =
            actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
        g
    } else {
        unimplemented!("{:?}", (g, cv, pos))
    }
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
    ret.push(celldata::unit_state(CellStateVariant::Road));
    ret
}

pub fn finalize_build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    g = logistics_plane::return_borrows(pos, g);
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
        CellStateVariant::Hot => CellState {
            variant: cv,
            data: celldata::CellStateData::Slot {
                slot: celldata::Slot::Empty,
            },
        },
        CellStateVariant::WoodCutter => CellState {
            variant: cv,
            data: celldata::CellStateData::InProgress {
                countdown: 3,
                on_done_data: actionmachine::OnDoneData::Nothing,
            },
        },
        CellStateVariant::Hub => {
            let new_ls_cell = LogisticsState::Source;
            hexgrid::set(pos, new_ls_cell, &mut g.logistics_plane);
            resource::new_hub()
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
