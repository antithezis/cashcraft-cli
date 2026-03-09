//! Animation system for CashCraft TUI
//!
//! Provides smooth animations for:
//! - Progress bar animations
//! - Number counting effects
//! - Transition effects
//! - Frame timing utilities

use std::time::{Duration, Instant};

/// Animation speed presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationSpeed {
    /// Slow animations (500ms base duration)
    Slow,
    /// Normal animations (250ms base duration)
    #[default]
    Normal,
    /// Fast animations (100ms base duration)
    Fast,
    /// Instant/no animation
    Instant,
}

impl AnimationSpeed {
    /// Get the base duration for this speed
    pub fn duration(&self) -> Duration {
        match self {
            AnimationSpeed::Slow => Duration::from_millis(500),
            AnimationSpeed::Normal => Duration::from_millis(250),
            AnimationSpeed::Fast => Duration::from_millis(100),
            AnimationSpeed::Instant => Duration::ZERO,
        }
    }

    /// Scale duration by a factor
    pub fn scaled(&self, factor: f64) -> Duration {
        let base_ms = self.duration().as_millis() as f64;
        Duration::from_millis((base_ms * factor) as u64)
    }
}

/// Easing function types
#[derive(Debug, Clone, Copy, Default)]
pub enum Easing {
    /// Linear interpolation
    Linear,
    /// Ease in (accelerating)
    EaseIn,
    /// Ease out (decelerating)
    #[default]
    EaseOut,
    /// Ease in-out (smooth start and end)
    EaseInOut,
    /// Bounce effect at end
    Bounce,
    /// Elastic effect
    Elastic,
}

impl Easing {
    /// Apply easing function to a progress value (0.0 to 1.0)
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(2),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let mut t = t;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    t -= 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    t -= 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    t -= 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            }
            Easing::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    (2.0_f64.powf(-10.0 * t)) * ((t - s) * (2.0 * std::f64::consts::PI / p)).sin()
                        + 1.0
                }
            }
        }
    }
}

/// A basic animation with progress tracking
#[derive(Debug, Clone)]
pub struct Animation {
    /// When the animation started
    start: Instant,
    /// Total duration of the animation
    duration: Duration,
    /// Whether the animation has completed
    pub completed: bool,
    /// Easing function to use
    easing: Easing,
    /// Whether animation is paused
    paused: bool,
    /// Time when paused
    pause_time: Option<Instant>,
}

impl Animation {
    /// Create a new animation with the given speed
    pub fn new(speed: AnimationSpeed) -> Self {
        Self {
            start: Instant::now(),
            duration: speed.duration(),
            completed: speed == AnimationSpeed::Instant,
            easing: Easing::EaseOut,
            paused: false,
            pause_time: None,
        }
    }

    /// Create with custom duration
    pub fn with_duration(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
            completed: duration.is_zero(),
            easing: Easing::EaseOut,
            paused: false,
            pause_time: None,
        }
    }

    /// Set the easing function
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Get raw progress (0.0 to 1.0) without easing
    pub fn progress(&self) -> f64 {
        if self.completed || self.duration.is_zero() {
            return 1.0;
        }

        let elapsed = if self.paused {
            self.pause_time.map(|t| t - self.start).unwrap_or_default()
        } else {
            self.start.elapsed()
        };

        let total = self.duration.as_secs_f64();
        let elapsed_f64 = elapsed.as_secs_f64();

        (elapsed_f64 / total).min(1.0)
    }

    /// Get eased progress (0.0 to 1.0)
    pub fn eased_progress(&self) -> f64 {
        self.easing.apply(self.progress())
    }

    /// Update animation state, returns true if still animating
    pub fn tick(&mut self) -> bool {
        if !self.paused && self.progress() >= 1.0 {
            self.completed = true;
        }
        !self.completed
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if !self.paused {
            self.paused = true;
            self.pause_time = Some(Instant::now());
        }
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        if self.paused {
            if let Some(pause_time) = self.pause_time {
                let paused_duration = pause_time.elapsed();
                self.start += paused_duration;
            }
            self.paused = false;
            self.pause_time = None;
        }
    }

    /// Reset and restart the animation
    pub fn restart(&mut self) {
        self.start = Instant::now();
        self.completed = false;
        self.paused = false;
        self.pause_time = None;
    }
}

/// Animated number counter (for smooth number transitions)
#[derive(Debug, Clone)]
pub struct NumberCounter {
    /// Starting value
    from: f64,
    /// Target value
    to: f64,
    /// Animation state
    animation: Animation,
    /// Format as currency
    currency: bool,
    /// Decimal places
    decimals: u8,
}

impl NumberCounter {
    /// Create a new number counter
    pub fn new(from: f64, to: f64, speed: AnimationSpeed) -> Self {
        Self {
            from,
            to,
            animation: Animation::new(speed),
            currency: false,
            decimals: 2,
        }
    }

    /// Format as currency
    pub fn currency(mut self, is_currency: bool) -> Self {
        self.currency = is_currency;
        self
    }

    /// Set decimal places
    pub fn decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals;
        self
    }

    /// Set easing function
    pub fn easing(mut self, easing: Easing) -> Self {
        self.animation = self.animation.easing(easing);
        self
    }

    /// Get current interpolated value
    pub fn current(&self) -> f64 {
        let progress = self.animation.eased_progress();
        self.from + (self.to - self.from) * progress
    }

    /// Get formatted current value
    pub fn formatted(&self) -> String {
        let value = self.current();
        let decimals = self.decimals as usize;
        if self.currency {
            // Format with currency symbol and proper decimal places
            let formatted = format!("{:.prec$}", value, prec = decimals);
            format!("${}", formatted)
        } else {
            format!("{:.prec$}", value, prec = decimals)
        }
    }

    /// Update animation state
    pub fn tick(&mut self) -> bool {
        self.animation.tick()
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.animation.is_complete()
    }

    /// Update target value (for continuous updates)
    pub fn set_target(&mut self, target: f64) {
        self.from = self.current();
        self.to = target;
        self.animation.restart();
    }
}

/// Transition animation between two states
#[derive(Debug, Clone)]
pub struct Transition<T> {
    /// Previous state
    from: T,
    /// Current/target state
    to: T,
    /// Animation state
    animation: Animation,
}

impl<T: Clone> Transition<T> {
    /// Create a new transition
    pub fn new(initial: T, _speed: AnimationSpeed) -> Self {
        Self {
            from: initial.clone(),
            to: initial,
            animation: Animation::new(AnimationSpeed::Instant),
        }
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: T, speed: AnimationSpeed) {
        self.from = self.to.clone();
        self.to = new_state;
        self.animation = Animation::new(speed);
    }

    /// Get the current progress (0.0 = from state, 1.0 = to state)
    pub fn progress(&self) -> f64 {
        self.animation.eased_progress()
    }

    /// Get the 'from' state
    pub fn from(&self) -> &T {
        &self.from
    }

    /// Get the 'to' state
    pub fn to(&self) -> &T {
        &self.to
    }

    /// Update animation state
    pub fn tick(&mut self) -> bool {
        self.animation.tick()
    }

    /// Check if transition is complete
    pub fn is_complete(&self) -> bool {
        self.animation.is_complete()
    }
}

/// Interpolate between two f64 values
impl Transition<f64> {
    /// Get interpolated value
    pub fn current(&self) -> f64 {
        let progress = self.progress();
        self.from + (self.to - self.from) * progress
    }
}

/// Pulse animation (for attention effects)
#[derive(Debug, Clone)]
pub struct Pulse {
    /// Base value
    base: f64,
    /// Amplitude of pulse
    amplitude: f64,
    /// Period in milliseconds
    period_ms: u64,
    /// Start time
    start: Instant,
    /// Number of pulses (None = infinite)
    count: Option<u32>,
    /// Current pulse count
    current_count: u32,
}

impl Pulse {
    /// Create a new pulse animation
    pub fn new(base: f64, amplitude: f64) -> Self {
        Self {
            base,
            amplitude,
            period_ms: 1000,
            start: Instant::now(),
            count: None,
            current_count: 0,
        }
    }

    /// Set pulse period
    pub fn period(mut self, ms: u64) -> Self {
        self.period_ms = ms;
        self
    }

    /// Set number of pulses
    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    /// Get current pulse value
    pub fn value(&self) -> f64 {
        if let Some(max_count) = self.count {
            if self.current_count >= max_count {
                return self.base;
            }
        }

        let elapsed_ms = self.start.elapsed().as_millis() as f64;
        let phase = (elapsed_ms / self.period_ms as f64) * 2.0 * std::f64::consts::PI;
        self.base + self.amplitude * phase.sin().abs()
    }

    /// Update and check if complete
    pub fn tick(&mut self) -> bool {
        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        self.current_count = (elapsed_ms / self.period_ms) as u32;

        if let Some(max_count) = self.count {
            self.current_count < max_count
        } else {
            true
        }
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        if let Some(max_count) = self.count {
            self.current_count >= max_count
        } else {
            false
        }
    }
}

/// Animation controller for managing multiple animations
#[derive(Debug, Default)]
pub struct AnimationController {
    /// Active animations by ID
    animations: std::collections::HashMap<String, Animation>,
}

impl AnimationController {
    pub fn new() -> Self {
        Self {
            animations: std::collections::HashMap::new(),
        }
    }

    /// Start a new animation
    pub fn start(&mut self, id: impl Into<String>, speed: AnimationSpeed) {
        self.animations.insert(id.into(), Animation::new(speed));
    }

    /// Start with custom duration
    pub fn start_duration(&mut self, id: impl Into<String>, duration: Duration) {
        self.animations
            .insert(id.into(), Animation::with_duration(duration));
    }

    /// Get progress for an animation
    pub fn progress(&self, id: &str) -> f64 {
        self.animations
            .get(id)
            .map(|a| a.eased_progress())
            .unwrap_or(1.0)
    }

    /// Check if animation exists and is active
    pub fn is_active(&self, id: &str) -> bool {
        self.animations
            .get(id)
            .map(|a| !a.is_complete())
            .unwrap_or(false)
    }

    /// Update all animations
    pub fn tick(&mut self) {
        self.animations.retain(|_, anim| anim.tick());
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }

    /// Check if any animation is active
    pub fn has_active(&self) -> bool {
        self.animations.values().any(|a| !a.is_complete())
    }
}

/// Frame timing helper for consistent animation FPS
#[derive(Debug)]
pub struct FrameTimer {
    /// Target frame duration
    target_duration: Duration,
    /// Last frame time
    last_frame: Instant,
}

impl FrameTimer {
    /// Create with target FPS
    pub fn with_fps(fps: u32) -> Self {
        let target_duration = Duration::from_secs_f64(1.0 / fps as f64);
        Self {
            target_duration,
            // Initialize in the past so first frame is always ready
            last_frame: Instant::now() - target_duration,
        }
    }

    /// Check if it's time for next frame
    pub fn should_update(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_frame) >= self.target_duration {
            self.last_frame = now;
            true
        } else {
            false
        }
    }

    /// Get remaining time until next frame
    pub fn time_until_next(&self) -> Duration {
        let elapsed = self.last_frame.elapsed();
        self.target_duration.saturating_sub(elapsed)
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::with_fps(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_animation_speed_duration() {
        assert_eq!(AnimationSpeed::Slow.duration(), Duration::from_millis(500));
        assert_eq!(
            AnimationSpeed::Normal.duration(),
            Duration::from_millis(250)
        );
        assert_eq!(AnimationSpeed::Fast.duration(), Duration::from_millis(100));
        assert_eq!(AnimationSpeed::Instant.duration(), Duration::ZERO);
    }

    #[test]
    fn test_easing_linear() {
        let easing = Easing::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = Easing::EaseOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!(easing.apply(0.5) > 0.5); // Faster at start
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_animation_instant() {
        let anim = Animation::new(AnimationSpeed::Instant);
        assert!(anim.is_complete());
        assert_eq!(anim.progress(), 1.0);
    }

    #[test]
    fn test_animation_progress() {
        let anim = Animation::new(AnimationSpeed::Fast);
        assert!(anim.progress() < 1.0);

        // Wait for animation to complete
        thread::sleep(Duration::from_millis(150));
        assert_eq!(anim.progress(), 1.0);
    }

    #[test]
    fn test_number_counter() {
        let mut counter = NumberCounter::new(0.0, 100.0, AnimationSpeed::Instant);
        assert_eq!(counter.current(), 100.0);
        assert!(counter.is_complete());

        counter = NumberCounter::new(0.0, 100.0, AnimationSpeed::Fast);
        assert!(counter.current() >= 0.0);
        assert!(counter.current() <= 100.0);
    }

    #[test]
    fn test_number_counter_formatted() {
        let counter = NumberCounter::new(0.0, 1234.56, AnimationSpeed::Instant)
            .currency(true)
            .decimals(2);
        assert_eq!(counter.formatted(), "$1234.56");
    }

    #[test]
    fn test_pulse() {
        let mut pulse = Pulse::new(1.0, 0.5).period(100).count(2);

        // Value should be between base and base+amplitude
        let value = pulse.value();
        assert!(value >= 1.0 && value <= 1.5);

        // After enough time, should be complete
        thread::sleep(Duration::from_millis(250));
        pulse.tick();
        assert!(pulse.is_complete());
    }

    #[test]
    fn test_animation_controller() {
        let mut controller = AnimationController::new();

        controller.start("test", AnimationSpeed::Fast);
        assert!(controller.is_active("test"));

        controller.start("instant", AnimationSpeed::Instant);
        assert!(!controller.is_active("instant"));

        controller.tick();
        assert!(!controller.is_active("nonexistent"));
    }

    #[test]
    fn test_frame_timer() {
        let mut timer = FrameTimer::with_fps(60);

        // First call should always update
        assert!(timer.should_update());

        // Immediate second call should not
        assert!(!timer.should_update());

        // After enough time, should update again
        thread::sleep(Duration::from_millis(20));
        assert!(timer.should_update());
    }
}
