use sdl2::rect::{Rect, Point};
use specs::{VecStorage, NullStorage, HashMapStorage};

use texture_manager::TextureId;
use ::{Vec2D};

/// Represents the XY world coordinates of the center of an entity.
///
/// This is distinct from the screen coordinates which are bounded by the size of the display.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

/// Represents the bounding box centered around an entity's position. BoundingBox alone doesn't
/// mean much without a Position also attached to the entity.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct BoundingBox {
    pub width: u32,
    pub height: u32,
}

/// The current velocity of an entity. Usually not manipulated directory by anything other
/// than the physics engine. Use AppliedForce to move entities instead.
///
/// Keep in mind that the coordinate system has +x to the right and +y downwards.
/// Unit: pixels / frame
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Velocity(pub Vec2D);

/// The maximum possible velocity that this entity can travel at in each direction. If this is
/// provided, the magnitude of the velocity will be clamped to be at most this value.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct TerminalVelocity {
    pub x: f64,
    pub y: f64,
}

/// Apply a force to a given entity. This is combined in the physics engine with other forces such
/// as gravity to create the net force acting on an object.
///
/// Keep in mind that the coordinate system has +x to the right and +y downwards.
/// Unit: kg * pixels / frame^2
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct AppliedForce(pub Vec2D);

/// Represents the mass in kg of an entity. Entities without a specified mass are assumed to be
/// infinitely heavy. Only entities with a position and bounding box are taken into account in
/// the physics engine, so this gives us a way to specify the mass of static objects without
/// having to add an infinite mass on every tile in the map.
#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct Mass(pub f64);

/// The keyboard controlled player. Only one entity should hold this at a given time.
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct KeyboardControlled;


/// The entity with this component and a Position component will be centered in the camera
/// when the scene is rendered.
/// Only one entity should hold this at a given time.
#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct CameraFocus;

/// Renders a sprite from a surface (spritesheet image).
///
/// The convention is that the sprite begins pointing to the right and flipping it horizontally
/// results in it facing left
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Sprite {
    /// The spritesheet to pull the image from
    pub texture_id: TextureId,
    /// The region of the spritesheet to use, unrelated to the actual bounding box
    pub region: Rect,
    /// Whether to flip the sprite along the horizontal axis
    pub flip_horizontal: bool,
}

#[derive(Debug, Default, Component)]
#[storage(HashMapStorage)]
pub struct MovementAnimation {
    pub steps: Vec<(TextureId, Rect)>,
    pub frames_per_step: usize,
    pub frame_counter: usize,
}
