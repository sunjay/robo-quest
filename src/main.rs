extern crate sdl2;
extern crate specs;

#[macro_use]
extern crate specs_derive;

use std::{
    env,
    thread,
    time::Duration,
    path::Path,
};

use sdl2::{
    Sdl,
    TimerSubsystem,
    EventPump,
    image::{Sdl2ImageContext, LoadTexture, INIT_PNG},
    event::Event,
    keyboard::{Keycode, Scancode, KeyboardState},
    rect::{Rect, Point},
    pixels::Color,
    render::{TextureCreator, Texture, Canvas},
    video::{Window, WindowContext},
};
use specs::{
    Builder,
    Join,
    DispatcherBuilder,
    ReadExpect,
    ReadStorage,
    System,
    VecStorage,
    NullStorage,
    HashMapStorage,
    World,
    WriteStorage,
};

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Position(Point);

#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Velocity(Point);

#[derive(Debug, Default, Component)]
#[storage(NullStorage)]
struct KeyboardControlled;

/// Renders a sprite from a surface (spritesheet image).
///
/// The convention is that the sprite begins pointing to the right and flipping it horizontally
/// results in it facing left
#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Sprite {
    /// The spritesheet to pull the image from
    texture_id: TextureId,
    /// The region of the spritesheet to use, unrelated to the actual bounding box
    region: Rect,
    /// Whether to flip the sprite along the horizontal axis
    flip_horizontal: bool,
}

#[derive(Debug, Default, Component)]
#[storage(HashMapStorage)]
struct MovementAnimation {
    steps: Vec<(TextureId, Rect)>,
    frames_per_step: usize,
    frame_counter: usize,
}

/// Resource that represents the number of frames elapsed since the last time all of the systems
/// were run. Value is guaranteed to be greater than or equal to 1.
/// Often this will be just 1 but it may be greater if there is lag or if a system takes too long.
struct FramesElapsed(usize);

/// Resource that represents which keys are currently pressed.
///
/// Each boolean is true if the key is pressed and false otherwise
#[derive(Debug, Clone)]
struct GameKeys {
    up_arrow: bool,
    down_arrow: bool,
    left_arrow: bool,
    right_arrow: bool,
    menu: bool,
    select: bool,
    start: bool,
    volume_down: bool,
    volume_up: bool,
    x: bool,
    y: bool,
    a: bool,
    b: bool,
    light_key_1: bool,
    light_key_2: bool,
    light_key_3: bool,
    light_key_4: bool,
    light_key_5: bool,
}

impl<'a> From<KeyboardState<'a>> for GameKeys {
    fn from(keys: KeyboardState) -> Self {
        // From mapping: https://github.com/clockworkpi/Keypad#keymaps
        Self {
            up_arrow: keys.is_scancode_pressed(Scancode::Up),
            down_arrow: keys.is_scancode_pressed(Scancode::Down),
            left_arrow: keys.is_scancode_pressed(Scancode::Left),
            right_arrow: keys.is_scancode_pressed(Scancode::Right),
            menu: keys.is_scancode_pressed(Scancode::Escape),
            select: keys.is_scancode_pressed(Scancode::Space),
            start: keys.is_scancode_pressed(Scancode::Return),
            volume_down: keys.is_scancode_pressed(Scancode::Minus),
            //FIXME: This probably isn't the right key
            volume_up: keys.is_scancode_pressed(Scancode::KpPlus),
            x: keys.is_scancode_pressed(Scancode::U),
            y: keys.is_scancode_pressed(Scancode::I),
            a: keys.is_scancode_pressed(Scancode::J),
            b: keys.is_scancode_pressed(Scancode::K),
            light_key_1: keys.is_scancode_pressed(Scancode::Home),
            light_key_2: keys.is_scancode_pressed(Scancode::PageUp),
            light_key_3: false, //FIXME: No way to check if Shift key pressed
            light_key_4: keys.is_scancode_pressed(Scancode::PageDown),
            light_key_5: keys.is_scancode_pressed(Scancode::End),
        }
    }
}

struct Keyboard;

impl<'a> System<'a> for Keyboard {
    type SystemData = (ReadExpect<'a, GameKeys>, WriteStorage<'a, Velocity>, ReadStorage<'a, KeyboardControlled>);

    fn run(&mut self, (keys, mut velocities, keyboard_controlled): Self::SystemData) {
        for (Velocity(ref mut vel), _) in (&mut velocities, &keyboard_controlled).join() {
            let y = vel.y();
            // Assuming that only a single arrow key can be held down at a time.
            if keys.right_arrow {
                *vel = Point::new(3, y);
            }
            else if keys.left_arrow {
                *vel = Point::new(-3, y);
            }
            else {
                *vel = Point::new(0, y);
            }
        }
    }
}

struct PositionUpdater;

impl<'a> System<'a> for PositionUpdater {
    type SystemData = (ReadExpect<'a, FramesElapsed>, ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, (frames, velocities, mut positions): Self::SystemData) {
        let FramesElapsed(frames_elapsed) = *frames;

        for (Velocity(vel), Position(pos)) in (&velocities, &mut positions).join() {
            *pos = pos.offset(vel.x() * frames_elapsed as i32, vel.y() * frames_elapsed as i32);
        }
    }
}

struct Animator;

impl<'a> System<'a> for Animator {
    type SystemData = (ReadExpect<'a, FramesElapsed>, ReadStorage<'a, Velocity>, WriteStorage<'a, Sprite>, WriteStorage<'a, MovementAnimation>);

    fn run(&mut self, (frames, velocities, mut positions, mut animations): Self::SystemData) {
        let FramesElapsed(frames_elapsed) = *frames;

        for (&Velocity(vel), sprite, animation) in (&velocities, &mut positions, &mut animations).join() {
            if vel.x() > 0 {
                // The assumption is that the sprite begins facing right
                sprite.flip_horizontal = false;
            }
            else if vel.x() < 0 {
                sprite.flip_horizontal = true;
            }
            else { // No horizontal movement
                // Only continue to animate if moving
                continue;
            }

            animation.frame_counter += frames_elapsed;
            let current_step = animation.frame_counter % (animation.steps.len() * animation.frames_per_step) / animation.frames_per_step;

            let (current_texture_id, current_region) = animation.steps[current_step];
            sprite.texture_id = current_texture_id;
            sprite.region = current_region;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TextureId(usize);

struct TextureManager<'a> {
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

struct Renderer {
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

fn main() -> Result<(), String> {
    let mut renderer = Renderer::init(320, 240)?;
    let texture_creator = renderer.texture_creator();
    let mut textures = TextureManager::new(&texture_creator);
    let mut event_pump = renderer.event_pump()?;

    let mut world = World::new();
    //FIXME: Replace with setup: https://slide-rs.github.io/specs/07_setup.html
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<KeyboardControlled>();
    world.register::<Sprite>();
    world.register::<MovementAnimation>();

    world.add_resource(FramesElapsed(1));
    world.add_resource(GameKeys::from(event_pump.keyboard_state()));

    // Add the robot
    let robot_texture = textures.create_png_texture("assets/robots.png")?;
    let canvas_size = renderer.dimensions();
    let robot_center = Point::new(canvas_size.0 as i32 / 2, canvas_size.1 as i32 / 2);
    let robot_animation = [
        // The position on the texture of the robot
        Rect::new(110, 115, 32, 30),
        Rect::new(110, 145, 32, 30),
    ];
    world.create_entity()
        .with(KeyboardControlled)
        .with(Position(robot_center))
        .with(Velocity(Point::new(0, 0)))
        .with(Sprite {
            texture_id: robot_texture,
            region: robot_animation[0],
            flip_horizontal: false,
        })
        .with(MovementAnimation {
            steps: robot_animation.into_iter().map(|&rect| (robot_texture, rect)).collect(),
            frames_per_step: 5,
            frame_counter: 0,
        })
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(Keyboard, "Keyboard", &[])
        .with(PositionUpdater, "PositionUpdater", &["Keyboard"])
        .with(Animator, "Animator", &["Keyboard"])
        .build();

    let mut timer = renderer.timer()?;

    let fps = 60;

    // Frames elapsed since the last render
    let mut last_frames_elapsed = 0;
    let mut running = true;
    while running {
        let ticks = timer.ticks(); // ms

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                _ => {},
            }
        }

        let frames_elapsed = (ticks as f64 / 1000.0 * fps as f64) as usize;
        let frames_elapsed_delta = frames_elapsed - last_frames_elapsed;

        // At least one frame must have passed for us to do anything
        if frames_elapsed_delta >= 1 {
            *world.write_resource::<FramesElapsed>() = FramesElapsed(frames_elapsed_delta);
            *world.write_resource::<GameKeys>() = GameKeys::from(event_pump.keyboard_state());

            dispatcher.dispatch(&mut world.res);

            renderer.render(&world, &textures)?;
            last_frames_elapsed = frames_elapsed;
        }
        else {
            let ms_per_frame = (1000.0 / fps as f64) as u64;
            let ms_elapsed = (timer.ticks() - ticks) as u64;
            thread::sleep(Duration::from_millis(ms_per_frame - ms_elapsed));
        }
    }

    Ok(())
}
