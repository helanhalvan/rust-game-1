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

impl CellGen for celldata::CellState {
    type GenContext = GenContext;

    fn new_chunk(p: hexgrid::Pos, c: &mut Self::GenContext) -> hexgrid::Matrix<Self> {
        let (chunk, _) = hexgrid::to_chunk_keys(p);
        let filename = format!("{}_{}.png", chunk.x, chunk.y);
        let map = PlaneMapBuilder::<_, 2>::new(c.main_noise.clone())
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
        map.write_to_file(&filename);
        let mut ret = vec![];
        for i in 0..CHUNK_SIZE {
            let mut row = vec![];
            for j in 0..CHUNK_SIZE {
                let wood = (map.get_value(i, j) * 6.0).round().clamp(0.0, 6.0) as i32;
                row.push(resource::new_pure_stockpile(
                    celldata::CellStateVariant::Hidden,
                    HashMap::from([(resource::ResourceType::Wood, wood)]),
                ))
            }
            ret.push(row)
        }
        ret
    }
}

#[derive(Clone)]
pub struct GenContext {
    main_noise: Fbm<Worley>,
}

pub fn new() -> hexgrid::Hexgrid<celldata::CellState, GenContext> {
    let seed = if let Ok(n) = SystemTime::now().duration_since(UNIX_EPOCH) {
        (n.as_nanos() & u32::MAX as u128) as u32
    } else {
        dbg!("TIME PANIC");
        0
    };
    dbg!(seed);
    hexgrid::new(
        GenContext {
            main_noise: Fbm::<Worley>::new(seed),
        },
        celldata::unit_state(celldata::CellStateVariant::OutOfBounds),
    )
}

pub fn test() {
    let fbm = Fbm::<Perlin>::default();

    PlaneMapBuilder::<_, 2>::new(fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
        .write_to_file("fbm_perlin.png");

    let fbm = Fbm::<Worley>::default();

    PlaneMapBuilder::<_, 2>::new(fbm.clone())
        .set_size(1000, 1000)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
        .write_to_file("fbm_worley1.png");

    PlaneMapBuilder::<_, 2>::new(fbm.clone())
        .set_size(1000, 1000)
        .set_x_bounds(0.0, 5.0)
        .set_y_bounds(0.0, 5.0)
        .build()
        .write_to_file("fbm_worley2.png");
    PlaneMapBuilder::<_, 2>::new(fbm.clone())
        .set_size(1000, 1000)
        .set_x_bounds(-5.0, 0.0)
        .set_y_bounds(-5.0, 0.0)
        .build()
        .write_to_file("fbm_worley3.png");
    PlaneMapBuilder::<_, 2>::new(fbm.clone())
        .set_size(1000, 1000)
        .set_x_bounds(-5.0, 0.0)
        .set_y_bounds(0.0, 5.0)
        .build()
        .write_to_file("fbm_worley4.png");

    let fbm = Fbm::<Fbm<Perlin>>::default();

    PlaneMapBuilder::<_, 2>::new(fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-5.0, 5.0)
        .set_y_bounds(-5.0, 5.0)
        .build()
        .write_to_file("fbm_fbm_perlin.png");
}
