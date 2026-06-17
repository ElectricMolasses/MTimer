use godot::prelude::*;
use godot::builtin::Callable;
use godot::classes::Node;
use godot::classes::timer::TimerProcessCallback;
use godot::classes::Engine;

struct MTimerExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MTimerExtension {}

#[derive(GodotConvert, Var, Export, Debug, Clone, Copy, PartialEq, Eq)]
#[godot(via = i32)]
pub enum TimeScaleMode {
    FLOAT,
    CALLABLE,
}

#[derive(GodotClass)]
#[class(base=Node)]
/// A Timer node that supports per-timer time scale modifications, via either a static float,
/// or a callback to automatically update with changing values.
///
/// This Timer meets all native Timer interface requirements to drop in over existing timers,
/// except the ability to create a Timer without adding it to the tree yourself.
pub struct MTimer {
    /// Specifies when the timer is updated during the main loop.
    #[export]
    process_callback: TimerProcessCallback,
    /// The time required for the time to end, in seconds. This property can also be set
    /// every time `start()` is called.
    ///
    /// **Note:** Timers can only process once per physics or process frame (depending on the
    /// `process_callback`). An unstable framerate may cause the timer to end inconsistently,
    /// which is especially noticeable if the wait time is lower than roughly `0.05` seconds.
    /// For very short timers, it is recommended to write your own code instead of using a
    /// **Timer** node. Timers are also affected by `Engine.time_scale`. This can be disabled
    /// with `ignore_time_scale`.
    #[export]
    wait_time: f64,
    #[export]
    one_shot: bool,
    /// If `true`, the timer will start immediately when it enters the scene tree.
    ///
    /// **Note:** After the timer enters the tree, this property is immediately set to `false`.
    ///
    /// **Note:** This property does nothing when the timer is running in the editor.
    #[export]
    autostart: bool,
    /// If `true`, the timer will ignore any applied `time_scale_mod`.
    #[export]
    ignore_time_scale: bool,
    /// If `true`, the timer will ignore `Engine.time_scale` and update with real, elapsed time.
    #[export]
    ignore_mod_time_scale: bool,
    /// The modifier to apply to `delta` on each `tick`.
    #[var(set = set_mul)]
    time_scale_mod: f64,
    /// The callback that should return an f64 representing the current time mod.
    ///
    /// This allows one callback to be provided to the `MTimer` for the `MTimer` to call
    ///  each tick, removing the need to update a changing modifier manually.
    #[var]
    time_scale_mod_cb: Callable,
    /// Internal only, used to determine whether or not MTimer should be reading the
    ///  time mod from a `Callback`, or the internal `time_scale_mod` value.
    time_scale_mod_mode: TimeScaleMode,

    #[var]
    /// If `true`, the timer is paused. A paused timer does not process until this property is
    ///  set back to `false`, even when `start()` is called. See also `stop()`.
    paused: bool,
    /// The timer's remaining time in seconds. This is always `0` if the timer is stopped.
    ///
    /// **Note**: This property should not be modified. It is based on `wait_time`.
    #[var(get = get_time_left)]
    time_left: f64,

    base: Base<Node>
}

#[godot_api]
impl MTimer {
    /// Emitted when the timer reaches the end.
    #[signal]
    fn timeout();

    /// Returns `true` if the timer is stopped or has not started.
    #[func]
    pub fn is_stopped(&self) -> bool {
        self.time_left <= 0.
    }

    /// Starts the timer, or resets the timer if it was started already. Fails if the timer
    /// is not inside the scene tree. If `time_sec` is greater than `0`, this value is used
    /// for the `wait_time`.
    ///
    /// **Note:** This method does not resume a paused timer. See `paused`.
    #[func]
    pub fn start(&mut self, #[opt(default = -1.0)] time_sec: f64) {
        if time_sec > 0. { self.wait_time = time_sec; }

        self.time_left = self.wait_time;
        self.paused = false;
    }

    /// Stops the timer. See also `paused`. Unlike `start()`, this can safely be called if
    /// the timer is not inside the scene tree.
    ///
    /// **Note:** Calling `stop()` does not emit the `timeout` signal, as the timer is not
    /// considered to have timed out. If this is desired, use `$Timer.timeout.emit()` after
    /// calling `stop()` to manually emit the signal.
    #[func]
    pub fn stop(&mut self) {
        self.time_left = -1.;
        self.paused = true;
        self.autostart = false;
    }

    /// Sets a callback that the `MTimer` will invoke to get the current time multiplier.
    ///
    /// The callback must adhere to the following signature: () -> float
    #[func]
    pub fn set_mul_cb(&mut self, cb: Callable) {
        self.time_scale_mod_cb = cb;
        self.time_scale_mod_mode = TimeScaleMode::CALLABLE;
    }

    /// Sets the static time multiplier that the `MTimer` will leverage if no callback
    /// is provided.
    #[func]
    fn set_mul(&mut self, mul: f64) {
        self.time_scale_mod = mul;
        self.clear_mul_cb();
    }

    /// Clears any set time multiplier callbacks and informs the `MTimer` that it should
    /// use whatever static value is currently set.
    #[func]
    pub fn clear_mul_cb(&mut self) {
        self.time_scale_mod_mode = TimeScaleMode::FLOAT;
        self.time_scale_mod_cb = Callable::invalid();
    }

    fn tick(&mut self, delta: f64) {
        let mut p_delta: f64 = match self.ignore_time_scale {
            true => delta * 1./Engine::singleton().get_time_scale(),
            false => delta,
        };
        p_delta *= match self.ignore_mod_time_scale {
            true => 1.,
            false => match self.time_scale_mod_mode {
                TimeScaleMode::FLOAT => self.time_scale_mod,
                TimeScaleMode::CALLABLE => self.time_scale_mod_cb.call(&[]).to::<f64>(),
            }
        };

        self.time_left -= p_delta;

        if self.time_left < 0. {
            match self.one_shot {
                true => self.stop(),
                false => self.time_left += self.wait_time,
            }
            self.signals().timeout().emit();
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
            ignore_mod_time_scale: false,
            one_shot: false,
            paused: true,
            process_callback: TimerProcessCallback::IDLE,
            time_left: 0.,
            wait_time: 1.,

            time_scale_mod_mode: TimeScaleMode::FLOAT,
            time_scale_mod: 1.,
            time_scale_mod_cb: Callable::invalid(),

            base
        }
    }

    fn ready(&mut self) {
        if self.autostart {
            self.start(-1.0);
            self.autostart = false;
        }
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
