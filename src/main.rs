#[macro_use]
extern crate failure;

#[macro_use]
extern crate specs_derive;
#[macro_use]
extern crate shred_derive;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate sdl2;
extern crate specs;
extern crate shred;
extern crate cgmath;

mod systems;
mod components;
mod renderer;
mod resources;
mod texture_manager;
mod level_file;
mod map;

use cgmath::Vector2;
type Vec2D = Vector2<f64>;

use std::{
    thread,
    time::Duration,
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    rect::{Rect, Point},
};
use specs::{
    Builder,
    DispatcherBuilder,
    World,
};

use components::{
    Position,
    BoundingBox,
    Velocity,
    AppliedForce,
    TerminalVelocity,
    Mass,
    Sprite,
    KeyboardControlled,
    CameraFocus,
    MovementAnimation,
};
use resources::{FramesElapsed, GameKeys};
use texture_manager::TextureManager;
use renderer::Renderer;
use map::{LevelMap, Tile};

fn main() -> Result<(), String> {
    let mut renderer = Renderer::init(320, 240)?;
    let texture_creator = renderer.texture_creator();
    let mut textures = TextureManager::new(&texture_creator);
    let mut event_pump = renderer.event_pump()?;

    let mut world = World::new();

    world.add_resource(FramesElapsed(1));
    world.add_resource(GameKeys::from(event_pump.keyboard_state()));
    //FIXME: Remove this unwrap() when we start using proper error types
    let level_map = LevelMap::load_file("maps/level1.json", &mut textures).unwrap();
    world.add_resource(level_map.clone());

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Keyboard, "Keyboard", &[])
        .with(systems::Physics, "Physics", &["Keyboard"])
        .with(systems::PositionUpdater, "PositionUpdater", &["Physics"])
        .with(systems::Animator, "Animator", &["PositionUpdater"])
        .build();
    dispatcher.setup(&mut world.res);
    // Renderer is not called in the dispatcher, so we need to separately set up the component
    // storages for anything it uses.
    Renderer::setup(&mut world.res);

    // Add the robot
    let robot_center = level_map.level_start();
    let robot_texture = textures.create_png_texture("assets/robots.png")?;
    let robot_animation = [
        // The position on the texture of the robot
        Rect::new(110, 115, 32, 30),
        Rect::new(110, 145, 32, 30),
    ];
    world.create_entity()
        .with(KeyboardControlled)
        .with(CameraFocus)
        .with(Position(robot_center))
        .with(Mass(1000.0))
        .with(BoundingBox {width: 32, height: 30})
        .with(Velocity(Vec2D {x: 0.0, y: 0.0}))
        .with(AppliedForce(Vec2D {x: 0.0, y: 0.0}))
        .with(TerminalVelocity {x: 5.0, y: 7.0})
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

    for &Tile {x, y, image_width, image_height, ..} in level_map.iter_map_tiles() {
        world.create_entity()
            .with(Position(Point::new(x + image_width as i32 / 2, y + image_height as i32 / 2)))
            .with(BoundingBox { width: image_width, height: image_height })
            .build();
    }

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
