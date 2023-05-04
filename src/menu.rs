use crate::{
    celldata::{self, CellState, CellStateVariant},
    hexgrid, logistics_plane, resource, GameState,
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
            CellStateVariant::Extract => Some(extract(c)),
            _ => None,
        }
    } else {
        None
    }
}

pub(crate) fn transition(
    cv0: CellStateVariant,
    pos: hexgrid::Pos,
    g: &mut GameState,
) -> Option<CellState> {
    let old_data = hexgrid::get(pos, &mut g.matrix).data;
    match cv0 {
        CellStateVariant::Industry
        | CellStateVariant::Infrastructure
        | CellStateVariant::Extract => Some(celldata::new(cv0, old_data)),
        CellStateVariant::Back => Some(celldata::new(CellStateVariant::Unused, old_data)),
        _ => None,
    }
}

fn extract(c: CellState) -> Vec<CellStateVariant> {
    let mut res = vec![
        CellStateVariant::WoodFarm,
        CellStateVariant::Seller,
        CellStateVariant::Back,
    ];
    match c.data {
        celldata::CellStateData::Resource(crate::resource::Resource::Pure(r)) => {
            if resource::get(resource::ResourceType::Wood, r) > 0 {
                res.push(CellStateVariant::WoodCutter);
                res
            } else {
                res
            }
        }
        _ => res,
    }
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

pub(crate) fn explore_able() -> Vec<CellStateVariant> {
    vec![CellStateVariant::Unused]
}
