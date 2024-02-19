// utilities
pub mod utils;
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
use bob_lib::tracker::{Goal, GoalTracker, GoalType};
use midgard::world_generator::{ContentsRadii, WorldGeneratorParameters};
use sense_and_find_by_rustafariani::{Action, Lssf};
use spyglass::spyglass::Spyglass;
use OhCrab_collection::collection::CollectTool;

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
    INIT = 0,
    CHILL, // wanders around for a while, exploring the world using `spyglass` to get inspired for her next masterpiece
    DETECT, // use `sense&find` to find stuff
    GATHER, // use `collect_tool` to gather stuff
    PAINT, // use `giotto_tool` to paint on the map
    DIE,  // as most artist do, the robot gracefully terminates its existance
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
            String::default(),
            String::default(),
            GoalType::GetItems,
            Some(Content::Tree(0)),
            20,
        ));

        ArtemisIA {
            ui,
            robot: Robot::new(),
            wrld_size,
            state: RobotState::INIT,
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
            Ok(RobotState::CHILL)
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

        let _ = spyglass.new_discover(self, world);

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
            Ok(RobotState::CHILL)
        } else {
            Ok(RobotState::DETECT)
        }
    }

    pub fn do_detect(&mut self, world: &mut World) -> Result<RobotState, String> {
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
                return Ok(RobotState::CHILL);
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
            return Ok(RobotState::GATHER);
        }

        Ok(RobotState::DETECT)
    }

    pub fn do_gather(&mut self, world: &mut World) -> Result<RobotState, String> {
        let rocks = CollectTool::collect_instantly_reachable(self, world, &Content::Rock(0));
        let trees = CollectTool::collect_instantly_reachable(self, world, &Content::Tree(0));

        if let Ok(count) = rocks {
            self.goal_tracker
                .update_manual(GoalType::GetItems, Some(Content::Tree(0)), count)
        }

        if let Ok(count) = trees {
            self.goal_tracker
                .update_manual(GoalType::GetItems, Some(Content::Tree(0)), count)
        }

        if rocks.is_ok() || trees.is_ok() {
            if self.goal_tracker.get_completed_number() > 0 {
                Ok(RobotState::PAINT)
            } else {
                Ok(RobotState::DETECT)
            }
        } else {
            Err("\nARTEMIS-IA: failed to gather\n".to_string())
        }
    }

    pub fn do_paint(&mut self, world: &mut World) -> Result<RobotState, String> {
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
                    Ok(RobotState::DIE)
                } else {
                    match s {
                        GiottoStatus::Finished
                        | GiottoStatus::FinishedCell
                        | GiottoStatus::WaitingForEnergy => Ok(RobotState::CHILL),
                        GiottoStatus::WaitingForMaterials => Ok(RobotState::DETECT),
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
            Ok(RobotState::DIE)
        } else {
            Err("\nARTEMIS-IA: failed to stop\n".to_string())
        }
    }

    pub fn run(&mut self, world: &mut World) {
        print_debug("why are you running? why-are-you-running?");
        let new_state;

        match &self.state {
            RobotState::INIT => new_state = self.do_init(),
            RobotState::CHILL => new_state = self.do_chill(world),
            RobotState::DETECT => new_state = self.do_detect(world),
            RobotState::GATHER => new_state = self.do_gather(world),
            RobotState::PAINT => new_state = self.do_paint(world),
            RobotState::DIE => new_state = self.do_stop(),
        }

        match new_state {
            Ok(new) => {
                print_debug(format!("state transition: {:?} -> {:?}", self.state, new).as_str());
                match (&self.state, &new) {
                    (RobotState::INIT, RobotState::CHILL)
                    | (RobotState::CHILL, RobotState::CHILL)
                    | (RobotState::CHILL, RobotState::DETECT)
                    | (RobotState::DETECT, RobotState::DETECT)
                    | (RobotState::DETECT, RobotState::GATHER)
                    | (RobotState::DETECT, RobotState::CHILL)
                    | (RobotState::GATHER, RobotState::DETECT)
                    | (RobotState::GATHER, RobotState::PAINT)
                    | (RobotState::PAINT, RobotState::CHILL)
                    | (RobotState::PAINT, RobotState::DETECT)
                    | (RobotState::PAINT, RobotState::DIE) => self.state = new,
                    _ => panic!("Invalid state transition"),
                }
            }
            Err(e) => print_debug(format!("ERROR: {}\n", e).as_str()),
        }
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