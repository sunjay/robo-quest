use std::collections::HashMap;

use sdl2::rect::Rect;
use specs::{Entity, System, Join, ReadExpect, ReadStorage, WriteStorage, Entities};
use nalgebra::{self as na, Isometry2, Point2};
use nphysics2d::{
    solver::SignoriniCoulombPyramidModel,
    object::{BodyHandle, ColliderHandle, Material},
    force_generator::ConstantAcceleration,
    volumetric::Volumetric,
    world::World,
};
use ncollide2d::{
    events::ProximityEvent,
    query::Proximity,
    shape::{Cuboid, Polyline, ShapeHandle, Shape},
};

use components::{Position, Velocity, Collisons, BoundingBox, Density, AppliedAcceleration};
use resources::FramesElapsed;
use math::{Vec2D, ToVec2D, ToPoint};
use map::LevelMap;

const COLLIDER_MARGIN: f64 = 0.01;

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
    applied_accel: ReadStorage<'a, AppliedAcceleration>,
    positions: WriteStorage<'a, Position>,
    velocities: WriteStorage<'a, Velocity>,
    collisions: WriteStorage<'a, Collisons>,
}

#[derive(Debug, Clone, Copy)]
enum SensorDirection {
    Top,
    Left,
    Right,
    Bottom,
}

pub struct Physics {
    world: World<f64>,
    /// Lookup table for entities kept in the physics engine. Needed for keeping track of which
    /// entities have been added and which have not beed added.
    bodies: HashMap<Entity, Body>,
    /// Lookup table for entities based on the sensor ColliderHandle. Needed for when sensor
    /// collisions are detected.
    sensors: HashMap<ColliderHandle, (Entity, SensorDirection)>,
}

impl Physics {
    pub const GRAVITY_ACCEL: f64 = 150.0; // pixels / frame^2

    pub fn new(fps: f64, map: &LevelMap) -> Self {
        let mut world = World::new();
        world.set_contact_model(SignoriniCoulombPyramidModel::new());
        world.set_gravity(Vec2D::y() * Self::GRAVITY_ACCEL);
        world.set_timestep(1.0/fps);

        let mut physics = Self {
            world,
            bodies: Default::default(),
            sensors: Default::default(),
        };

        for static_boundary in map.static_boundaries() {
            //TODO: Load friction from map file
            physics.add_static_polyline(static_boundary, 0.5);
        }

        physics
    }

    fn add_static_rect(&mut self, entity: Entity, rect: Rect, friction: f64) {
        let shape = Cuboid::new(Vec2D::new(
            rect.width() as f64 / 2.0 - COLLIDER_MARGIN,
            rect.height() as f64 / 2.0 - COLLIDER_MARGIN,
        ));
        let collider_handle = self.add_static_shape(shape, rect.center().to_vec2d(), friction);
        let body = Body::StaticCollider(collider_handle);
        self.insert_body(entity, body);
    }

    fn add_static_polyline(&mut self, points: &[Point2<f64>], friction: f64) {
        let shape = Polyline::new(points.to_vec());
        self.add_static_shape(shape, Vec2D::zeros(), friction);
    }

    fn add_static_shape(&mut self, shape: impl Shape<f64>, center: Vec2D, friction: f64) -> ColliderHandle {
        assert!(friction >= 0.0 && friction <= 1.0, "Friction must be between 0.0 and 1.0");

        self.world.add_collider(
            COLLIDER_MARGIN,
            ShapeHandle::new(shape),
            BodyHandle::ground(),
            Isometry2::new(center, na::zero()),
            Material::new(friction, friction / 2.0),
        )
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

    /// Adds a sensor to the given body and registers that it results in the given entity touching
    /// something in the given direction
    fn insert_sensor(
        &mut self,
        entity: Entity,
        body_handle: BodyHandle,
        direction: SensorDirection,
        offset_x: f64,
        offset_y: f64,
        width: f64,
        height: f64,
    ) {
        let sensor_geom = ShapeHandle::new(Cuboid::new(Vec2D::new(
            width / 2.0,
            height / 2.0,
        )));
        let collider_handle = self.world.add_sensor(
            sensor_geom,
            body_handle,
            Isometry2::new(Vec2D::new(offset_x, offset_y), na::zero()),
        );
        self.sensors.insert(collider_handle, (entity, direction))
            .map(|_| unreachable!("collider handle should have been unique in the physics engine"));
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
            applied_accel,
            mut positions,
            mut velocities,
            mut collisions,
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
                        0.0,
                    );

                    if let Some(&Velocity(vel)) = velocities.get(entity) {
                        self.world.rigid_body_mut(body_handle)
                            .expect("Body handle did not map to a rigid body")
                            .set_linear_velocity(vel);
                    }

                    // If this entity will need to detect collisions, attach sensors to it
                    if let Some(&Collisons {..}) = collisions.get(entity) {
                        let width = width as f64;
                        let height = height as f64;
                        let sensor_size = 2.0;
                        // Sensor needs to be a little narrower so that it doesn't falsely detect
                        // collisions on the corners of the bounding box
                        let sensor_scale_factor = 0.8;

                        // Positions each sensor at its position around the entity
                        self.insert_sensor(
                            entity,
                            body_handle,
                            SensorDirection::Top,
                            0.0,
                            -(height / 2.0 - sensor_size / 2.0),
                            width * sensor_scale_factor,
                            sensor_size,
                        );
                        self.insert_sensor(
                            entity,
                            body_handle,
                            SensorDirection::Bottom,
                            0.0,
                            height / 2.0 + sensor_size / 2.0,
                            width * sensor_scale_factor,
                            sensor_size,
                        );
                        self.insert_sensor(
                            entity,
                            body_handle,
                            SensorDirection::Left,
                            -(width / 2.0 + sensor_size / 2.0),
                            0.0,
                            sensor_size,
                            height * sensor_scale_factor,
                        );
                        self.insert_sensor(
                            entity,
                            body_handle,
                            SensorDirection::Right,
                            width / 2.0 - sensor_size / 2.0,
                            0.0,
                            sensor_size,
                            height * sensor_scale_factor,
                        );
                    }
                },
                // Static collider
                None => {
                    //TODO: Friction(f64) should be an optional Component which defaults to 0.0 if not present
                    self.add_static_rect(entity, Rect::from_center(pos, width, height), 0.5);
                },
            }
        }

        // Apply accelerations to every rigid body (if any accelerations have been applied)
        let body_accel = self.bodies.iter()
            .filter_map(|(&entity, body)| match (body, applied_accel.get(entity)) {
                (&Body::RigidBody {body_handle, ..}, Some(&AppliedAcceleration(accel))) => {
                    Some((body_handle, accel))
                },
                _ => None,
            });

        let mut force_handles = Vec::new();
        for (body_handle, accel) in body_accel {
            let mut generator = ConstantAcceleration::new(accel, 0.0);
            generator.add_body_part(body_handle);
            let force_handle = self.world.add_force_generator(generator);
            force_handles.push(force_handle);
        }

        for _ in 0..frames_elapsed {
            self.world.step();
        }

        for force_handle in force_handles {
            self.world.remove_force_generator(force_handle);
        }

        // Handle sensor events to determine which collisions have occurred
        for ProximityEvent {collider1, collider2, new_status, ..} in self.world.proximity_events() {
            for sensor in &[self.sensors.get(collider1), self.sensors.get(collider2)] {
                if let Some(&(entity, direction)) = sensor {
                    let collisions = collisions.get_mut(entity)
                        .expect("Body with sensors should have a Collisons component");
                    let status = match new_status {
                        Proximity::WithinMargin | Proximity::Intersecting => true,
                        Proximity::Disjoint => false,
                    };
                    match direction {
                        SensorDirection::Top => collisions.top = status,
                        SensorDirection::Left => collisions.left = status,
                        SensorDirection::Right => collisions.right = status,
                        SensorDirection::Bottom => collisions.bottom = status,
                    }
                }
            }
        }

        // Update every tracked entity with the latest values from the physics engine
        // We don't need to update static colliders because they do not move
        for (&entity, body) in self.bodies.iter() {
            if let &Body::RigidBody {body_handle, ..} = body {
                let Position(position) = positions.get_mut(entity)
                    .expect("Rigid body should have had a position");
                let Velocity(velocity) = velocities.get_mut(entity)
                    .expect("Rigid body should have had a velocity");

                let physics_body = self.world.rigid_body(body_handle)
                    .expect("Body handle did not map to a rigid body");
                *position = physics_body.position().translation.vector.to_point();
                *velocity = physics_body.velocity().linear;
            }
        }
    }
}
