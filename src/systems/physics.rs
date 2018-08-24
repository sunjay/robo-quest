use std::collections::HashMap;

use sdl2::rect::Rect;
use specs::{Entity, System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};
use nalgebra::{self as na, Isometry2};
use nphysics2d::{
    solver::{SignoriniCoulombPyramidModel, IntegrationParameters},
    object::{BodyHandle, ColliderHandle, Material, BodySet},
    force_generator::ForceGenerator,
    volumetric::Volumetric,
    math::Force,
    world::World,
};
use ncollide2d::shape::{Cuboid, ShapeHandle};

use components::{Position, Velocity, BoundingBox, Density, AppliedForce};
use resources::FramesElapsed;
use math::{Vec2D, ToVec2D, ToPoint};

const COLLIDER_MARGIN: f64 = 0.01;

struct ApplyForces {
    body_forces: Vec<(BodyHandle, Vec2D)>,
}

impl ForceGenerator<f64> for ApplyForces {
    fn apply(&mut self, _: &IntegrationParameters<f64>, bodies: &mut BodySet<f64>) -> bool {
        for &(body_handle, force) in self.body_forces.iter() {
            if !bodies.contains(body_handle) {
                // This means that a entity was removed by its AppliedForce component still hung
                // around somehow.
                unreachable!("Body was removed from the world before a force could be applied");
            }

            let mut physics_body = bodies.body_part_mut(body_handle);
            physics_body.apply_force(&Force::linear(force));
        }

        // If false, the physics world will remove this force generator after this call.
        // We don't want this in case world.step() is called more than once.
        true
    }
}

#[derive(Debug)]
enum Body {
    /// Represents a rigid body in the physics engine.
    RigidBody {
        body_handle: BodyHandle,
        collider_handle: ColliderHandle,
    },
    /// Represents a static collider in the physics engine.
    StaticCollider(ColliderHandle),
}

#[derive(SystemData)]
pub struct PhysicsData<'a> {
    entities: Entities<'a>,
    frames: ReadExpect<'a, FramesElapsed>,
    densities: ReadStorage<'a, Density>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    applied_forces: ReadStorage<'a, AppliedForce>,
    positions: WriteStorage<'a, Position>,
    velocities: WriteStorage<'a, Velocity>,
}

pub struct Physics {
    world: World<f64>,
    bodies: HashMap<Entity, Body>,
}

impl Physics {
    pub const GRAVITY_ACCEL: f64 = 150.0; // pixels / frame^2

    pub fn new(fps: f64) -> Self {
        let mut world = World::new();
        world.set_contact_model(SignoriniCoulombPyramidModel::new());
        world.set_gravity(Vec2D::y() * Self::GRAVITY_ACCEL);
        world.set_timestep(1.0/fps);
        Self {
            world,
            bodies: Default::default(),
        }
    }

    fn add_static_rect(&mut self, entity: Entity, rect: Rect, friction: f64) {
        assert!(friction >= 0.0 && friction <= 1.0, "Friction must be between 0.0 and 1.0");

        let body = Body::StaticCollider(self.world.add_collider(
            COLLIDER_MARGIN,
            ShapeHandle::new(Cuboid::new(Vec2D::new(
                rect.width() as f64 / 2.0 - COLLIDER_MARGIN,
                rect.height() as f64 / 2.0 - COLLIDER_MARGIN,
            ))),
            BodyHandle::ground(),
            Isometry2::new(rect.center().to_vec2d(), na::zero()),
            Material::new(friction, friction / 2.0),
        ));
        self.insert_body(entity, body);
    }

    fn add_rigid_body(&mut self, entity: Entity, rect: Rect, density: f64, friction: f64) -> BodyHandle {
        assert!(friction >= 0.0 && friction <= 1.0, "Friction must be between 0.0 and 1.0");

        let geom = ShapeHandle::new(Cuboid::new(Vec2D::new(
            rect.width() as f64 / 2.0 - COLLIDER_MARGIN,
            rect.height() as f64 / 2.0 - COLLIDER_MARGIN,
        )));
        let body_handle = self.world.add_rigid_body(
            Isometry2::new(rect.center().to_vec2d(), na::zero()),
            geom.inertia(density),
            geom.center_of_mass(),
        );
        let body = Body::RigidBody {
            body_handle,
            collider_handle: self.world.add_collider(
                COLLIDER_MARGIN,
                geom,
                body_handle,
                Isometry2::identity(),
                Material::new(friction, friction / 2.0),
            ),
        };
        self.insert_body(entity, body);
        body_handle
    }

    fn insert_body(&mut self, entity: Entity, body: Body) {
        self.bodies.insert(entity, body)
            .map(|_| unreachable!("an entity was added to the physics engine more than once"));
    }
}

impl<'a> System<'a> for Physics {
    type SystemData = PhysicsData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let PhysicsData {
            entities,
            frames,
            densities,
            bounding_boxes,
            applied_forces,
            mut positions,
            mut velocities,
        } = data;
        let FramesElapsed(frames_elapsed) = *frames;

        for (entity, &BoundingBox {width, height}, &Position(pos)) in (&*entities, &bounding_boxes, &positions).join() {
            // Check if already added
            if self.bodies.contains_key(&entity) {
                continue;
            }

            // Determine if this is a static body or not
            let density = densities.get(entity);
            match density {
                // Rigid body
                Some(&Density(density)) => {
                    let body_handle = self.add_rigid_body(
                        entity,
                        Rect::from_center(pos, width, height),
                        density,
                        //TODO: Friction(f64) should be an optional Component which defaults to 0.0 if not present
                        0.5,
                    );

                    let vel = velocities.get(entity);
                    if let Some(&Velocity(vel)) = vel {
                        self.world.rigid_body_mut(body_handle)
                            .expect("Body handle did not map to a rigid body")
                            .set_linear_velocity(vel);
                    }
                },
                // Static collider
                None => {
                    //TODO: Friction(f64) should be an optional Component which defaults to 0.0 if not present
                    self.add_static_rect(entity, Rect::from_center(pos, width, height), 0.5);
                },
            }
        }

        // Apply forces to every rigid body (if any forces have been applied)
        let body_forces = self.bodies.iter()
            .filter_map(|(&entity, body)| match (body, applied_forces.get(entity)) {
                (&Body::RigidBody {body_handle, ..}, Some(&AppliedForce(force))) => {
                    Some((body_handle, force))
                },
                _ => None,
            })
            .collect();
        let force_generator = ApplyForces {body_forces};
        let force_handle = self.world.add_force_generator(force_generator);

        for _ in 0..frames_elapsed {
            self.world.step();
        }

        self.world.remove_force_generator(force_handle);

        // Update every tracked entity with the latest values from the physics engine
        // We don't need to update static colliders because they do not move
        for (&entity, body) in self.bodies.iter() {
            if let &Body::RigidBody {body_handle, ..} = body {
                let position = positions.get_mut(entity)
                    .expect("Rigid body should have had a position");
                let velocity = velocities.get_mut(entity)
                    .expect("Rigid body should have had a velocity");

                let physics_body = self.world.rigid_body(body_handle)
                    .expect("Body handle did not map to a rigid body");
                position.0 = physics_body.position().translation.vector.to_point();
                velocity.0 = physics_body.velocity().linear;
            }
        }
    }
}
