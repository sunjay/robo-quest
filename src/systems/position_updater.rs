use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Velocity, Position};
use resources::FramesElapsed;

#[derive(SystemData)]
pub struct PositionData<'a> {
    frames: ReadExpect<'a, FramesElapsed>,
    velocities: ReadStorage<'a, Velocity>,
    positions: WriteStorage<'a, Position>,
}

/// Updates the position of each entity based on its velocity
pub struct PositionUpdater;

impl<'a> System<'a> for PositionUpdater {
    type SystemData = PositionData<'a>;

    fn run(&mut self, PositionData {frames, velocities, mut positions}: Self::SystemData) {
        let FramesElapsed(frames_elapsed) = *frames;
        let frames_elapsed = frames_elapsed as f64;

        for (Velocity(vel), Position(pos)) in (&velocities, &mut positions).join() {
            *pos = pos.offset((vel.x * frames_elapsed) as i32, (vel.y * frames_elapsed) as i32);
        }
    }
}
