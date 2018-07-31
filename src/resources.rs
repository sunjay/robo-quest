//! ECS Resources for use by various systems

use sdl2::keyboard::{KeyboardState, Scancode};

/// Resource that represents the number of frames elapsed since the last time all of the systems
/// were run. Value is guaranteed to be greater than or equal to 1.
/// Often this will be just 1 but it may be greater if there is lag or if a system takes too long.
pub struct FramesElapsed(pub usize);

/// Resource that represents which keys are currently pressed.
///
/// Each boolean is true if the key is pressed and false otherwise
#[derive(Debug, Clone)]
pub struct GameKeys {
    pub up_arrow: bool,
    pub down_arrow: bool,
    pub left_arrow: bool,
    pub right_arrow: bool,
    pub menu: bool,
    pub select: bool,
    pub start: bool,
    pub volume_down: bool,
    pub volume_up: bool,
    pub x: bool,
    pub y: bool,
    pub a: bool,
    pub b: bool,
    pub light_key_1: bool,
    pub light_key_2: bool,
    pub light_key_3: bool,
    pub light_key_4: bool,
    pub light_key_5: bool,
}

impl<'a> From<KeyboardState<'a>> for GameKeys {
    fn from(keys: KeyboardState) -> Self {
        // From mapping: https://github.com/clockworkpi/Keypad#keymaps
        Self {
            up_arrow: keys.is_scancode_pressed(Scancode::Up),
            down_arrow: keys.is_scancode_pressed(Scancode::Down),
            left_arrow: keys.is_scancode_pressed(Scancode::Left),
            right_arrow: keys.is_scancode_pressed(Scancode::Right),
            menu: keys.is_scancode_pressed(Scancode::Escape),
            select: keys.is_scancode_pressed(Scancode::Space),
            start: keys.is_scancode_pressed(Scancode::Return),
            volume_down: keys.is_scancode_pressed(Scancode::Minus),
            //FIXME: This probably isn't the right key
            volume_up: keys.is_scancode_pressed(Scancode::KpPlus),
            x: keys.is_scancode_pressed(Scancode::U),
            y: keys.is_scancode_pressed(Scancode::I),
            a: keys.is_scancode_pressed(Scancode::J),
            b: keys.is_scancode_pressed(Scancode::K),
            light_key_1: keys.is_scancode_pressed(Scancode::Home),
            light_key_2: keys.is_scancode_pressed(Scancode::PageUp),
            light_key_3: false, //FIXME: No way to check if Shift key pressed
            light_key_4: keys.is_scancode_pressed(Scancode::PageDown),
            light_key_5: keys.is_scancode_pressed(Scancode::End),
        }
    }
}
