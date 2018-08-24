use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Density, AppliedForce, KeyboardControlled};
use resources::GameKeys;

use super::physics::Physics;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    keys: ReadExpect<'a, GameKeys>,
    densities: ReadStorage<'a, Density>,
    applied_forces: WriteStorage<'a, AppliedForce>,
    keyboard_controlled: ReadStorage<'a, KeyboardControlled>,
}

pub struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, KeyboardData {keys, densities, mut applied_forces, keyboard_controlled}: Self::SystemData) {
        //FIXME: This needs to be redone now that we track density instead of mass
        for (&Density(mass), AppliedForce(ref mut force), _) in (&densities, &mut applied_forces, &keyboard_controlled).join() {
            // Assuming that only a single arrow key can be held down at a time.
            if keys.right_arrow {
                force.x = mass * 1.0; // kg * pixels / frame^2
            }
            else if keys.left_arrow {
                force.x = mass * -1.0; // kg * pixels / frame^2
            }
            else {
                force.x = mass * 0.0; // kg * pixels / frame^2
            }

            if keys.b {
                // Must overcome gravity and then accelerate even more
                force.y = -(Physics::GRAVITY_ACCEL + 0.5) * mass; // kg * pixels / frame^2
            }
            else {
                force.y = 0.0; // kg * pixels / frame^2
            }
        }
    }
}
