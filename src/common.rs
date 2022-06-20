use serde::Deserialize;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// In means scale goes 1.0 -> 1.5
/// Out means scale goes 1.0 -> 0.5
#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
pub enum PinchDirection {
    In,
    Out,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
pub enum RotateDirection {
    Clockwise,
    Anticlockwise,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Deserialize)]
pub enum AnyDirection {
    Cardinal(Direction),
    Pinch(PinchDirection),
    Rotate(RotateDirection),
}
