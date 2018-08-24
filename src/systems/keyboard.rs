use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{AppliedAcceleration, KeyboardControlled};
use resources::GameKeys;

use super::physics::Physics;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    keys: ReadExpect<'a, GameKeys>,
    applied_accel: WriteStorage<'a, AppliedAcceleration>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
}

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {keys, mut applied_accel, keyboard_controlled}: Self::SystemData) {
        for (AppliedAcceleration(ref mut accel), _) in (&mut applied_accel, &keyboard_controlled).join() {
            // Assuming that only a single arrow key can be held down at a time.
            if keys.right_arrow {
                accel.x = 100.0;
            }
            else if keys.left_arrow {
                accel.x = -100.0;
            }
            else {
                accel.x = 0.0;
            }

            if keys.b {
                // Must overcome gravity and then accelerate even more
                accel.y = -(Physics::GRAVITY_ACCEL + 1000.0);
            }
            else {
                accel.y = 0.0;
            }
        }
    }
}
