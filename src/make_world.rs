use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use noise::{utils::*, Fbm, Perlin, Worley};

use crate::{
    celldata,
    hexgrid::{self, CellGen, CHUNK_SIZE},
    resource,
};

pub(crate) const MAX_WOOD: i32 = 6;
pub(crate) const SCALING_WOOD: i32 = 16;
pub(crate) const MAX_WOOD_RANGE: i32 = MAX_WOOD * SCALING_WOOD;

impl CellGen for celldata::CellState {
    type GenContext = GenContext;

    fn new_chunk(p: hexgrid::Pos, c: &mut Self::GenContext) -> hexgrid::Chunk<Self> {
        let (chunk, _) = hexgrid::to_chunk_keys(p);
        //let filename = format!("{}_{}.png", chunk.x, chunk.y);
        let wood_map = chunk_plane(c.wood_noise.clone(), chunk);
        let ore_map = chunk_plane(c.ore_noise.clone(), chunk);
        //map.write_to_file(&filename);
        let mut ret = vec![];
        for i in 0..CHUNK_SIZE {
            let mut row = vec![];
            for j in 0..CHUNK_SIZE {
                let wood = (wood_map.get_value(i, j) * MAX_WOOD_RANGE as f64)
                    .round()
                    .clamp(0.0, MAX_WOOD_RANGE as f64) as i32;
                let ore = (ore_map.get_value(i, j) * MAX_WOOD_RANGE as f64)
                    .round()
                    .clamp(0.0, MAX_WOOD_RANGE as f64) as i32;
                let new = resource::new_pure_stockpile(
                    celldata::CellStateVariant::Hidden,
                    HashMap::from([
                        (resource::ResourceType::Wood, wood),
                        (resource::ResourceType::IronOre, ore),
                    ]),
                );
                row.push(new)
            }
            ret.push(row)
        }
        hexgrid::matrix::new(CHUNK_SIZE, CHUNK_SIZE, ret)
    }
}

fn chunk_plane<T: noise::NoiseFn<f64, 2>>(src: Fbm<T>, chunk: hexgrid::Pos) -> NoiseMap {
    let map = PlaneMapBuilder::<_, 2>::new(src)
        .set_size(hexgrid::CHUNK_SIZE, hexgrid::CHUNK_SIZE)
        .set_x_bounds(
            chunk.x as f64 * 0.01,
            (chunk.x + CHUNK_SIZE as i32) as f64 * 0.01,
        )
        .set_y_bounds(
            chunk.y as f64 * 0.01,
            (chunk.y + CHUNK_SIZE as i32) as f64 * 0.01,
        )
        .build();
    map
}

#[derive(Clone)]
pub(crate) struct GenContext {
    wood_noise: Fbm<Fbm<Perlin>>,
    ore_noise: Fbm<Worley>,
}

pub(crate) fn new() -> hexgrid::Hexgrid<celldata::CellState, GenContext> {
    let seed = if let Ok(n) = SystemTime::now().duration_since(UNIX_EPOCH) {
        (n.as_nanos() & u32::MAX as u128) as u32
    } else {
        dbg!("TIME PANIC");
        0
    };
    dbg!(seed);
    hexgrid::new(
        GenContext {
            wood_noise: Fbm::<Fbm<Perlin>>::new(seed),
            ore_noise: Fbm::<Worley>::new(seed + 1),
        },
        celldata::unit_state(celldata::CellStateVariant::OutOfBounds),
    )
}
