# Wzmach

Wzmach is a gesture engine for Linux, compatible with both _Wayland_ and _X11_. It
allows you to map keyboard actions to different touchpad figures, such as
multi-finger swipes, pinches and shears.

Pronounciation: [ `[vzmax]` ](https://forvo.com/search/%D0%B2%D0%B7%D0%BC%D0%B0%D1%85/)

## Installation

If you don't have a rust toolchain, you can grab a statically-linked binary
release in the [releases page](https://github.com/d86leader/wzmach/releases). You can put that file anywhere in
your PATH, for example into `~/.local/bin`.

**Important**: for wzmach to work you need to give it permission to observe and
emit input events. Do it with these commands:

    cd ~/.local/bin # or the other place where you put the file
    sudo chown "$USER:input" wzmach
    sudo chmod g+s wzmach
    # warning: cp does not preserve permissions, so don't copy it after the modifications

After that you can verify that the permissions are granted by using `stat`:

    stat wzmach
    # should produce a line like:
    # Access: (2754/-rwxr-sr--)  Uid: ( 1000/    USERNAME)   Gid: (  322/   input)

And verify that your system respects these permissions by running:

    RUST_LOG=debug wzmach debug-events
    # try to perform multi-finger gestures and see a lot of output. If the output is empty, something is broken!

### Installation with cargo

Same as the above, but you can grab the source code and build it with

    cargo build --release

You can then install with `cargo install` and grant the permissions by hand, or use

    make install

which executes the above commands for you.

## Configuration

Wzmach reads configuration from `$XDG_CONFIG_HOME/wzmach/config.ron`, which on
your system is probably at `~/.config/wzmach/config.ron`. You can put the
[default config](./config.ron) there, which provides tab and desktop movement
in the style of the old libinput-gestures.

You can then edit that config file to add or replace your gestures. After
editing this file you need to restart wzmach.

The default config provides description of top-level fields. Below I describe
the available gestures and actions.

#### KeyboardInput

Send keyboard events when a gesture is executed. First, it presses all the
modifier keys in the order they appear. Then, it clicks (presses and depresses)
all the sequence keys one after another. After that, all modifier keys get
depressed in the reverse order.

    // Example: start omni-completion in vim
    KeyboardInput (

        // These keys are pressed for all the duration of the action
        modifiers: ["RightControl"],

        // There keys are pressed one at a time
        sequence: ["X", "O"],

    )

#### InlineScript

Run a command in the `sh` shell. All wildcards and special symbols get
interpreted like the shell always does.

    // Example: toggle a scroll lock LED (works in X11 only)
    InlineScript (
        code: r#"
            on=$(xset -q | grep 'Scroll Lock:' | cut -d ":" -f 7)
            echo $on
            if [ $on == "off" ]; then
               xset led named "Scroll Lock"
            else
               xset -led named "Scroll Lock"
            fi
        "#,
    ),

The example above features a raw string literal. It's delimited by `r###"` and
`"###` with any number of `#` symbols, and any symbol can appear inside this
string. You can use raw string literals anywhere a string is expected in
config, but it's most useful with this and the next action.

#### ExecuteCommand

Like `InlineScript`, but skip the shell and invoke the command literally.
The difference between this and that is like a difference between `system` and
`execv`.

    // Example: unclutter desktop in KDE
    ExecuteCommand (
        // Path can be absolute (/usr/bin/qdbus-qt5) or just a command name. In
        // the second case it's looked up in $PATH
        path: "qdbus-qt5",
        args: [
            "org.kde.KWin",
            "/KWin",
            "unclutterDesktop",
        ],
        // Actually this example doesn't work because of
        // https://bugs.freedesktop.org/show_bug.cgi?id=52202
    ),

Note that you can use this instead of the previous action. In fact, this is
what you should do if you want your command to run in bash or zsh instead of
sh.

    ExecuteCommand (
        path: "/usr/bin/env",
        args: [
            "bash",
            "-c",
            r##" your command goes here "##,
        ],
    ),

#### Swipe

Swipe is moving all of your fingers together in one direction.

Example configuration:

    (
        // What triggers the action
        trigger: Swipe (

            // Amount of fingers, from 1 to infinity in theory, and from 3 to
            // 4 or 5 in practice
            fingers: 3,

            // Direction of the swipe: Up, Down, Left or Right
            direction: Up,

            // Can this gesture be repeated multiple times without lifting the
            // fingers? true or false
            repeated: false,

        ),

        // The action to execute upon trigger. Use UinputAction, CommandAction
        // or ShellCommandAction here
        action: UinputAction (
            modifiers: ["RightControl"],
            sequence: ["T"],
        )
    ),

#### Shear

Shear is when you rest your digits and move your thumb; or when you move your
digits and thumb in different directions. It is very easy to confuse vertical
shears and pinches, so you probably don't want to create triggers for both.

Example:

    (
        trigger: Shear (

            // Amount of fingers, from 2 to infinity in theory, and from 2 to
            // 4 or 5 in practice. 3 fingers means two digits + 1 thumb
            fingers: 4,

            // Direction of your thumb to move. Up, Down, Left or Right
            direction: Left,

            // Can this gesture be repeated multiple times without lifting the
            // fingers? true or false
            // In practice I run out of thumb before I can trigger it twice.
            repeated: false,

        ),
        action: UinputAction (
            modifiers: ["LeftAlt"],
            sequence: ["Tab"],
        )
    ),

#### Pinch

Pinch is when you move your thumb towards your digits, or away from them, as if
you want to pinch the surface of your touchpad. You may also know this gesture
as "zoom the picture on your phone".

Example:

    (
        trigger: Pinch (

            // Amount of fingers, from 2 to infinity in theory, and from 2 to
            // 4 or 5 in practice. 3 fingers means two digits + 1 thumb
            fingers: 2,

            // Direction of the pinch in terms of zooming the picture: In or Out
            direction: In,

            // Can this gesture be repeated multiple times without lifting the
            // fingers? true or false
            repeated: false,

        ),
        action: UinputAction (
            modifiers: ["RightControl"],
            sequence: ["Equal"],
        )
    ),

#### Rotate

Rotate is when you rotate your fingers in one direction around a "center of
mass" of all your fingers; like rotating a map on your phone. It is extremely
easy to confuse shears and rotations, so you probably don't want to create
triggers for both.

Example:

    (
        trigger: Rotate (

            // Amount of fingers, from 2 to infinity in theory, and from 2 to
            // 4 or 5 in practice. 3 fingers means two digits + 1 thumb
            fingers: 2,

            // Direction of the fingers' rotation: Clockwise or Anticlockwise
            direction: Anticlockwise,

            // Can this gesture be repeated multiple times without lifting the
            // fingers? true or false
            repeated: true,

        ),
        action: UinputAction (
            modifiers: ["RightControl"],
            sequence: ["PageUp"],
        )
    )

#### Hold

Holding several digits on touchpad without movement. In practice it works like
shit, both because of libinput weirdness and because a proper implementation is
way harder.

Example:

    (
        trigger: Hold (

            // Amount of fingers, from 2 to infinity in theory, and from 2 to
            // 4 or 5 in practice.
            fingers: 4,

        ),
        action: UinputAction (
            modifiers: ["RightControl", "RightAlt"],
            sequence: ["Esc"],
        )
    ),

## FAQ

#### Does wzmach work on wayland?

Yes! The goal of developing wzmach was for me to finally migrate to wayland.
This is also the reason I still haven't implemented window-local gestures, as
it's untrivial and DE-dependent without x-things.

#### Does wzmach work on X11?

Yes! Wzmach minimally relies only on libinput and uinput which is indenedent of
wayland/X11. So unless you are on a linux version with synaptics and no virtual
devices, it will work.

#### Can I use 2 finger swipes, for example to emulate MacOS's browser gestures?

Not presently, since libinput overrides those with scrolling event. In the
future I want to give the ability to interpret scrolling events as gestures.

#### What are the differences from touchegg?

1. Wzmach works on wayland
2. Wzmach has a different trigger mechanism, more similar to that of
   BetterTouchTool for mac: gestures are executed immediately after the
   threshold has passed, and you can execute multiple gestures without lifting
   your fingers
3. Touchegg has cool animations, and for wzmach they are not even planned
4. Touchegg has a GUI for configuration, and for wzmach it is only in plans
5. Touchegg has advanced actions like directly switching your desktop. This
   relies on X11, and so is not possible for wzmach

#### What is this configuration language? Did you make it up yourself?

It's called [RON](https://github.com/ron-rs/ron) and no, although the idea for
it lies on the surface.

##### A line for vim whitespace detection
vim: ts=4 sw=4 sts=4
