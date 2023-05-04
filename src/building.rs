use std::{cmp::min, collections::HashMap};

use crate::{
    actionmachine::{self},
    celldata::{self, CellState, CellStateData, CellStateVariant},
    hexgrid::{self},
    logistics_plane::{self, LogisticsState},
    resource, GameState,
};

pub(crate) fn has_actions(
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

pub(crate) fn buildable() -> Vec<CellStateVariant> {
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

pub(crate) fn explore_able() -> Vec<CellStateVariant> {
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
        CellStateVariant::Hot => Some(4),
        CellStateVariant::Feeder => Some(1),
        CellStateVariant::Seller => Some(1),
        CellStateVariant::Insulation => Some(2),
        CellStateVariant::WoodCutter => Some(3),
        CellStateVariant::Road => Some(1),
        _ => None,
    }
}

fn buildtime_keep_res(cv: CellStateVariant) -> Option<actionmachine::InProgressWait> {
    match cv {
        CellStateVariant::Unused => Some(2),
        _ => None,
    }
}

fn buildcost_cell(cv: CellStateVariant) -> Option<CellState> {
    match cv {
        to @ CellStateVariant::Hub => Some(resource::new_stockpile(
            CellStateVariant::Building,
            HashMap::from([(resource::ResourceType::Builders, 1)]),
            to,
        )),
        _ => None,
    }
}

pub(crate) fn required_per_build_action(_cv: CellStateVariant) -> resource::ResourcePacket {
    resource::from_key_value(HashMap::from([(resource::ResourceType::Wood, 10)]))
}
pub(crate) fn build_action_req(_cv: CellStateVariant) -> resource::ResourceValue {
    10
}

pub(crate) fn use_builder(pos: hexgrid::Pos, mut g: GameState) -> GameState {
    let p = resource::new_packet(1, 0);
    logistics_plane::try_borrow_resources(pos, p, &mut g).unwrap()
}

pub(crate) fn do_build_progress(
    mut c: CellState,
    p: hexgrid::Pos,
    r: resource::ResourceStockpile,
    cv2: celldata::CellStateVariant,
    mut g: GameState,
) -> GameState {
    g = logistics_plane::return_lp(p, g);

    let builders = resource::get(resource::ResourceType::Builders, r);
    let req = required_per_build_action(cv2);
    let done_threshold = build_action_req(cv2);
    let pre_progress = resource::get(resource::ResourceType::BuildTime, r);
    let work_left = done_threshold - pre_progress;
    let mut progress = 0;
    for _ in 0..min(builders, work_left) {
        if let Some(g1) = logistics_plane::try_take_resources(p, req, &mut g) {
            g = g1;
            progress = progress + 1
        } else {
            break;
        }
    }
    if progress == builders {
        if let Some(g1) =
            logistics_plane::try_borrow_resources(p, resource::new_packet(1, 0), &mut g)
        {
            if let Some(c1) = resource::add(resource::ResourceType::Builders, c, 1) {
                c = c1;
                g = g1;
            }
        }
    }
    if (progress + pre_progress) == done_threshold {
        finalize_build(actionmachine::Other::CellStateVariant(cv2), p, g)
    } else {
        dbg!((c, progress));
        let c1 = resource::add(resource::ResourceType::BuildTime, c, progress).unwrap();
        dbg!(c1);
        hexgrid::set(p, c1, &mut g.matrix);
        g
    }
}

pub(crate) fn max_buildtime() -> actionmachine::InProgressWait {
    has_buildtime()
        .into_iter()
        .filter_map(buildtime)
        .max()
        .unwrap()
}

pub(crate) fn build(cv: CellStateVariant, pos: hexgrid::Pos, mut g: GameState) -> GameState {
    if let Some(new_cell) = menu_variant_transition(cv) {
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g
    } else {
        let new_cell = if let Some(b) = buildtime(cv) {
            actionmachine::new_in_progress_with_variant(CellStateVariant::Building, b, cv)
        } else if let Some(new_cell) = buildcost_cell(cv) {
            new_cell
        } else if let Some(b) = buildtime_keep_res(cv) {
            let old_res = match hexgrid::get(pos, &mut g.matrix) {
                CellState {
                    variant: CellStateVariant::Hidden,
                    data: CellStateData::Resource(resource::Resource::Pure(res)),
                } => res,
                a => todo!("{:?}", a),
            };
            actionmachine::new_in_progress_with_variant_and_resource(
                CellStateVariant::Building,
                b,
                cv,
                old_res,
            )
        } else {
            unimplemented!("{:?}", (cv, pos))
        };
        g = use_builder(pos, g);
        g.action_machine =
            actionmachine::maybe_insert(g.action_machine, pos, CellStateVariant::Building);
        hexgrid::set(pos, new_cell, &mut g.matrix);
        g
    }
}

pub(crate) fn finalize_build(
    oth: actionmachine::Other,
    pos: hexgrid::Pos,
    mut g: GameState,
) -> GameState {
    g = logistics_plane::return_borrows(pos, g);
    g = logistics_plane::return_lp(pos, g);
    do_build(oth, pos, g)
}

pub(crate) fn do_build(
    oth: actionmachine::Other,
    pos: hexgrid::Pos,
    mut g: GameState,
) -> GameState {
    let cv = match oth {
        actionmachine::Other::CellStateVariant(cv) => cv,
        actionmachine::Other::CvAndRS(cv, _) => cv,
    };
    g.action_machine = actionmachine::remove(g.action_machine, pos, CellStateVariant::Building);
    g.action_machine = actionmachine::maybe_insert(g.action_machine, pos, cv);
    let new_cell = match cv {
        a @ (CellStateVariant::Insulation
        | CellStateVariant::Feeder
        | CellStateVariant::Seller
        | CellStateVariant::Road) => celldata::unit_state(a),
        CellStateVariant::Unused => {
            let res = match oth {
                actionmachine::Other::CvAndRS(_, res) => res,
                a => todo!("{:?}", a),
            };
            resource::new_pure_stockpile(cv, resource::to_key_value(res))
        }
        CellStateVariant::Hot => CellState {
            variant: cv,
            data: celldata::CellStateData::Slot {
                slot: celldata::Slot::Empty,
            },
        },
        a @ CellStateVariant::WoodCutter => actionmachine::new_in_progress(a, 3),
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
    dbg!(new_cell);
    hexgrid::set(pos, new_cell, &mut g.matrix);
    if cv == CellStateVariant::Hub {
        g = logistics_plane::update_logistics(pos, true, g);
    }
    if cv == CellStateVariant::Road {
        g = logistics_plane::update_logistics(pos, false, g);
    }
    if let Some(new_delta) = celldata::leak_delta(cv, pos, &mut g.matrix) {
        g.resources.leak = g.resources.leak + new_delta;
        g.resources.heat_efficency = g.resources.tiles as f64 / g.resources.leak as f64;
    }
    if celldata::is_hot_v(cv) {
        g.resources.tiles = g.resources.tiles + 1;
    }
    g
}
