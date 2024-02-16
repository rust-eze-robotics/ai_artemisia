pub mod utils;
// std libs
use rand::Rng;
use std::collections::VecDeque;
// robotics_lib
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::robot_map,
    runner::{backpack::BackPack, Robot, Runnable},
    world::{
        coordinates::Coordinate,
        tile::{Content, Tile},
        World,
    },
};
// tools
use giotto_tool::tools::{
    coordinate::GiottoCoordinate, debugger::GiottoDebug, drawer::Drawer, image::GiottoImage,
    status::GiottoStatus,
};
use sense_and_find_by_rustafariani::Lssf;
use spyglass::spyglass::{Spyglass, SpyglassResult};
use OhCrab_collection::collection::CollectTool;

#[derive(Debug, PartialEq)]
pub enum RobotState {
    INIT = 0,
    CHILL, // wanders around for a while, exploring the world using `spyglass` to get inspired for her next masterpiece
    GATHER, // use `sense&find` to find collect stuff
    PAINT, // use `giotto_tool` to paint on the map
    STOP,  // as most artist do, the robot gracefully terminates its existance
           // NUM_STATES,
}

pub struct ArtemisIA {
    robot: Robot,
    wrld_size: usize,
    state: RobotState,
    countdown: i32,
    rocks: VecDeque<(usize, usize)>,
    trees: VecDeque<(usize, usize)>,
    // event_queue: Rc<RefCell<Vec<Event>>>,
}

impl ArtemisIA {
    // pub fn new(wrld_size: usize, event_queue: Rc<RefCell<Vec<Event>>>) -> Self {

    pub fn new(wrld_size: usize) -> Self {
        println!("\nARTEMIS-IA: ArtemisIA created\n");

        ArtemisIA {
            robot: Robot::new(),
            wrld_size,
            state: RobotState::INIT,
            countdown: 1,
            rocks: VecDeque::new(),
            trees: VecDeque::new(),
            // event_queue: Rc::new(RefCell::new(Vec::new())),
        }
    }

    // state functions
    pub fn do_init(&mut self) -> Result<RobotState, String> {
        println!("\nARTEMIS-IA: in(n)it\n");

        let mut rng = rand::thread_rng();
        self.countdown = rng.gen_range(0..=13);

        if true {
            Ok(RobotState::CHILL)
        } else {
            Err("\nARTEMIS-IA: failed to init\n".to_string())
        }
    }

    pub fn do_chill(&mut self, world: &mut World) -> Result<RobotState, String> {
        // wanders around for a while, explore with spyglass, relax and get inspired for her next masterpiece

        println!("\nARTEMIS-IA: chilling");

        let mut spyglass = Spyglass::new(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
            self.wrld_size,
            self.wrld_size,
            Some(self.robot.energy.get_energy_level()),
            true,
            (self.wrld_size / 2) as f64,
            |tile: &Tile| matches!(tile.clone().content, Content::Rock(_)),
        );
        match spyglass.new_discover(self, world) {
            SpyglassResult::Complete => {
                println!("\nARTEMIS-IA: chilled enough, saw lots of ROCKS, time to gather");
            }
            SpyglassResult::Failed(_) => {
                return Err("\nARTEMIS-IA: oh no! our spyglass...is broken!\n".to_string());
            }
            _ => {
                println!("\nARTEMIS-IA: chilling, looking for ROCKS");
            }
        }

        spyglass.set_stop_when(|tile: &Tile| matches!(tile.clone().content, Content::Tree(_)));
        match spyglass.new_discover(self, world) {
            SpyglassResult::Complete => {
                println!("\nARTEMIS-IA: chilled enough, saw lots of TREES, time to gather\n");
            }
            SpyglassResult::Failed(_) => {
                return Err("\nARTEMIS-IA: oh no! our spyglass...is broken!\n".to_string());
            }
            _ => {
                println!("\nARTEMIS-IA: chilling, looking for TREES");
            }
        }

        println!("\nARTEMIS-IA: spyglass complete, time to lssf\n");

        let map = robot_map(world).unwrap();

        let mut lssf = Lssf::new();
        lssf.update_map(&map);
        let _ = lssf.update_cost(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
        );

        self.rocks.extend(lssf.get_content_vec(&Content::Rock(0)));
        self.trees.extend(lssf.get_content_vec(&Content::Tree(0)));

        println!("\nARTEMIS-IA: lssf lessgo\n");

        if !self.rocks.is_empty() && !self.trees.is_empty() {
            Ok(RobotState::GATHER)
        } else {
            Ok(RobotState::CHILL)
        }
    }

    pub fn do_gather(&mut self, world: &mut World) -> Result<RobotState, String> {
        println!("\nARTEMIS-IA: gathering");

        let map = robot_map(world).unwrap();

        let mut gathered = false;

        if let Some((row, col)) = self.rocks.pop_front() {
            if let Some(tile) = map[row][col].as_ref() {
                let content = &tile.content;

                if let Ok(_) =
                    CollectTool::collect_content(self, world, content, usize::MAX, self.robot.energy.get_energy_level())
                {
                    gathered = true;
                }
            }
        }

        println!("\nARTEMIS-IA: we have rocks, lets get some trees");

        if let Some((row, col)) = self.trees.pop_front() {
            if let Some(tile) = map[row][col].as_ref() {
                let content = &tile.content;

                if let Ok(_) =
                CollectTool::collect_content(self, world, content, usize::MAX, self.robot.energy.get_energy_level())
                {
                    gathered = true;
                }
            }
        }

        println!("\nARTEMIS-IA: we have trees, lets paint");

        if gathered {
                Ok(RobotState::PAINT)
        } else {
            if self.rocks.is_empty() || self.trees.is_empty() {
                Ok(RobotState::GATHER)
            } else {
                Err("\nARTEMIS-IA: failed to gather\n".to_string())
            }
        }
    }

    pub fn do_paint(&mut self, world: &mut World) -> Result<RobotState, String> {
        // pain't, create art from pain (and stuff you collected)
        let img: GiottoImage;

        println!("\nARTEMIS-IA: painting\n");

        if self.countdown > 0 {
            img = utils::rand_img();
            self.countdown -= 1;
        } else {
            // img = utils::build_img("res/img/fontana_concettospaziale.png");
            img = utils::build_img("res/img/meow.png");
        }

        let coord = GiottoCoordinate::new(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
        );
        let mut painter = Drawer::new(img, coord, GiottoDebug::new(false));

        let paint_state = painter.draw_until_possible(self, world, false);

        println!("\nARTEMIS-IA: painting: {:?}", paint_state);

        match paint_state {
            Ok(s) => {
                if self.countdown <= 0 && s == GiottoStatus::Finished {
                    Ok(RobotState::STOP)
                } else {
                    match s {
                        GiottoStatus::Finished
                        | GiottoStatus::FinishedCell
                        | GiottoStatus::WaitingForEnergy => Ok(RobotState::CHILL),
                        GiottoStatus::WaitingForMaterials => Ok(RobotState::GATHER),
                    }
                }
            }
            Err(_) => Err("\nARTEMIS-IA: painting failed\n".to_string()),
        }
    }

    pub fn do_stop(&mut self) -> Result<RobotState, String> {
        // 'peg out', 'die', 'stop working'
        // grand sortie: when the robot paints the amount of paintings, assigned during the init state,
        // it gracefully pegs out, and as a last performance the whole map gets covered in lava (red canva, inspired to Fontana's "Concetto Spaziale")

        // TODO: gracefully terminate the robot, find out how to do it ;)

        println!("\nARTEMIS-IA: ok i'll die now, bye!\n");

        if true {
            Ok(RobotState::STOP)
        } else {
            Err("\nARTEMIS-IA: failed to stop\n".to_string())
        }
    }

    pub fn run(&mut self, world: &mut World) {
        println!("\nARTEMIS-IA: why are you running? why-are-you-running?");
        let new_state;

        match &self.state {
            RobotState::INIT => new_state = self.do_init(),
            RobotState::CHILL => new_state = self.do_chill(world),
            RobotState::GATHER => new_state = self.do_gather(world),
            RobotState::PAINT => new_state = self.do_paint(world),
            RobotState::STOP => new_state = self.do_stop(),
        }

        match new_state {
            Ok(new) => {
                println!("ARTEMIS-IA: state transition: {:?} -> {:?}\n", self.state, new);
                match (&self.state, &new) {
                    (RobotState::INIT, RobotState::CHILL)
                    | (RobotState::CHILL, RobotState::GATHER)
                    | (RobotState::GATHER, RobotState::GATHER)
                    | (RobotState::GATHER, RobotState::PAINT)
                    | (RobotState::PAINT, RobotState::CHILL)
                    | (RobotState::PAINT, RobotState::GATHER)
                    | (RobotState::PAINT, RobotState::STOP) => self.state = new,
                    _ => panic!("Invalid state transition"),
                }
            },
            Err(e) => println!("\nARTEMIS-IA: error: {}\n", e),
        }

        
    }
}

impl Runnable for ArtemisIA {
    fn process_tick(&mut self, world: &mut World) {
        self.run(world);
    }

    fn handle_event(&mut self, event: Event) {
        // self.event_queue.borrow_mut().push(event);
        println!("event: {:?}", event);
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
