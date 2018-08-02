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

mod systems;
mod components;
mod renderer;
mod resources;
mod texture_manager;
mod level_file;
mod map;

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

use components::{Position, BoundingBox, Velocity, Sprite, KeyboardControlled, MovementAnimation};
use resources::{FramesElapsed, GameKeys};
use texture_manager::TextureManager;
use renderer::Renderer;
use map::LevelMap;

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
    world.add_resource(level_map);

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Keyboard, "Keyboard", &[])
        .with(systems::Physics, "Physics", &["Keyboard"])
        .with(systems::BoundaryEnforcer, "BoundaryEnforcer", &["Physics"])
        .with(systems::Animator, "Animator", &["Keyboard"])
        .build();
    dispatcher.setup(&mut world.res);

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
        .with(BoundingBox {width: 32, height: 30})
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
