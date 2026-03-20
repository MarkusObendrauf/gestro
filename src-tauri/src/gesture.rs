use crate::config::Direction;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Debug)]
pub enum GestureState {
    Idle,
    /// Right button is held; waiting to see if movement exceeds threshold.
    #[allow(dead_code)]
    Pressed { start: Point, current: Point },
    /// Movement threshold exceeded; tracking gesture.
    Gesturing { start: Point, current: Point },
}

pub enum GestureEvent {
    RightDown { pos: Point },
    MouseMove { pos: Point },
    RightUp,
}

/// Outcome after processing an event.
pub enum GestureOutcome {
    /// Nothing to do yet.
    None,
    /// No gesture detected; caller should pass through a right-click.
    Passthrough,
    /// Gesture completed in the given direction.
    Gesture(Direction),
}

pub struct GestureEngine {
    pub state: GestureState,
    pub threshold_px: f64,
}

impl GestureEngine {
    pub fn new(threshold_px: f64) -> Self {
        Self {
            state: GestureState::Idle,
            threshold_px,
        }
    }

    pub fn update_threshold(&mut self, threshold_px: f64) {
        self.threshold_px = threshold_px;
    }

    pub fn process(&mut self, event: GestureEvent) -> GestureOutcome {
        match event {
            GestureEvent::RightDown { pos } => {
                self.state = GestureState::Pressed {
                    start: pos,
                    current: pos,
                };
                GestureOutcome::None
            }

            GestureEvent::MouseMove { pos } => match self.state {
                GestureState::Pressed { start, .. } => {
                    if pos.distance_to(&start) >= self.threshold_px {
                        self.state = GestureState::Gesturing {
                            start,
                            current: pos,
                        };
                    } else {
                        self.state = GestureState::Pressed {
                            start,
                            current: pos,
                        };
                    }
                    GestureOutcome::None
                }
                GestureState::Gesturing { start, .. } => {
                    self.state = GestureState::Gesturing {
                        start,
                        current: pos,
                    };
                    GestureOutcome::None
                }
                GestureState::Idle => GestureOutcome::None,
            },

            GestureEvent::RightUp => match self.state {
                GestureState::Pressed { .. } => {
                    self.state = GestureState::Idle;
                    GestureOutcome::Passthrough
                }
                GestureState::Gesturing { start, current } => {
                    self.state = GestureState::Idle;
                    let direction = calc_direction(start, current);
                    GestureOutcome::Gesture(direction)
                }
                GestureState::Idle => GestureOutcome::None,
            },
        }
    }
}

/// Calculate the 8-way direction from `start` to `end`.
/// Screen coordinates: x increases right, y increases down.
/// We flip y so that "up on screen" → North.
fn calc_direction(start: Point, end: Point) -> Direction {
    let dx = end.x - start.x;
    let dy = -(end.y - start.y); // flip y: up = positive
    let angle_rad = dy.atan2(dx);
    let angle_deg = angle_rad.to_degrees();
    Direction::from_angle(angle_deg)
}
