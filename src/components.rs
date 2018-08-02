use sdl2::rect::{Rect, Point};
use specs::{VecStorage, NullStorage, HashMapStorage};

use texture_manager::TextureId;

/// Represents the XY world coordinates of an entity.
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

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Velocity(pub Point);

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
