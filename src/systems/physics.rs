use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};

use components::{Velocity, Position, Mass, BoundingBox, AppliedForce, TerminalVelocity};
use resources::FramesElapsed;
use Vec2D;

#[derive(SystemData)]
pub struct PhysicsData<'a> {
    entities: Entities<'a>,
    frames: ReadExpect<'a, FramesElapsed>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    positions: ReadStorage<'a, Position>,
    masses: ReadStorage<'a, Mass>,
    applied_forces: ReadStorage<'a, AppliedForce>,
    terminal_velocities: ReadStorage<'a, TerminalVelocity>,
    velocities: WriteStorage<'a, Velocity>,
}

const GRAVITY_ACCEL: f64 = 0.0981; // pixels / frame^2

pub struct Physics;

impl<'a> System<'a> for Physics {
    type SystemData = PhysicsData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let PhysicsData {
            entities,
            frames,
            bounding_boxes,
            positions,
            masses,
            applied_forces,
            terminal_velocities,
            mut velocities,
        } = data;
        let FramesElapsed(frames_elapsed) = *frames;
        let frames_elapsed = frames_elapsed as f64;

        for (entity, Position(pos), BoundingBox {width, height}, &Mass(mass), AppliedForce(applied), Velocity(vel)) in (&*entities, &positions, &bounding_boxes, &masses, &applied_forces, &mut velocities).join() {
            let gravity = Vec2D {x: 0.0, y: GRAVITY_ACCEL * mass};
            let net_force = gravity + applied;

            let acceleration = net_force / mass;
            // vf = vi + a*t
            *vel += acceleration * frames_elapsed;
            if let Some(&TerminalVelocity {x, y}) = terminal_velocities.get(entity) {
                if vel.x.abs() > x {
                    vel.x = vel.x.signum() * x;
                }
                if vel.y.abs() > y {
                    vel.y = vel.y.signum() * y;
                }
            }
        }
    }
}
