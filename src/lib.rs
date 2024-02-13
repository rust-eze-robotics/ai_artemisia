pub mod utils;

// std libs
use rand::Rng;
use std::sync::Mutex;

// robotics_lib
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::runner::{backpack::BackPack, Robot, Runnable};
use robotics_lib::world::{coordinates::Coordinate, World};

// tools
use giotto_tool::tools::coordinate::GiottoCoordinate;
use giotto_tool::tools::debugger::GiottoDebug;
use giotto_tool::tools::drawer::Drawer;
use giotto_tool::tools::image::GiottoImage;
use giotto_tool::tools::status::GiottoStatus;
use spyglass::spyglass::Spyglass;

enum RobotState {
    INIT = 0,
    CHILL, // wanders around for a while, exploring the world using `spyglass` to get inspired for her next masterpiece
    COLLECT, // use `sense&find` to find collect stuff
    PAINT, // use `giotto_tool` to paint on the map
    STOP,  // as most artist do, the robot gracefully terminates its existance
    NUM_STATES,
}

struct ArtemisIA {
    robot: Robot,
    wrld_size: usize,
    state: RobotState,
    countdown: i32,
}

impl ArtemisIA {
    fn new() -> Self {
        ArtemisIA {
            robot: Robot::new(),
            wrld_size: 500,
            state: RobotState::INIT,
            countdown: 0,
        }
    }

    // state functions
    fn do_init(&mut self) -> RobotState {
        let mut rng = rand::thread_rng();
        self.countdown = rng.gen_range(0..=13);

        RobotState::CHILL
    }
    fn do_chill(&mut self) -> RobotState {
        // wanders around for a while, explore with spyglass, relax and get inspired
        let spyglass = Spyglass::new_default(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
            7,
            self.wrld_size,
        );

        RobotState::COLLECT
    }
    fn do_collect(&mut self) -> RobotState {
        // TODO: sense&find to collect stuff

        RobotState::PAINT
    }
    fn do_paint(&mut self, world: &mut World) -> RobotState {
        // pain't, create art from pain (and stuff you collected)
        let img: GiottoImage;

        if self.countdown > 0 {
            img = utils::rand_img();
            self.countdown -= 1;
        } else {
            img = utils::build_img("res/img/fontana_concettospaziale.png");
        }

        let coord = GiottoCoordinate::new(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
        );
        let painter = Mutex::new(Drawer::new(img, coord, GiottoDebug::new(false)));

        let paint_state = painter
            .lock()
            .unwrap()
            .draw_until_possible(self, world, false);

        match paint_state {
            Ok(s) => {
                if self.countdown <= 0 && s == GiottoStatus::Finished {
                    RobotState::STOP
                } else {
                    match s {
                        GiottoStatus::Finished
                        | GiottoStatus::FinishedCell
                        | GiottoStatus::WaitingForEnergy => RobotState::CHILL,
                        GiottoStatus::WaitingForMaterials => RobotState::COLLECT,
                    }
                }
            }
            Err(_) => RobotState::CHILL,
        }
    }
    fn do_stop(&mut self) -> RobotState {
        // 'peg out', 'die', 'stop working'
        // grand sortie: when the robot paints the amount of paintings, assigned during the init state,
        // it gracefully pegs out, and as a last performance the whole map gets covered in lava (red canva, inspired to Fontana's "Concetto Spaziale")

        // TODO: gracefully terminate the robot, find out how to do it ;)

        RobotState::STOP
    }

    fn run(&mut self, world: &mut World) {
        let new_state;

        match &self.state {
            RobotState::INIT => new_state = self.do_init(),
            RobotState::CHILL => new_state = self.do_chill(),
            RobotState::COLLECT => new_state = self.do_collect(),
            RobotState::PAINT => new_state = self.do_paint(world),
            RobotState::STOP => new_state = self.do_stop(),
            _ => panic!("Invalid state"),
        }

        match (&self.state, &new_state) {
            (RobotState::INIT, RobotState::CHILL)
            | (RobotState::CHILL, RobotState::COLLECT)
            | (RobotState::COLLECT, RobotState::PAINT)
            | (RobotState::PAINT, RobotState::CHILL)
            | (RobotState::PAINT, RobotState::COLLECT)
            | (RobotState::PAINT, RobotState::STOP) => self.state = new_state,
            _ => panic!("Invalid state transition"),
        }
    }
}

impl Runnable for ArtemisIA {
    fn process_tick(&mut self, world: &mut robotics_lib::world::World) {
        self.run(world);
    }

    fn handle_event(&mut self, event: Event) {
        println!("{:?}", event);
    }
    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }
    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }
    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }
    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }
    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }
}
