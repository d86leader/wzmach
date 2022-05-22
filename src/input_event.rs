use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::{RawFd, FromRawFd, IntoRawFd, AsRawFd}};
use std::path::Path;

use input::{Libinput, LibinputInterface};
use input::event::gesture::{GestureEventCoordinates, GesturePinchEventTrait, GestureEventTrait, GestureEndEvent};
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

/* Gesture Information */

#[derive(PartialEq, Debug)]
pub enum Gesture {
    None,
    Swipe(SwipeGesture),
    Pinch(PinchGesture),
    Hold(HoldGesture),
}

#[derive(PartialEq, Debug)]
pub struct SwipeGesture {
    pub begin_time: u32,
    pub fingers: i32,
    pub dx: f64,
    pub dy: f64,
}

#[derive(PartialEq, Debug)]
pub struct PinchGesture {
    pub begin_time: u32,
    pub fingers: i32,
    pub scale: f64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(PartialEq, Debug)]
pub struct HoldGesture {
    pub begin_time: u32,
    pub fingers: i32,
}

#[derive(PartialEq, Debug)]
pub enum GestureState {
    /// Arg is current event time
    Ongoing(u32),
    /// Args are: event that just finished, time of finish
    Ended(Gesture, u32),
    /// Args are: event that just finished, time of finish
    Cancelled(Gesture, u32)
}

impl Gesture {
    fn update(&mut self, gest: &input::event::GestureEvent) -> GestureState {
        use input::event::gesture::*;
        match gest {
            GestureEvent::Swipe(sw) => match sw {
                GestureSwipeEvent::Begin(_ev) => {
                    *self = Gesture::Swipe(SwipeGesture {
                        begin_time: gest.time(),
                        fingers: gest.finger_count(),
                        dx: 0.0,
                        dy: 0.0,
                    });
                    GestureState::Ongoing(gest.time())
                },
                GestureSwipeEvent::Update(ev) => {
                    self.update_coords(ev);
                    GestureState::Ongoing(gest.time())
                },
                GestureSwipeEvent::End(ev) =>
                    self.end_gesture(ev),
                _ => {
                    eprintln!("WARNING: swipe update from the future");
                    GestureState::Ongoing(gest.time())
                },
            },
            GestureEvent::Pinch(pc) => match pc {
                GesturePinchEvent::Begin(_ev) => {
                    *self = Gesture::Pinch(PinchGesture {
                        begin_time: gest.time(),
                        fingers: gest.finger_count(),
                        scale: pc.scale(),
                        dx: 0.0,
                        dy: 0.0,
                    });
                    GestureState::Ongoing(gest.time())
                },
                GesturePinchEvent::Update(ev) => {
                    self.update_coords(ev);
                    self.update_scale(ev);
                    GestureState::Ongoing(gest.time())
                },
                GesturePinchEvent::End(ev) => {
                    self.update_scale(ev);
                    self.end_gesture(ev)
                },
                _ => {
                    eprintln!("WARNING: pinch update from the future");
                    GestureState::Ongoing(gest.time())
                },

            },
            GestureEvent::Hold(ho) => match ho {
                GestureHoldEvent::Begin(_ev) => {
                    *self = Gesture::Hold(HoldGesture {
                        begin_time: gest.time(),
                        fingers: gest.finger_count(),
                    });
                    GestureState::Ongoing(gest.time())
                },
                GestureHoldEvent::End(_ev) =>
                    GestureState::Ended(
                        std::mem::replace(self, Gesture::None),
                        gest.time(),
                    ),
                _ => {
                    eprintln!("WARNING: hold update from the future");
                    GestureState::Ongoing(gest.time())
                },
            }
            _ => {
                eprintln!("WARNING: event from the future");
                GestureState::Ongoing(gest.time())
            },
        }
    }

    fn update_coords(&mut self, upd: &dyn GestureEventCoordinates) {
        match *self {
            Gesture::Swipe(ref mut swipe) => {
                swipe.dx += upd.dx();
                swipe.dy += upd.dy();
            },
            Gesture::Pinch(ref mut pinch) => {
                pinch.dx += upd.dx();
                pinch.dy += upd.dy();
            },
            _ => eprintln!("ERROR: impossible coords update!")
        }
    }

    fn update_scale(&mut self, upd: &dyn GesturePinchEventTrait) {
        match *self {
            Gesture::Pinch(ref mut pinch) => {
                pinch.scale = upd.scale()
            },
            _ => eprintln!("ERROR: impossible scale update!")
        }
    }

    fn end_gesture<T>(&mut self, upd: &T) -> GestureState
        where T: GestureEventTrait + GestureEndEvent
    {
        if upd.cancelled() {
            GestureState::Cancelled(
                std::mem::replace(self, Gesture::None),
                upd.time(),
            )
        } else {
            GestureState::Ended(
                std::mem::replace(self, Gesture::None),
                upd.time(),
            )
        }
    }
}

/* The gesture iterator */

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

        let mut input = Libinput::new_with_udev(Interface);
        input.udev_assign_seat("seat0").unwrap();

        GestureProducer {
            input,
            current: Gesture::None,
        }
    }

    pub fn current(&self) -> &Gesture {
        &self.current
    }

    fn poll_events(&mut self) {
        use nix::poll::{poll, PollFlags, PollFd};
        let pollfd = PollFd::new(self.input.as_raw_fd(), PollFlags::POLLIN);
        poll(&mut [pollfd], -1).unwrap();
        self.input.dispatch().unwrap();
    }
}

impl Iterator for GestureProducer {
    type Item = GestureState;
    fn next(&mut self) -> Option<GestureState> {
        loop {
            match self.input.next() {
                Some(input::Event::Gesture(gest)) =>
                    break Some(self.current.update(&gest)),
                Some(_) => (),
                None => self.poll_events(),
            }
        }
    }
}
