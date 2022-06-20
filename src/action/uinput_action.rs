use super::{Action, ActionError};
use uinput::event::keyboard::Key;

pub struct KeyboardInputAction {
    pub device: std::rc::Rc<std::cell::RefCell<uinput::Device>>,
    pub modifiers: Vec<Key>,
    pub sequence: Vec<Key>,
}

impl KeyboardInputAction {
    pub fn default_device() -> std::rc::Rc<std::cell::RefCell<uinput::Device>> {
        let device = uinput::open("/dev/uinput")
            .unwrap()
            .name("wzmach_virtual")
            .unwrap()
            .event(uinput::event::Keyboard::All)
            .unwrap()
            .create()
            .unwrap();
        log::debug!("Created uinput device");
        std::rc::Rc::new(std::cell::RefCell::new(device))
    }
}

impl Action for KeyboardInputAction {
    fn execute(&mut self) -> Result<(), ActionError> {
        let mut device = self.device.borrow_mut();
        log::debug!("Execute action {:?} + {:?}", self.modifiers, self.sequence);
        for modifier in &self.modifiers {
            device.press(modifier)?;
        }
        for key in &self.sequence {
            device.click(key)?;
        }
        for modifier in self.modifiers.iter().rev() {
            device.release(modifier)?;
        }
        device.synchronize()?;
        Ok(())
    }
}

impl From<uinput::Error> for ActionError {
    fn from(err: uinput::Error) -> ActionError {
        ActionError(format!("{}", err))
    }
}
