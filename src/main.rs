use midgard::world_generator::{WorldGenerator, WorldGeneratorParameters};
use robotics_lib::runner::Runner;
use rust_eze_ai_artemisia::ArtemisIA;

fn main() {
    let world_size = 200;
    let params = WorldGeneratorParameters {
        seed: 15,                    // fixed seed
        world_size,                  // smaller world
        amount_of_rivers: Some(1.2), // more rivers
        amount_of_streets: None,     // disable streets
        ..Default::default()         // the rest of the parameters keep their default value
    };
    let mut world_generator = WorldGenerator::new(params);

    let artemis = Box::new(ArtemisIA::new(world_size));
    let run = Runner::new(artemis, &mut world_generator);

    match run {
        Ok(mut robot) => {
            for i in 0..300 {
                println!("\nGame tick {}", i);
                let _ = robot.game_tick();
            }
        }
        Err(e) => println!("LibError {:?}", e),
    }

    println!("in the end, it all ends.");
}
