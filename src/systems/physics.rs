use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Velocity, Position};
use resources::FramesElapsed;

pub struct PositionUpdater;

impl<'a> System<'a> for PositionUpdater {
    type SystemData = (ReadExpect<'a, FramesElapsed>, ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, (frames, velocities, mut positions): Self::SystemData) {
        let FramesElapsed(frames_elapsed) = *frames;

        for (Velocity(vel), Position(pos)) in (&velocities, &mut positions).join() {
            *pos = pos.offset(vel.x() * frames_elapsed as i32, vel.y() * frames_elapsed as i32);
        }
    }
}
