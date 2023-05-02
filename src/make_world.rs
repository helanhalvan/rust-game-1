use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, Seed, Size, Step};
use worldgen::world::tile::{Constraint, ConstraintType};
use worldgen::world::{Tile, World};

pub fn test() {
    let noise = PerlinNoise::new();

    let nm1 = NoiseMap::new(noise)
        .set(Seed::of("blarg"))
        .set(Step::of(0.005, 0.005));

    let nm2 = NoiseMap::new(noise)
        .set(Seed::of("Hello!asdas"))
        .set(Step::of(0.05, 0.05));

    let nm = Box::new(nm1 + nm2 * 3);

    let world = World::new()
        .set(Size::of(80, 50))
        // Water
        .add(Tile::new('~').when(worldgen::constraint!(nm.clone(), < -0.1)))
        // Grass
        .add(Tile::new(',').when(worldgen::constraint!(nm.clone(), < 0.45)))
        // Mountains
        .add(Tile::new('^').when(worldgen::constraint!(nm.clone(), > 0.8)))
        // Hills
        .add(Tile::new('n'));

    for row in world.generate(0, 0).iter() {
        for val in row.iter() {
            for c in val.iter() {
                print!("{}", c);
            }

            println!("");
        }

        println!("");
    }
}
