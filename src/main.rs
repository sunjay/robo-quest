extern crate sdl2;

use std::{
    path::Path,
    time::{Duration, Instant},
    thread,
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    rect::{Rect, Point},
    pixels::Color,
    surface::Surface,
};

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Robo Quest", 320, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .accelerated().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));

    let mut timer = sdl_context.timer()?;
    let mut event_pump = sdl_context.event_pump()?;

    let robot_surface = Surface::load_bmp(Path::new("assets/robots.bmp"))?;
    let robot_texture = texture_creator.create_texture_from_surface(&robot_surface).unwrap();
    // The position on the texture of the robot
    let robot_animation = [
        Rect::new(110, 115, 32, 30),
        Rect::new(110, 145, 32, 30),
    ];
    let robot_animation_speed = 0.9;
    let canvas_size = canvas.output_size()?;
    let mut robot_center = Point::new(canvas_size.0 as i32 / 2, canvas_size.1 as i32 / 2);
    let mut face_left = false;
    let mut momentum = 0;

    let fps = 60;
    let mut running = true;
    while running {
        let frame_timer = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    momentum = 5;
                    face_left = false;
                },
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    momentum = -5;
                    face_left = true;
                },
                Event::KeyUp {keycode: Some(Keycode::Left), ..} |
                Event::KeyUp {keycode: Some(Keycode::Right), ..} => {
                    momentum = 0;
                }
                _ => {},
            }
        }

        let ticks = timer.ticks() as i32; // ms

        let frames_per_anim_frame = (fps as f64 * (1.0 - robot_animation_speed)) as usize;
        let frames_elapsed = (ticks as f64 / 1000.0 * fps as f64) as usize;
        robot_center = robot_center.offset(momentum, 0);

        // Current frame of the animation
        let robot_animation_frame = (frames_elapsed / frames_per_anim_frame) % robot_animation.len();

        let robot_source_rect = robot_animation[robot_animation_frame];
        let mut robot_dest_rect = robot_source_rect.clone();
        robot_dest_rect.center_on(robot_center);

        canvas.clear();
        // copy the frame to the canvas
        canvas.copy_ex(&robot_texture, Some(robot_source_rect), Some(robot_dest_rect), 0.0, None, face_left, false)?;
        canvas.present();

        let elapsed = frame_timer.elapsed();
        let frame_length = Duration::from_millis(1000 / fps);
        if frame_length > elapsed {
            thread::sleep(frame_length - elapsed);
        }
    }

    Ok(())
}
