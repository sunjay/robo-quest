use sdl2::rect::Point;
use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Velocity, KeyboardControlled};
use resources::GameKeys;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    keys: ReadExpect<'a, GameKeys>,
    velocities: WriteStorage<'a, Velocity>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
}

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {keys, mut velocities, keyboard_controlled}: Self::SystemData) {
        for (Velocity(ref mut vel), _) in (&mut velocities, &keyboard_controlled).join() {
            let y = vel.y();
            // Assuming that only a single arrow key can be held down at a time.
            if keys.right_arrow {
                *vel = Point::new(3, y);
            }
            else if keys.left_arrow {
                *vel = Point::new(-3, y);
            }
            else {
                *vel = Point::new(0, y);
            }
        }
    }
}
