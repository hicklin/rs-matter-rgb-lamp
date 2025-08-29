use core::cell::Cell;

// -------------------------------
// SDK impl of OnOff cluster
trait OnOffHooks {
    fn toggle_device(&self, on: bool);

    fn raw_get_on_off(&self) -> bool;
    fn raw_set_on_off(&self, on: bool);
}

struct OnOffState<O: OnOffHooks> {
    handler: O,
}

impl<O: OnOffHooks> OnOffState<O> {
    fn new(handler: O) -> Self {
        OnOffState{
            handler,
        }
    }

    fn coupled_cluster_set_on_off(&self, on:bool) {
        println!("setting on off to {}", on);
        let current_on_off = self.handler.raw_get_on_off();

        if current_on_off != on {
            self.handler.toggle_device(on);
            self.handler.raw_set_on_off(on);
        }
    }
}

struct OnOffCluster<'o, 'l, O: OnOffHooks, L: LevelControlHooks> {
    on_off_state: &'o OnOffState<O>,
    level_control_state: Option<&'l LevelControlState<L>>,
}

impl<'o, 'l, O: OnOffHooks, L: LevelControlHooks> OnOffCluster<'o, 'l, O, L> {
    fn new(on_off_state: &'o OnOffState<O>, level_control_state: Option<&'l LevelControlState<L>>) -> Self {

        // At this point would we know if there is a coupled LevelControl cluster?

        Self {
            on_off_state,
            level_control_state,
        }
    }

    fn on_off(&self) -> bool {
        self.on_off_state.handler.raw_get_on_off()
    }

    // Updates the OnOff cluster and calls the LevelControl cluster.
    // This represents handling of Matter commands.
    fn set_on_off(&self, on: bool) {
        if self.on_off() != on {
            self.on_off_state.handler.toggle_device(on);
            self.on_off_state.handler.raw_set_on_off(on);
            if on {
                if let Some(level_control_state) = self.level_control_state {
                    level_control_state.coupled_cluster_level_to_on_level();
                }
            }
        }
    }
    
}

// -------------------------------
// SDK impl of LevelControl cluster
trait LevelControlHooks {
    fn set_device_level(&self, level: u8);

    fn raw_get_level(&self) -> u8;
    fn raw_set_level(&self, level: u8);
    fn raw_get_on_level(&self) -> u8;
    fn raw_set_on_level(&self, level: u8);
}

struct LevelControlState<O: LevelControlHooks> {
    handler: O
}

impl<O: LevelControlHooks> LevelControlState<O> {
    fn new(handler: O) -> Self {
        Self { handler }
    }

    fn coupled_cluster_level_to_on_level(&self) {
        let on_level = self.handler.raw_get_on_level();
        self.handler.set_device_level(on_level);
        self.handler.raw_set_level(on_level);
    }
}

struct LevelControlCluster<'l, 'o, L: LevelControlHooks, O: OnOffHooks> {
    level_control_state: &'l LevelControlState<L>,
    on_off_state: Option<&'o OnOffState<O>>,
}

impl<'l, 'o, L: LevelControlHooks, O: OnOffHooks> LevelControlCluster<'l, 'o, L, O> {
    fn new(level_control_state: &'l LevelControlState<L>, on_off_state: Option<&'o OnOffState<O>>) -> Self {

        // here we can check that if the ON_OFF feature is set, on_off_state is not None.

        Self{
            level_control_state,
            on_off_state,
        }
    }

    fn level(&self) -> u8 {
        self.level_control_state.handler.raw_get_level()
    }

    fn set_level(&self, level: u8) {
        if self.level() == level {
            return;
        }

        if let Some(on_off_state) = self.on_off_state {
            on_off_state.coupled_cluster_set_on_off(self.level() != 0);
        }

        self.level_control_state.handler.set_device_level(level);
        self.level_control_state.handler.raw_set_level(level);
    }

    fn on_level(&self) -> u8 {
        self.level_control_state.handler.raw_get_on_level()
    }

    fn set_on_level(&self, level: u8) {
        self.level_control_state.handler.raw_set_on_level(level);
    }

}


// -------------------------------------------
// Implemented by the SDK consumer
struct UserOnOffLogic {
    on_off: Cell<bool>,
}

impl UserOnOffLogic {
    fn new() -> Self {
        Self {
            on_off: Cell::new(false),
        }
    }
}

impl OnOffHooks for UserOnOffLogic {
    fn toggle_device(&self, on: bool) {
        println!("Toggled device to {}", on)
    }

    fn raw_get_on_off(&self) -> bool {
        self.on_off.get()
    }

    fn raw_set_on_off(&self, on: bool) {
        self.on_off.set(on);
    }
}

struct UserLevelControlLogic {
    level: Cell<u8>,
    on_level: Cell<u8>,
}

impl UserLevelControlLogic {
    fn new() -> Self {
        Self {
            level: Cell::new(0),
            on_level: Cell::new(42),
        }
    }
}


impl LevelControlHooks for UserLevelControlLogic {
    fn set_device_level(&self, level: u8) {
        println!("Setting device level to {}", level)
    }

    fn raw_get_level(&self) -> u8 {
        self.level.get()
    }

    fn raw_set_level(&self, level: u8) {
        self.level.set(level);
    }

    fn raw_get_on_level(&self) -> u8 {
        self.on_level.get()
    }

    fn raw_set_on_level(&self, level: u8) {
        self.on_level.set(level);
    }
}


fn main() {

    // Initialising the clusters
    let on_off_handler = UserOnOffLogic::new();
    let on_off_state = OnOffState::new(on_off_handler);

    let level_control_handler = UserLevelControlLogic::new();
    let level_control_state = LevelControlState::new(level_control_handler);

    let on_off_cluster = OnOffCluster::new(&on_off_state, Some(&level_control_state));
    let level_control_cluster = LevelControlCluster::new(&level_control_state, Some(&on_off_state));


    println!("Test 1: on_off value: {} | level_control value: {}", on_off_cluster.on_off(), level_control_cluster.level());
    assert_eq!(on_off_cluster.on_off(), false);
    assert_eq!(level_control_cluster.level(), 0);

    on_off_cluster.set_on_off(true);

    println!("Test 2: on_off value: {} | level_control value: {}", on_off_cluster.on_off(), level_control_cluster.level());
    assert_eq!(on_off_cluster.on_off(), true);
    assert_eq!(level_control_cluster.level(), level_control_cluster.on_level());

    on_off_cluster.set_on_off(false);
    println!("Test 3: on_off value: {} | level_control value: {}", on_off_cluster.on_off(), level_control_cluster.level());
    assert_eq!(on_off_cluster.on_off(), false);

    level_control_cluster.set_level(83);
    println!("Test 4: on_off value: {} | level_control value: {}", on_off_cluster.on_off(), level_control_cluster.level());
    assert_eq!(on_off_cluster.on_off(), true);
    assert_eq!(level_control_cluster.level(), 83);
}
