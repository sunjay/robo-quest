use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Collisons, AppliedAcceleration, KeyboardControlled};
use resources::GameKeys;

use super::physics::Physics;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    keys: ReadExpect<'a, GameKeys>,
    collisions: ReadStorage<'a, Collisons>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
    applied_accel: WriteStorage<'a, AppliedAcceleration>,
}

#[derive(Default)]
pub struct Keyboard {
    jumping: bool,
}

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {keys, collisions, keyboard_controlled, mut applied_accel}: Self::SystemData) {
        for (AppliedAcceleration(ref mut accel), collisions, _) in (&mut applied_accel, &collisions, &keyboard_controlled).join() {
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

            if keys.b && collisions.bottom && !self.jumping {
                // Must overcome gravity and then accelerate even more
                accel.y = -(Physics::GRAVITY_ACCEL + 4000.0);
                self.jumping = true;
            }
            else {
                accel.y = 0.0;
                self.jumping = false;
            }
        }
    }
}
