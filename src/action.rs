mod command_action;
mod uinput_action;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("Failure executing action: {0}")]
pub struct ActionError(pub String);

pub trait Action {
    fn execute(&mut self) -> Result<(), ActionError>;
}

pub use uinput_action::UinputAction;
pub use command_action::{CommandAction, ShellCommandAction};