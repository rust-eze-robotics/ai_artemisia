pub mod utils;

// stdlibs
use rand::Rng;
use spyglass::spyglass::Spyglass;

enum RobotState {
    INIT = 0,
    CHILL, // idle state, relax
    COLLECT, // use spyglass + sense&find to find collect stuff
    PAINT, // use giotto_tool to paint on the map
    STOP, // as most artist do, the robot gracefully terminates its existance
    NUM_STATES
}


// trait StateFN {
//     fn state_fn() -> RobotState {} // state function
//     fn transition_fn() {} // state transition function
// }


struct Robot {
    state: RobotState,
    countdown: i32,
    spyglass: Spyglass
}

impl Robot {
    fn new() -> Self {
        Robot {
            state: RobotState::INIT,
            countdown: 0,
            spyglass: Spyglass::new_default(0,0, 10, 100)
        }
    }

    // state functions
    fn do_init(&mut self) {
        let mut rng = rand::thread_rng();
        self.countdown = rng.gen_range(0..=13);
        self.to_collect();
    }

    fn do_collect(&mut self) {
        // TODO: use spyglass + sense&find to collect stuff

        self.to_chill();
    }
    fn do_chill(&mut self) {
        // idle for a random amount of time, relax and find inspiration
        self.to_paint();
    }
    fn do_paint(&mut self) {
        // pain't, create art from pain (and stuff you collected)

        if self.countdown == 0 {
            self.to_stop();
        } else {
            self.to_collect();
        }
    }
    fn do_stop(&mut self) {
        // 'peg out', 'die', 'stop working'
        // grand sortie: when the robot paints the amount of paintings, assigned during the init state,
        // it gracefully pegs out, and as a last performance the whole map gets covered in lava (red canva, inspired to Fontana's "Concetto Spaziale")
        
        let path = "res/img/fontana_concettospaziale.jpg";
        let painting = utils::build_img(path);

        println!("{:?}", painting);
    }

    // transition functions
    fn to_collect(&mut self) {
        self.state = RobotState::COLLECT; 
        self.do_collect();
    }
    fn to_chill(&mut self) {
        self.state = RobotState::CHILL;
        self.do_chill();
    }
    fn to_paint(&mut self) {
        self.state = RobotState::PAINT;
        self.do_paint();
    }
    fn to_stop(&mut self) {
        self.state = RobotState::STOP;
    }
}

