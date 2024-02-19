use midgard::world_generator::{WorldGenerator, WorldGeneratorParameters};
use robotics_lib::{event::events::Event, runner::Runner, world::World};
use rusteze_ai_artemisia::ArtemisIA;
use ui_lib::RunnableUi;

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

    let ui = Box::new(UI{});

    let artemis = Box::new(ArtemisIA::new(world_size, ui));
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

struct UI {}
#[allow(unused_variables)]
impl RunnableUi for UI {
    fn process_tick(&mut self, world: &mut World){}
    fn handle_event(&mut self, event: Event){}
}