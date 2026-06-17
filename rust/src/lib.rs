use godot::prelude::*;
use godot::classes::Node;
use godot::classes::timer::TimerProcessCallback;

struct MTimerExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MTimerExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct MTimer {
    #[export]
    /// Specifies when the timer is updated during the main loop.
    process_callback: TimerProcessCallback,
    #[export]
    wait_time: f64,
    #[export]
    one_shot: bool,
    #[export]
    /// If `true`, the timer will start immediately when it enters the scene tree.
    ///
    /// **Note:** After the timer enters the tree, this property is immediately set to `false`.
    ///
    /// **Note:** This property does nothing when the timer is running in the editor.
    autostart: bool,
    #[export]
    /// If `true`, the timer will ignore any applied `time_scale_mod`.
    ignore_time_scale: bool,
    #[export]
    /// The modifier to apply to `delta` on each `tick`.
    time_scale_mod: f64,

    /// If `true`, the timer is paused. A paused timer does not process until this property is
    ///  set back to `false`, even when `start()` is called. See also `stop()`.
    paused: bool,
    time_left: f64,

    base: Base<Node>
}

#[godot_api]
impl MTimer {
    #[signal]
    /// Emitted when the timer reaches the end.
    fn timeout();

    #[func]
    pub fn is_stopped(&self) -> bool {
        self.get_time_left() <= 0.
    }

    #[func]
    pub fn start(&mut self, #[opt(default = -1.0)] time_sec: f64) {
        if time_sec > 0. { self.wait_time = time_sec; }

        self.time_left = self.wait_time;
        self.paused = false;
    }

    #[func]
    pub fn stop(&mut self) {
        self.time_left = -1.;
        self.paused = true;
        self.autostart = false;
    }

    pub fn set_paused(&mut self, paused: bool) {
        if self.paused == paused { return }

        self.paused = paused;
    }

    fn tick(&mut self, delta: f64) {
        match self.ignore_time_scale {
            true => self.time_left -= delta,
            false => self.time_left -= delta * self.time_scale_mod
        }

        if self.time_left < 0. {
            match self.one_shot {
                true => self.stop(),
                false => self.time_left += self.wait_time,
            }
            // Emit timeout
            godot_print!("Timing out!");
            self.signals().timeout();
        }
    }

    #[func]
    /// Gets the timers remaining time in seconds. This is always `0` if the timer is stopped.
    /// 
    /// **Note:** This property is read-only and cannot be modified. It is based on `wait_time`.
    fn get_time_left(&self) -> f64 {
        match self.time_left > 0. {
            true => self.time_left,
            false => 0.
        }
    }
}

#[godot_api]
impl INode for MTimer {
    fn init(base: Base<Node>) -> Self {
        Self {
            autostart: false,
            ignore_time_scale: false,
            one_shot: false,
            paused: false,
            process_callback: TimerProcessCallback::IDLE,
            time_left: 0.,
            wait_time: 1.,

            time_scale_mod: 1.,

            base
        }
    }

    fn ready(&mut self) {
        self.time_left = self.wait_time;
    }

    fn process(&mut self, delta: f64) {
        if self.paused { return }
        if self.process_callback == TimerProcessCallback::IDLE {
            self.tick(delta);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.paused { return }
        if self.process_callback == TimerProcessCallback::PHYSICS {
            self.tick(delta);
        }
    }
}
