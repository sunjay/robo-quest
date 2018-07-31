use std::env;

use sdl2::{
    self,
    Sdl,
    TimerSubsystem,
    EventPump,
    image::{Sdl2ImageContext, INIT_PNG},
    pixels::Color,
    render::{TextureCreator, Canvas},
    video::{Window, WindowContext},
};
use specs::{
    Join,
    ReadStorage,
    World,
};

use texture_manager::TextureManager;
use components::{Position, Sprite};

pub struct Renderer {
    sdl_context: Sdl,
    /// Required to use images, but not used for anything after it is created
    _image_context: Sdl2ImageContext,
    canvas: Canvas<Window>,
}

impl Renderer {
    pub fn init(width: u32, height: u32) -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let _image_context = sdl2::image::init(INIT_PNG).unwrap();

        // Scale display if a certain environment variable is set
        let display_scale = env::var("DISPLAY_SCALE")
            .map(|x| x.parse().expect("DISPLAY_SCALE must be a number"))
            .unwrap_or(1.0);

        //FIXME: Remove this unwrap() when we start using proper error types
        let window_width = (width as f32 * display_scale) as u32;
        let window_height = (height as f32 * display_scale) as u32;
        let window = video_subsystem.window("Robo Quest", window_width, window_height)
            .position_centered()
            .build()
            .unwrap();

        //FIXME: Remove this unwrap() when we start using proper error types
        let mut canvas = window.into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();

        // The background color
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

        //FIXME: Remove this unwrap() when we start using proper error types
        canvas.set_logical_size(width, height).unwrap();

        Ok(Self {
            sdl_context,
            _image_context,
            canvas,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.canvas.logical_size()
    }

    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn timer(&self) -> Result<TimerSubsystem, String> {
        self.sdl_context.timer()
    }

    pub fn event_pump(&self) -> Result<EventPump, String> {
        self.sdl_context.event_pump()
    }

    pub fn render(&mut self, world: &World, textures: &TextureManager) -> Result<(), String> {
        self.canvas.clear();

        let (positions, sprites): (ReadStorage<Position>, ReadStorage<Sprite>) = world.system_data();
        //FIXME: The ordering of rendering needs to be made explicit with layering or something
        for (Position(pos), sprite) in (&positions, &sprites).join() {
            let texture = textures.get(sprite.texture_id);
            let source_rect = sprite.region;
            let mut dest_rect = source_rect.clone();
            dest_rect.center_on(*pos);

            self.canvas.copy_ex(
                texture,
                Some(source_rect),
                Some(dest_rect),
                0.0,
                None,
                sprite.flip_horizontal,
                false
            )?;
        }

        self.canvas.present();

        Ok(())
    }
}
