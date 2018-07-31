use std::path::Path;

use sdl2::{
    image::LoadTexture,
    render::{TextureCreator, Texture},
    video::WindowContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(usize);

pub struct TextureManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    // NOTE: Ideally, this would just be managed in the renderer, but we can't do that because
    // we can't have a field in a struct that refers to another field. Textures are dependent
    // on the TextureCreator and they need to be stored separately in order for this to work.
    textures: Vec<Texture<'a>>,
}

impl<'a> TextureManager<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        Self {
            texture_creator,
            textures: Default::default(),
        }
    }

    pub fn get(&self, TextureId(index): TextureId) -> &Texture<'a> {
        &self.textures[index]
    }

    pub fn create_png_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<TextureId, String> {
        let texture = self.texture_creator.load_texture(path)?;

        self.textures.push(texture);
        Ok(TextureId(self.textures.len() - 1))
    }
}
