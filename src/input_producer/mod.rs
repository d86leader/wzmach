//! Provides 'GestureProducer' that you can observe as an Iterator to get
//! gesture events

pub mod event;

use event::{Gesture, InputEvent};

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::path::Path;

use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

/* Libinput thing */

struct Interface;

/// Interface that just tries to open files directly. This requires running as
/// root or using sgid and the group "input"
impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

/* The gesture iterator */

/// The producer of gesture input. Create and then use as iterator.
pub struct GestureProducer {
    input: input::Libinput,
    current: Gesture,
}

impl GestureProducer {
    pub fn new() -> Self {
        // Gesture sequences always start with a LIBINPUT_EVENT_GESTURE_FOO_START
        // event. All following gesture events will be of the
        // LIBINPUT_EVENT_GESTURE_FOO_UPDATE type until a
        // LIBINPUT_EVENT_GESTURE_FOO_END is generated which signals the end of the
        // gesture.
        // Source: https://wayland.freedesktop.org/libinput/doc/latest/api/group__event__gesture.html
        // But in the end it doesn't matter (explicitly) for this
        // implementation. Maybe I implicitly depend on it somewhere..

        let mut input = Libinput::new_with_udev(Interface);
        input.udev_assign_seat("seat0").unwrap();

        GestureProducer {
            input,
            current: Gesture::None,
        }
    }

    fn poll_events(&mut self) {
        use nix::poll::PollFlags;
        let pollfd = nix::poll::PollFd::new(self.input.as_raw_fd(), PollFlags::POLLIN);
        nix::poll::poll(&mut [pollfd], -1).unwrap();
        self.input.dispatch().unwrap();
    }
}

impl Iterator for GestureProducer {
    type Item = InputEvent;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.input.next() {
                Some(input::Event::Gesture(gest)) => {
                    let state = self.current.update(&gest);
                    break Some(InputEvent::from_state(state, &self.current));
                }
                Some(_) => (),
                None => self.poll_events(),
            }
        }
    }
}
