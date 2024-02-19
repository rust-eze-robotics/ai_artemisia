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
use bob_lib::tracker::{Goal, GoalTracker, GoalType};
use giotto_tool::tools::{
    coordinate::GiottoCoordinate, debugger::GiottoDebug, drawer::Drawer, image::GiottoImage,
    status::GiottoStatus,
};
use midgard::params::{ContentsRadii, WorldGeneratorParameters};
use sense_and_find_by_Rustafariani::{Action, Lssf};
use spyglass::spyglass::Spyglass;
use OhCrab_collection::collection::CollectTool;

/// This function returns the WorldGeneratorParameters for ArtemisIA
/// # Returns
/// * a WorldGeneratorParameters
/// # Example
/// ```
/// let wgp = get_world_generator_parameters();
/// ```
/// # Note
/// * The function returns a WorldGeneratorParameters with the hardcoded values
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

/// This enum represents the states of the robot
/// # Example
/// ```
/// let state = RobotState::INIT;
/// ```
/// # Note
/// * The states are INIT, CHILL, DETECT, GATHER, PAINT, DIE
/// * The states are used to control the robot's behavior, because it's built as a FSM
#[derive(Debug, PartialEq)]
pub enum RobotState {
    INIT = 0,
    CHILL, // wanders around for a while, exploring the world using `spyglass` to get inspired for her next masterpiece
    DETECT, // use `sense&find` to find stuff
    GATHER, // use `collect_tool` to gather stuff
    PAINT, // use `giotto_tool` to paint on the map
    DIE,   // as most artist do, the robot gracefully terminates its existance
           // NUM_STATES,
}

/// This struct represents the ArtemisIA robot
/// # Note
/// * The struct uses the Runnable trait to interact with the world
/// * The struct uses the RunnableUi trait to interact with the UI
/// * The struct uses the Robot struct to store the robot's data
/// * The struct uses the RobotState enum to store the robot's state
/// * The struct uses the GoalTracker struct to store the robot's goals
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
    // constructor
    /// This function returns a new ArtemisIA
    /// # Arguments
    /// * `wrld_size` - The size of the world
    /// * `ui` - A Box<dyn RunnableUi>
    /// # Returns
    /// * a ArtemisIA
    /// # Example
    /// ```
    /// let artemis = ArtemisIA::new(10, Box::new(ui));
    /// ```
    /// # Note
    /// * The function returns a new ArtemisIA with the given parameters set
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

    /// This function handles the INIT state
    /// # Returns
    /// * a Result<RobotState, String>
    /// # Example
    /// ```
    /// let state = artemis.do_init();
    /// ```
    /// # Note
    /// * The function returns a Result with the new state or an error message
    /// * The function sets the countdown to a random number between 0 and 13, which is the number of paintings the robot will make before going into the DIE state
    /// * The function returns CHILL if the countdown is greater than 0, otherwise it returns an error
    pub fn do_init(&mut self) -> Result<RobotState, String> {
        let mut rng = rand::thread_rng();
        self.countdown = rng.gen_range(0..=13);

        if true {
            Ok(RobotState::CHILL)
        } else {
            Err("\nARTEMIS-IA: failed to init\n".to_string())
        }
    }

    /// This function handles the CHILL state
    /// # Arguments
    /// * `world` - A mutable reference to the World
    /// # Returns
    /// * a Result<RobotState, String>
    /// # Example
    /// ```
    /// let state = artemis.do_chill(world);
    /// ```
    /// # Note
    /// * The function uses the Spyglass to explore the world and get inspired for the next masterpiece
    /// * The function uses the Lssf to look around and find contents in the world
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

    /// This function handles the GATHER state
    /// # Arguments
    /// * `world` - A mutable reference to the World
    /// # Returns
    /// * a Result<RobotState, String>
    /// # Example
    /// ```
    /// let state = artemis.do_gather(world);
    /// ```
    /// # Note
    /// * The function uses the CollectTool to gather rocks and trees
    /// * The function updates the goal tracker with the number of rocks and trees gathered
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

    /// This function handles the PAINT state
    /// # Arguments
    /// * `world` - A mutable reference to the World
    /// # Returns
    /// * a Result<RobotState, String>
    /// # Example
    /// ```
    /// let state = artemis.do_paint(world);
    /// ```
    /// # Note
    /// * The function uses the giotto_tool to paint on the map
    /// THIS FUNCTION IS ACTUALLY BROKEN! IT'S GOING TO PANIC!
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
        let mut painter = Drawer::new(img, coord, GiottoDebug::new(utils::DEBUG));

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
            Err(e) => Err(format!("\nARTEMIS-IA: painting failed because {:?}\n", e)),
        }
    }

    /// This function handles the DIE state
    /// # Returns
    /// * a Result<RobotState, String>
    /// # Example
    /// ```
    /// let state = artemis.do_stop();
    /// ```
    /// # Note
    /// * The function calls the handle_event function with the Event::Terminated
    pub fn do_die(&mut self) -> Result<RobotState, String> {
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

    // run function

    /// This function runs the robot
    /// # Arguments
    /// * `world` - A mutable reference to the World
    /// # Example
    /// ```
    /// artemis.run(world);
    /// ```
    /// # Note
    /// * The function prints debug messages
    /// * The function calls the state function for the current state
    /// * The function updates the state based on the new state
    /// * The function panics if the state transition is invalid
    pub fn run(&mut self, world: &mut World) {
        print_debug("why are you running? why-are-you-running?");
        let new_state;

        match &self.state {
            RobotState::INIT => new_state = self.do_init(),
            RobotState::CHILL => new_state = self.do_chill(world),
            RobotState::DETECT => new_state = self.do_detect(world),
            RobotState::GATHER => new_state = self.do_gather(world),
            RobotState::PAINT => new_state = self.do_paint(world),
            RobotState::DIE => new_state = self.do_die(),
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

// Runnable trait implementation

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
