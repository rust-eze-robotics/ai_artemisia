pub mod utils;
use bob_lib::tracker::{Goal, GoalTracker, GoalType};
use midgard::params::{ContentsRadii, WorldGeneratorParameters};
// std libs
use rand::Rng;
use std::collections::VecDeque;
use ui_lib::RunnableUi;
use utils::print_debug;
// robotics_lib
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{go, robot_map, robot_view, teleport, Direction},
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
use sense_and_find_by_Rustafariani::{Action, Lssf};
use spyglass::spyglass::{Spyglass, SpyglassResult};
use OhCrab_collection::collection::{CollectTool, LibErrorExtended};

pub fn get_world_generator_parameters() -> WorldGeneratorParameters {
    WorldGeneratorParameters {
        time_progression_minutes: 60,
        contents_radii: ContentsRadii {
            rocks_in_plains: 2,
            rocks_in_hill: 2,
            rocks_in_mountain: 2,
            trees_in_forest: 2,
            trees_in_hill: 2,
            trees_in_mountain: 2,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[derive(Debug, PartialEq)]
pub enum RobotState {
    Init = 0,
    Chill, // wanders around for a while, exploring the world using `spyglass` to get inspired for her next masterpiece
    Gather, // use `sense&find` to find stuff
    Collect, // use CollectTool to collect stuff
    Paint, // use `giotto_tool` to paint on the map
    Stop,  // as most artist do, the robot gracefully terminates its existance
           // NUM_STATES,
}

pub struct ArtemisIA {
    robot: Robot,
    wrld_size: usize,
    ui: Box<dyn RunnableUi>,
    state: RobotState,
    countdown: i32,
    contents: VecDeque<(usize, usize)>,
    actions: VecDeque<Action>,
    goal_tracker: GoalTracker,
}

impl ArtemisIA {
    // pub fn new(wrld_size: usize, event_queue: Rc<RefCell<Vec<Event>>>) -> Self {

    pub fn new(wrld_size: usize, ui: Box<dyn RunnableUi>) -> Self {
        let mut goal_tracker = GoalTracker::new();
        goal_tracker.add_goal(Goal::new(
            String::from("Contents"),
            String::default(),
            GoalType::GetItems,
            None,
            20,
        ));

        ArtemisIA {
            ui,
            robot: Robot::new(),
            wrld_size,
            state: RobotState::Init,
            countdown: 1,
            contents: VecDeque::new(),
            actions: VecDeque::new(),
            goal_tracker,
        }
    }

    // state functions
    pub fn do_init(&mut self) -> Result<RobotState, String> {
        let mut rng = rand::thread_rng();
        self.countdown = rng.gen_range(0..=13);

        if true {
            Ok(RobotState::Chill)
        } else {
            Err("\nARTEMIS-IA: failed to init\n".to_string())
        }
    }

    pub fn do_chill(&mut self, world: &mut World) -> Result<RobotState, String> {
        // wanders around for a while, explore with spyglass, relax and get inspired for her next masterpiece

        let mut spyglass = Spyglass::new(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
            10,
            self.wrld_size,
            Some(self.robot.energy.get_energy_level()),
            true,
            (self.wrld_size / 2) as f64,
            |tile: &Tile| tile.content == Content::Rock(0) || tile.content == Content::Tree(0),
        );

        let result = spyglass.new_discover(self, world);

        let map = robot_map(world).unwrap();

        let mut lssf = Lssf::new();
        lssf.update_map(&map);
        let _ = lssf.update_cost(
            self.robot.coordinate.get_row(),
            self.robot.coordinate.get_col(),
        );

        self.contents = VecDeque::new();
        self.contents
            .extend(lssf.get_content_vec(&Content::Rock(0)).iter());
        self.contents
            .extend(lssf.get_content_vec(&Content::Tree(0)).iter());

        if self.contents.is_empty() {
            Ok(RobotState::Chill)
        } else {
            Ok(RobotState::Gather)
        }
    }

    pub fn do_gather(&mut self, world: &mut World) -> Result<RobotState, String> {
        if self.actions.is_empty() {
            if let Some((row, col)) = self.contents.pop_front() {
                let map = robot_map(world).unwrap();

                let mut lssf = Lssf::new();
                lssf.update_map(&map);
                let _ = lssf.update_cost(
                    self.robot.coordinate.get_row(),
                    self.robot.coordinate.get_col(),
                );

                let result = lssf.get_action_vec(row, col);

                if let Ok(vec) = result {
                    self.actions = VecDeque::new();
                    self.actions.extend(vec.into_iter());
                }
            } else {
                return Ok(RobotState::Chill);
            }
        }

        if self.actions.len() > 1 {
            if let Some(action) = self.actions.pop_front() {
                match action {
                    Action::East => {
                        let _ = go(self, world, Direction::Right);
                        robot_view(self, world);
                    }
                    Action::South => {
                        let _ = go(self, world, Direction::Down);
                        robot_view(self, world);
                    }
                    Action::West => {
                        let _ = go(self, world, Direction::Left);
                        robot_view(self, world);
                    }
                    Action::North => {
                        let _ = go(self, world, Direction::Up);
                        robot_view(self, world);
                    }
                    Action::Teleport(row, col) => {
                        let _ = teleport(self, world, (row, col));
                    }
                }
            }
        }

        if self.actions.len() == 1 {
            self.actions = VecDeque::new();
            return Ok(RobotState::Collect);
        }

        Ok(RobotState::Gather)
    }

    pub fn do_collect(&mut self, world: &mut World) -> Result<RobotState, String> {
        let rocks = CollectTool::collect_instantly_reachable(self, world, &Content::Rock(0));
        let trees = CollectTool::collect_instantly_reachable(self, world, &Content::Tree(0));

        if let Ok(count) = rocks {
            self.goal_tracker
                .update_manual(GoalType::GetItems, None, count)
        }

        if let Ok(count) = trees {
            self.goal_tracker
                .update_manual(GoalType::GetItems, None, count)
        }

        if rocks.is_ok() || trees.is_ok() {
            if self.goal_tracker.get_completed_number() > 0 {
                Ok(RobotState::Paint)
            } else {
                Ok(RobotState::Gather)
            }
        } else {
            Err("\nARTEMIS-IA: failed to gather\n".to_string())
        }
    }

    pub fn do_paint(&mut self, world: &mut World) -> Result<RobotState, String> {
        return Ok(RobotState::Paint);
        // pain't, create art from pain (and stuff you collected)
        let img: GiottoImage;

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

        match paint_state {
            Ok(s) => {
                if self.countdown <= 0 && s == GiottoStatus::Finished {
                    Ok(RobotState::Stop)
                } else {
                    match s {
                        GiottoStatus::Finished
                        | GiottoStatus::FinishedCell
                        | GiottoStatus::WaitingForEnergy => Ok(RobotState::Chill),
                        GiottoStatus::WaitingForMaterials => Ok(RobotState::Gather),
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

        if true {
            self.handle_event(Event::Terminated);
            Ok(RobotState::Stop)
        } else {
            Err("\nARTEMIS-IA: failed to stop\n".to_string())
        }
    }

    pub fn run(&mut self, world: &mut World) {
        let new_state;

        println!("{:?}", self.state);

        match &self.state {
            RobotState::Init => new_state = self.do_init(),
            RobotState::Chill => new_state = self.do_chill(world),
            RobotState::Gather => new_state = self.do_gather(world),
            RobotState::Collect => new_state = self.do_collect(world),
            RobotState::Paint => new_state = self.do_paint(world),
            RobotState::Stop => new_state = self.do_stop(),
        }

        if let Ok(state) = new_state {
            self.state = state;
        }

        // :)
        // match new_state {
        //     Ok(new) => {
        //         print_debug(format!("state transition: {:?} -> {:?}", self.state, new).as_str());
        //         match (&self.state, &new) {
        //             (RobotState::Init, RobotState::Chill)
        //             | (RobotState::Chill, RobotState::Chill)
        //             | (RobotState::Chill, RobotState::Gather)
        //             | (RobotState::Gather, RobotState::Gather)
        //             | (RobotState::Gather, RobotState::Paint)
        //             | (RobotState::Paint, RobotState::Chill)
        //             | (RobotState::Paint, RobotState::Gather)
        //             | (RobotState::Paint, RobotState::Stop) => self.state = new,
        //             _ => panic!("Invalid state transition"),
        //         }
        //     }
        //     Err(e) => print_debug(format!("ERROR: {}\n", e).as_str()),
        // }
    }
}

impl Runnable for ArtemisIA {
    fn process_tick(&mut self, world: &mut World) {
        self.run(world);
        self.ui.process_tick(world);
    }

    fn handle_event(&mut self, event: Event) {
        self.ui.handle_event(event);
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
