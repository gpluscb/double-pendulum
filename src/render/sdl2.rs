use crate::core::util::{hsva_to_rgba, Point};
use crate::render::Renderer;
use itertools::Itertools;
use sdl2::pixels::Color;
use sdl2::rect::Point as SDL2Point;
use sdl2::render::WindowCanvas;
use crate::DoublePendulumCollection;

pub struct SDL2Renderer(WindowCanvas);

impl SDL2Renderer {
    pub fn new(canvas: WindowCanvas) -> Self {
        SDL2Renderer(canvas)
    }
}

impl Renderer for SDL2Renderer {
    fn render_frame(&mut self, pendulums: &DoublePendulumCollection) -> Result<(), String> {
        let configurations_len_f64 = pendulums.pendulum_configurations().len() as f64;
        let pendulum_a = pendulums.pendulum_a();
        let pendulum_b = pendulums.pendulum_b();

        let canvas = &mut self.0;

        let (x_max, y_max) = canvas.window().size();
        let (rel_x_max, rel_y_max) = (x_max / 2, y_max / 2);
        let minimum_rel_max = u32::min(rel_x_max, rel_y_max);
        let midpoint = SDL2Point::new(rel_x_max as i32, rel_y_max as i32);
        let max_extension = pendulums.pendulum_a().length()
            + pendulums.pendulum_b().length();
        let conversion_constant = minimum_rel_max as f64 / max_extension;

        let convert_point = |point: Point| {
            SDL2Point::new(
                (conversion_constant * point.x) as i32,
                (conversion_constant * -point.y) as i32,
            ) + midpoint
        };

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        #[derive(Copy, Clone)]
        struct PendulumRenderInfo {
            a_point: SDL2Point,
            b_point: SDL2Point,
            h: f64,
        }

        let render_infos: Vec<_> = pendulums
            .pendulum_configurations()
            .iter()
            .enumerate()
            .map(|(i, pendulum)| {
                let (a_position, b_position) = pendulum.positions(pendulum_a, pendulum_b);
                let (new_a, new_b) = (convert_point(a_position), convert_point(b_position));

                let h = (360 * i) as f64 / configurations_len_f64;

                PendulumRenderInfo {
                    a_point: new_a,
                    b_point: new_b,
                    h,
                }
            })
            .collect();

        for (info_1, info_2) in render_infos.iter().tuple_windows() {
            canvas.set_draw_color(hsva_to_rgba(info_1.h, 1.0, 1.0, 0.01));

            canvas.draw_lines([midpoint, info_1.a_point, info_1.b_point].as_ref())?;

            canvas.set_draw_color(Color::BLUE);
            canvas.draw_points([info_1.a_point, info_1.b_point].as_ref())?;
        }

        canvas.present();

        Ok(())
    }
}
