use sdl2::rect::{Rect, Point};
use specs::{VecStorage, NullStorage, HashMapStorage};

use texture_manager::TextureId;

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Position(pub Point);

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct Velocity(pub Point);

#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
pub struct KeyboardControlled;

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
