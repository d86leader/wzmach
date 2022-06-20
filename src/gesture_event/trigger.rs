//! Triggers for gesture recognition
//!
//! pub(crate): These triggers can also perform computations to see if events
//! match them

use crate::common::{AnyDirection, Direction, PinchDirection, RotateDirection};
use crate::input_producer::event::{HoldGesture, PinchGesture, SwipeGesture};

const VSLOPE: f64 = 1.0;
const HSLOPE: f64 = 1.0 / VSLOPE;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Trigger {
    Swipe(CardinalTrigger),
    Pinch(PinchTrigger),
    /// Shear is when you move different fingers in different directions.
    /// Usually done by resting your digits and moving your thumb. The direction
    /// is the direction of the thumb, and the distance is relative movement. Be
    /// careful as vertical shearing can be mistaken by the engine as a pinch
    Shear(CardinalTrigger),
    /// Rotate is when you rotate your fingers during a pinch. This heavily
    /// conflicts with shears as far as recognition goes
    Rotate(RotateTrigger),
    /// Sent only when hold ended
    Hold(HoldTrigger),
    // TODO: hold in progress. Need to track my own time, bleh
}

/// Common struct for triggers in a certain direction over a certain distance:
/// swipes and shears.
/// Wow, why rust still has the same record problems that haskell does? Why
/// can't I just have my anonymous structs
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct CardinalTrigger {
    pub fingers: i32,
    pub direction: Direction,
    pub distance: f64,
    pub repeated: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct PinchTrigger {
    pub fingers: i32,
    pub direction: PinchDirection,
    pub scale: f64,
    pub repeated: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct RotateTrigger {
    pub fingers: i32,
    pub direction: RotateDirection,
    /// Measured in something like degrees, although on my touchpad 90 units
    /// don't match a real 90 degree rotation, but it's pretty close
    pub distance: f64,
    pub repeated: bool,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct HoldTrigger {
    pub fingers: i32,
    pub time: u32,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct Origin {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub rotation: f64,
}

/* Impls for matchins */

impl Direction {
    fn matches(&self, dx: f64, dy: f64) -> bool {
        // from running libinput: up is negative, left is positive
        match self {
            Direction::Up => dy <= VSLOPE * dx && dy <= -VSLOPE * dx,
            Direction::Down => dy >= VSLOPE * dx && dy >= -VSLOPE * dx,
            Direction::Right => dx >= HSLOPE * dy && dx >= -HSLOPE * dy,
            Direction::Left => dx <= HSLOPE * dy && dx <= -HSLOPE * dy,
        }
    }
}

impl RotateDirection {
    fn matches(&self, sign: f64) -> bool {
        match self {
            RotateDirection::Anticlockwise => sign < 0.0,
            RotateDirection::Clockwise => sign > 0.0,
        }
    }
}

impl CardinalTrigger {
    pub(crate) fn matches_swipe(&self, gest: &SwipeGesture, o: Origin) -> bool {
        self.fingers == gest.fingers
            && self.direction.matches(gest.dx - o.x, gest.dy - o.y)
            && ((gest.dx - o.x).abs() >= self.distance || (gest.dy - o.y).abs() >= self.distance)
    }
    // TODO: deduplicate. Same implementation, different types with same shape
    pub(crate) fn matches_shear(&self, gest: &PinchGesture, o: Origin) -> bool {
        self.fingers == gest.fingers
            && self.direction.matches(gest.dx - o.x, gest.dy - o.y)
            && ((gest.dx - o.x).abs() >= self.distance || (gest.dy - o.y).abs() >= self.distance)
    }
}

impl PinchTrigger {
    pub(crate) fn matches(&self, gest: &PinchGesture, origin: f64) -> bool {
        /*
        // This is useful when debugging how I broke pinches again, but is too
        // verbose even for trace
        log::trace!(
            "consider {:?}, {:.3} < {:.3} < {:.3}",
            gest,
            origin / self.scale,
            gest.scale,
            origin * self.scale
        );
        */
        self.fingers == gest.fingers
            && match self.direction {
                PinchDirection::In => origin * self.scale <= gest.scale,
                PinchDirection::Out => origin / self.scale >= gest.scale,
            }
    }
}

impl RotateTrigger {
    pub(crate) fn matches(&self, gest: &PinchGesture, origin: f64) -> bool {
        let angle = gest.angle - origin;
        self.fingers == gest.fingers
            && self.direction.matches(angle.signum())
            && angle.abs() >= self.distance
    }
}

impl HoldTrigger {
    pub(crate) fn matches(&self, gest: &HoldGesture, ctime: u32) -> bool {
        self.fingers == gest.fingers && ctime.saturating_sub(gest.begin_time) >= self.time
    }
}

/* Impl for generalized field access */

impl Trigger {
    pub(crate) fn repeated(&self) -> bool {
        match self {
            Trigger::Swipe(s) => s.repeated,
            Trigger::Pinch(p) => p.repeated,
            Trigger::Shear(s) => s.repeated,
            Trigger::Rotate(r) => r.repeated,
            // you can't repeat holds, but repeated are simpler to handle
            Trigger::Hold(_) => true,
        }
    }

    pub(crate) fn direction(&self) -> Option<AnyDirection> {
        match self {
            Trigger::Swipe(s) => Some(AnyDirection::Cardinal(s.direction)),
            Trigger::Pinch(p) => Some(AnyDirection::Pinch(p.direction)),
            Trigger::Shear(s) => Some(AnyDirection::Cardinal(s.direction)),
            Trigger::Rotate(r) => Some(AnyDirection::Rotate(r.direction)),
            Trigger::Hold(_) => None,
        }
    }
}
