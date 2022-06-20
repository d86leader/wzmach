use std::path::{Path, PathBuf};

use bpaf::construct;

mod action;
mod common;
mod config;
mod gesture_event;
mod input_producer;

#[derive(PartialEq, Eq, Debug, Clone)]
enum Opts {
    Run { config_path: Option<String> },
    DebugConfig { path: String },
    DebugGestures,
    DebugEvents,
}

fn parse_opts() -> Opts {
    let debug_config = bpaf::command(
        "debug-config",
        Some("Parse config file and check it for errors"),
        bpaf::Info::default()
            .descr("Parse and print a config file")
            .for_parser(bpaf::positional("FILENAME")),
    )
    .map(|path| Opts::DebugConfig { path });

    let debug_gestures = bpaf::command(
        "debug-gestures",
        Some("Print all completed gestures but execute no actions"),
        bpaf::Info::default()
            .descr("If you see nothing you need to `export RUST_LOG=debug` or `trace`")
            .for_parser(bpaf::Parser::pure(Opts::DebugGestures)),
    );

    let debug_events = bpaf::command(
        "debug-events",
        Some("Print all incoming libinput gesture events and execute nothing"),
        bpaf::Info::default()
            .descr("If you see nothing you need to `export RUST_LOG=debug` or `trace`")
            .for_parser(bpaf::Parser::pure(Opts::DebugEvents)),
    );

    let config_path = bpaf::long("config")
        .help("Path to a config file to use instead of default")
        .argument("PATH")
        .optional();
    let run = construct!(Opts::Run { config_path });

    let parser = debug_config
        .or_else(debug_gestures)
        .or_else(debug_events)
        .or_else(run);

    bpaf::Info::default()
        .descr("Touchpad gesture engine")
        .for_parser(parser)
        .run()
}

fn main() {
    env_logger::init();
    log::trace!("initialized logging");

    match parse_opts() {
        Opts::DebugConfig { path } => {
            let c = config::Config::load(path);
            println!("{:?}", c);
        }

        Opts::DebugGestures => debug_events(),

        Opts::DebugEvents => {
            let producer = input_producer::GestureProducer::new();
            log::debug!("Created input connection");
            for event in producer {
                log::debug!("update: {:?}", event);
            }
        }

        Opts::Run { config_path } => run(config_path),
    }
}

/// Builds a [`PathBuf`] from string literals and identifiers. Slash separators
/// are optional (`/`) between items. A beginning slash is significant as it
/// will push `/` to the buffer first. A trailing slash at the end of an
/// invocation is ignored. String literals are pushed into the buffer directly
/// as path components, and identifiers are trated as environment variable
/// names. If any of the environment variables used in the invocation
/// are undefined, the expression will resolve to `None`.
macro_rules! env_path {
    (@root $buffer:ident | / $($tail:tt)*) => {
        $buffer.push("/");
        env_path!(@push $buffer | $($tail)*)
    };
    (@root $buffer:ident | $($tail:tt)*) => {
        env_path!(@push $buffer | $($tail)*)
    };
    (@push $buffer:ident | $(/)?) => {};
    (@push $buffer:ident | $(/)? $segment:literal $($tail:tt)*) => {
        $buffer.push($segment);
        env_path!(@push $buffer | $($tail)*)
    };
    (@push $buffer:ident | $(/)? $variable:ident $($tail:tt)*) => {
        $buffer.push(std::env::var_os(stringify!($variable))?);
        env_path!(@push $buffer | $($tail)*)
    };
    ($($munch:tt)*) => {(|| {
        use std::path::PathBuf;
        let mut buffer = PathBuf::new();
        env_path!(@root buffer | $($munch)*);
        Some(buffer)
    })()};
}

fn run(command_config: Option<String>) {
    let default_config_path: &Path = Path::new("/etc/wzmach/config.ron");
    let config_paths = &[
        command_config.map(PathBuf::from),
        env_path!(XDG_CONFIG_HOME / "wzmach.ron"),
        env_path!(XDG_CONFIG_HOME / "wzmach" / "config.ron"),
        env_path!(HOME / ".config" / "wzmach.ron"),
        env_path!(HOME / ".config" / "wzmach" / "config.ron"),
    ];

    // let is_root = nix::unistd::getuid().is_root();

    if default_config_path.is_file() {
        log::warn!(
            "The default config expected at '{}' was not found!",
            default_config_path.to_string_lossy()
        );

        // TODO: Create the default config file. Requires `serde::Serialize` on `config::Config`.
        // if is_root {
        //     log::info!("Creating the default config file.");
        // }
    }

    // find config path

    let config_path = config_paths.iter().flatten().find(|path| path.is_file());

    // read config

    // TODO: Improve logging here, this doesn't say it's using default.
    let config = config_path
        .map(config::Config::load)
        .and_then(Result::ok)
        .unwrap_or_default();

    let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();

    // run

    let (triggers, mut actions) = config.make_triggers(is_wayland);

    log::info!("Starting up");
    let producer = input_producer::GestureProducer::new();
    log::debug!("Created input connection");
    let events = gesture_event::EventAdapter::new(producer, &triggers);
    for action_inds in events {
        for index in action_inds {
            match actions[index].execute() {
                Ok(()) => (),
                Err(action::ActionError(msg)) => log::error!("{}", msg),
            }
        }
    }
}

fn debug_events() {
    let producer = input_producer::GestureProducer::new();
    log::debug!("Created input connection");
    let triggers = {
        let mut ts = Vec::new();
        use common::{Direction, PinchDirection, RotateDirection};
        use gesture_event::trigger::*;
        for fingers in 2..5 {
            for repeated in [false, true] {
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Up,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Down,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Left,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Swipe(CardinalTrigger {
                    fingers,
                    direction: Direction::Right,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Pinch(PinchTrigger {
                    fingers,
                    direction: PinchDirection::In,
                    scale: 1.3,
                    repeated,
                }));
                ts.push(Trigger::Pinch(PinchTrigger {
                    fingers,
                    direction: PinchDirection::Out,
                    scale: 1.3,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Up,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Down,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Left,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Shear(CardinalTrigger {
                    fingers,
                    direction: Direction::Right,
                    distance: 100.0,
                    repeated,
                }));
                ts.push(Trigger::Rotate(RotateTrigger {
                    fingers,
                    direction: RotateDirection::Anticlockwise,
                    distance: 45.0,
                    repeated,
                }));
                ts.push(Trigger::Rotate(RotateTrigger {
                    fingers,
                    direction: RotateDirection::Clockwise,
                    distance: 45.0,
                    repeated,
                }));
            }
            ts.push(Trigger::Hold(HoldTrigger { fingers, time: 50 }));
        }
        ts
    };
    let events = gesture_event::EventAdapter::new(producer, &triggers);
    for event in events {
        for i in event {
            log::debug!("triggered: {:?}", triggers[i]);
        }
    }
}
