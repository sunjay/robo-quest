use sdl2::rect::Rect;
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

pub struct Physics;

impl Physics {
    pub const GRAVITY_ACCEL: f64 = 0.0981; // pixels / frame^2
}

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

        for (entity, Position(pos), &BoundingBox {width, height}, &Mass(mass), AppliedForce(applied), Velocity(vel)) in (&*entities, &positions, &bounding_boxes, &masses, &applied_forces, &mut velocities).join() {
            let gravity = Vec2D {x: 0.0, y: Self::GRAVITY_ACCEL * mass};

            let mut collision_force = Vec2D {x: 0.0, y: 0.0};

            let mut bottom_bumper = Rect::new(0, 0, width - 2, 2);
            bottom_bumper.center_on(pos.offset(0, height as i32 / 2));
            let mut left_bumper = Rect::new(0, 0, 2, height - 2);
            left_bumper.center_on(pos.offset(-(width as i32 / 2), 0));
            let mut right_bumper = Rect::new(0, 0, 2, height - 2);
            right_bumper.center_on(pos.offset(width as i32 / 2, 0));

            for (other, &Position(other_pos), &BoundingBox {width, height}) in (&*entities, &positions, &bounding_boxes).join() {
                if entity == other {
                    continue;
                }
                let mut other_rect = Rect::new(0, 0, width, height);
                other_rect.center_on(other_pos);
                if bottom_bumper.has_intersection(other_rect) {
                    collision_force.y = -gravity.y;

                    // Stop moving down once we have reached the ground
                    if vel.y > 0.0 {
                        vel.y = 0.0;
                    }
                    // Apply friction if moving
                    if vel.x.abs() > 0.0 {
                        collision_force.x = -100.00 * vel.x.signum();
                    }
                }

                if left_bumper.has_intersection(other_rect) {
                    // Stop moving once we hit a wall
                    if applied.x < 0.0 {
                        collision_force.x = -applied.x;
                    }
                    if vel.x < 0.0 {
                        vel.x = 0.0;
                    }
                }

                if right_bumper.has_intersection(other_rect) {
                    // Stop moving once we hit a wall
                    if applied.x > 0.0 {
                        collision_force.x = -applied.x;
                    }
                    if vel.x > 0.0 {
                        vel.x = 0.0;
                    }
                }
            }

            let net_force = gravity + applied + collision_force;
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
