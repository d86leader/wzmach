use crate::common::{Direction, PinchDirection, RotateDirection};
use crate::gesture_event::trigger as gesture;

use serde::Deserialize;

#[derive(PartialEq, Debug, Clone, Copy, Deserialize)]
pub enum Trigger {
    Swipe {
        fingers: u32,
        direction: Direction,
        repeated: bool,
    },
    Shear {
        fingers: u32,
        direction: Direction,
        repeated: bool,
    },
    Pinch {
        fingers: u32,
        direction: PinchDirection,
        repeated: bool,
    },
    Rotate {
        fingers: u32,
        direction: RotateDirection,
        repeated: bool,
    },
    Hold {
        fingers: u32,
    },
}

#[derive(PartialEq, Debug, Clone, Copy, Deserialize)]
struct Cardinal {
    fingers: u32,
    direction: Direction,
    repeated: bool,
}

#[derive(PartialEq, Debug, Clone, Copy, Deserialize)]
struct Pinch {
    fingers: u32,
    direction: PinchDirection,
    repeated: bool,
}

#[derive(PartialEq, Debug, Clone, Copy, Deserialize)]
struct Hold {
    fingers: u32,
}

impl Trigger {
    pub fn make(
        self,
        swipe_distance: u32,
        shear_distance: u32,
        pinch_distance: f64,
        rotate_distance: f64,
    ) -> gesture::Trigger {
        match self {
            Trigger::Swipe {
                fingers,
                direction,
                repeated,
            } => gesture::Trigger::Swipe(gesture::CardinalTrigger {
                fingers: fingers.try_into().expect("Too many fingers"),
                direction,
                distance: swipe_distance.try_into().expect("Incorrect distance"),
                repeated,
            }),
            Trigger::Shear {
                fingers,
                direction,
                repeated,
            } => gesture::Trigger::Shear(gesture::CardinalTrigger {
                fingers: fingers.try_into().expect("Too many fingers"),
                direction,
                distance: shear_distance.try_into().expect("Incorrect distance"),
                repeated,
            }),
            Trigger::Pinch {
                fingers,
                direction,
                repeated,
            } => gesture::Trigger::Pinch(gesture::PinchTrigger {
                fingers: fingers.try_into().expect("Too many fingers"),
                direction,
                scale: pinch_distance,
                repeated,
            }),
            Trigger::Rotate {
                fingers,
                direction,
                repeated,
            } => gesture::Trigger::Rotate(gesture::RotateTrigger {
                fingers: fingers.try_into().expect("Too many fingers"),
                direction,
                distance: rotate_distance,
                repeated,
            }),
            Trigger::Hold { fingers } => gesture::Trigger::Hold(gesture::HoldTrigger {
                fingers: fingers.try_into().expect("Too many fingers"),
                time: 1, // Time not implemented currently
            }),
        }
    }
}
