mod core;
mod render;

use crate::core::{
    DoublePendulumCollection, DoublePendulumConfiguration, Pendulum, PendulumConfiguration,
};
use crate::render::image::ImageRenderer;
use crate::render::sdl2::SDL2Renderer;
use crate::render::Renderer;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::BlendMode;
use std::f64::consts::PI;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let render_in_window = true;

    let pend_a = Pendulum::new(180.0, 10.0);
    let pend_b = Pendulum::new(162.0, 1.0);

    let initial_configuration = DoublePendulumConfiguration::new(
        PendulumConfiguration::new(PI, PI / 2.0),
        PendulumConfiguration::new(PI - 3.0, PI / 4.0),
    );
    let initial_a_configuration = initial_configuration.a_configuration();
    let initial_b_configuration = initial_configuration.b_configuration();

    let pendulum_configurations: Vec<_> = (0..5_000)
        .map(|i| {
            DoublePendulumConfiguration::new(
                initial_a_configuration,
                PendulumConfiguration::new(
                    initial_b_configuration.angle() + 0.00000001 * i as f64,
                    initial_b_configuration.angular_velocity(),
                ),
            )
        })
        .collect();

    let mut pendulums = DoublePendulumCollection::new(pend_a, pend_b, pendulum_configurations);

    let target_step = Duration::from_secs_f64(0.0001);
    // Aiming for 60fps if we get realtime physics
    let target_steps_per_render = (1.0 / 60.0 / target_step.as_secs_f64()) as u32;

    if render_in_window {
        render_to_sdl2_window(target_step, target_steps_per_render, &mut pendulums)?;
    } else {
        render_to_images(target_step, target_steps_per_render, &mut pendulums)?;
    }

    let json = serde_json::to_vec_pretty(&pendulums).map_err(|e| e.to_string())?;
    std::fs::write(
        "out/last_abort.json",
        json,
    )
    .map_err(|e| e.to_string())
}

fn render_to_sdl2_window(
    target_step: Duration,
    target_steps_per_render: u32,
    pendulums: &mut DoublePendulumCollection,
) -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("double-pendulum", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("could not make a canvas");

    canvas.set_blend_mode(BlendMode::Blend);

    let renderer = SDL2Renderer::new(canvas);
    let mut event_pump = sdl_context.event_pump()?;

    let before_calc = || {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return ControlFlow::Break(()),
                _ => {}
            }
        }

        ControlFlow::Continue(())
    };

    main_loop(
        renderer,
        before_calc,
        target_step,
        target_steps_per_render,
        pendulums,
    )
}

fn render_to_images(
    target_step: Duration,
    target_steps_per_render: u32,
    pendulums: &mut DoublePendulumCollection,
) -> Result<(), String> {
    let renderer = ImageRenderer::new(
        1080,
        1080,
        PathBuf::from("out"),
    );

    static RUNNING: AtomicBool = AtomicBool::new(true);

    let before_calc = || {
        if RUNNING.load(Ordering::Relaxed) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    };

    ctrlc::set_handler(|| RUNNING.store(false, Ordering::Relaxed)).map_err(|e| e.to_string())?;

    main_loop(
        renderer,
        before_calc,
        target_step,
        target_steps_per_render,
        pendulums,
    )
}

fn main_loop(
    mut renderer: impl Renderer,
    mut before_calc: impl FnMut() -> ControlFlow<(), ()>,
    target_step: Duration,
    target_steps_per_render: u32,
    pendulums: &mut DoublePendulumCollection,
) -> Result<(), String> {
    let mut cumulative_calc_time = Duration::ZERO;

    let mut step_time;
    let mut last_step = Instant::now();

    let mut render_iterations = 0u32;

    'out: loop {
        let start_calc = Instant::now();

        if matches!(before_calc(), ControlFlow::Break(_)) {
            break 'out;
        }

        step_time = Duration::min(last_step.elapsed(), target_step); // Step at most step time!!
        last_step = Instant::now();

        pendulums.step_all_n_times(step_time, target_steps_per_render);

        renderer.render_frame(pendulums)?;

        let calc_time = start_calc.elapsed();
        cumulative_calc_time += calc_time;

        let total_iterations = render_iterations * target_steps_per_render;
        let to_sleep = (target_step * target_steps_per_render).saturating_sub(calc_time);
        println!(
            "step: {}s, slep: {}s, calc: {}s, render iteration: {}, total iteration: {}, total simulated time: {}s",
            step_time.as_secs_f64(),
            to_sleep.as_secs_f64(),
            calc_time.as_secs_f64(),
            render_iterations,
            total_iterations,
            (step_time * total_iterations).as_secs_f64(),
        );
        thread::sleep(to_sleep);

        render_iterations += 1;
    }

    println!(
        "Total/Avg calc time: {}, {}",
        cumulative_calc_time.as_secs_f64(),
        (cumulative_calc_time / render_iterations).as_secs_f64()
    );

    Ok(())
}
