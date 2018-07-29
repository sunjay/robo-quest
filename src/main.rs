extern crate sdl2;
extern crate specs;

#[macro_use]
extern crate specs_derive;

use std::path::Path;

use sdl2::{
    Sdl,
    TimerSubsystem,
    EventPump,
    event::Event,
    keyboard::{Keycode, Scancode, KeyboardState},
    rect::{Rect, Point},
    pixels::Color,
    surface::Surface,
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
    current_step: usize,
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
        for (vel, _) in (&mut velocities, &keyboard_controlled).join() {
            unimplemented!();
        }
    }
}

struct PositionUpdater;

impl<'a> System<'a> for PositionUpdater {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        for (Velocity(vel), Position(pos)) in (&vel, &mut pos).join() {
            let time_delta = 0.05; //TODO
            *pos = pos.offset((vel.x() as f64 * time_delta) as i32, (vel.y() as f64 * time_delta) as i32);
        }
    }
}

struct Animator;

impl<'a> System<'a> for Animator {
    type SystemData = (ReadExpect<'a, FramesElapsed>, ReadStorage<'a, Velocity>, ReadStorage<'a, Sprite>, ReadStorage<'a, MovementAnimation>);

    fn run(&mut self, (frames, vel, pos, animations): Self::SystemData) {
        unimplemented!();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TextureId(usize);

struct Renderer<'a> {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    surfaces: Vec<Surface<'a>>,
    textures: Vec<Texture<'a>>,
}

impl<'a> Renderer<'a> {
    pub fn init() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        //FIXME: Remove this unwrap() when we start using proper error types
        let window = video_subsystem.window("Robo Quest", 320, 240)
            .position_centered()
            .build()
            .unwrap();


        //FIXME: Remove this unwrap() when we start using proper error types
        let mut canvas = window.into_canvas()
            .accelerated()
            .build()
            .unwrap();
        let texture_creator = canvas.texture_creator();

        // The background color
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

        Ok(Self {
            sdl_context,
            canvas,
            texture_creator,
            surfaces: Vec::new(),
            textures: Vec::new(),
        })
    }

    pub fn dimensions(&self) -> Result<(u32, u32), String> {
        self.canvas.output_size()
    }

    pub fn timer(&self) -> Result<TimerSubsystem, String> {
        self.sdl_context.timer()
    }

    pub fn event_pump(&self) -> Result<EventPump, String> {
        self.sdl_context.event_pump()
    }

    pub fn create_bmp_texture<P: AsRef<Path>>(&mut self, path: P) -> Result<TextureId, String> {
        let surface = Surface::load_bmp(path)?;
        self.surfaces.push(surface);
        //FIXME: Remove this unwrap() when we start using proper error types
        let texture = self.texture_creator.create_texture_from_surface(self.surfaces.last().unwrap()).unwrap();

        self.textures.push(texture);
        Ok(TextureId(self.textures.len() - 1))
    }

    pub fn render(&mut self, world: &World) -> Result<(), String> {
        self.canvas.clear();

        let (positions, sprites): (ReadStorage<Position>, ReadStorage<Sprite>) = world.system_data();
        //FIXME: The ordering of rendering needs to be made explicit with layering or something
        for (Position(pos), sprite) in (&positions, &sprites).join() {
            let TextureId(texture_index) = sprite.texture_id;
            let texture = &self.textures[texture_index];
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
    let mut renderer = Renderer::init()?;
    let event_pump = renderer.event_pump()?;

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
    let robot_texture = renderer.create_bmp_texture("assets/robot.bmp")?;
    let canvas_size = renderer.dimensions()?;
    let robot_center = Point::new(canvas_size.0 as i32 / 2, canvas_size.1 as i32 / 2);
    let robot_animation = [
        // The position on the texture of the robot
        Rect::new(110, 115, 32, 30),
        Rect::new(110, 145, 32, 30),
    ];
    world.create_entity()
        .with(KeyboardControlled)
        .with(Position(robot_center))
        .with(Sprite {
            texture_id: robot_texture,
            region: robot_animation[0],
            flip_horizontal: false,
        })
        .with(MovementAnimation {
            steps: robot_animation.into_iter().map(|&rect| (robot_texture, rect)).collect(),
            frames_per_step: 5,
            current_step: 0,
        })
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .with(Keyboard, "Keyboard", &[])
        .with(PositionUpdater, "PositionUpdater", &["Keyboard"])
        .with(Animator, "Animator", &["Keyboard"])
        .build();

    let mut timer = renderer.timer()?;

    let fps = 30;

    // Frames elapsed since the last render
    let mut last_frames_elapsed = 0;
    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                _ => {},
            }
        }

        let ticks = timer.ticks(); // ms

        let frames_elapsed = (ticks as f64 / 1000.0 * fps as f64) as usize;
        let frames_elapsed_delta = frames_elapsed - last_frames_elapsed;

        // At least one frame must have passed for us to do anything
        if frames_elapsed_delta >= 1 {
            let mut elapsed_resource = world.write_resource::<FramesElapsed>();
            *elapsed_resource = FramesElapsed(frames_elapsed_delta);
            let mut keystate_resource = world.write_resource::<GameKeys>();
            *keystate_resource = GameKeys::from(event_pump.keyboard_state());

            dispatcher.dispatch(&mut world.res);

            last_frames_elapsed = frames_elapsed;
        }
        else {
            //TODO: sleep for the remainder of the frame in order to preserve battery
        }
    }

    Ok(())
}
