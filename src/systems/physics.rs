use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Velocity, Position};
use resources::FramesElapsed;

#[derive(SystemData)]
pub struct PhysicsData<'a> {
    frames: ReadExpect<'a, FramesElapsed>,
    velocities: ReadStorage<'a, Velocity>,
    positions: WriteStorage<'a, Position>,
}

pub struct Physics;

impl<'a> System<'a> for Physics {
    type SystemData = PhysicsData<'a>;

    fn run(&mut self, PhysicsData {frames, velocities, mut positions}: Self::SystemData) {
        let FramesElapsed(frames_elapsed) = *frames;

        for (Velocity(vel), Position(pos)) in (&velocities, &mut positions).join() {
            *pos = pos.offset(vel.x() * frames_elapsed as i32, vel.y() * frames_elapsed as i32);
        }
    }
}
