use crate::core::util::{hsva_to_rgba, Point};
use crate::render::Renderer;
use crate::{DoublePendulumCollection, DoublePendulumConfiguration};
use image::{ImageBuffer, Rgba};
use imageproc::drawing;
use imageproc::drawing::{Blend, Canvas};
use imageproc::point::Point as ImageprocPoint;
use itertools::Itertools;
use std::f32;
use std::path::{Path, PathBuf};

pub struct ImageRenderer {
    width: u32,
    height: u32,
    count: usize,
    base_path: PathBuf,
}

impl ImageRenderer {
    pub fn new(width: u32, height: u32, base_path: PathBuf) -> Self {
        ImageRenderer {
            width,
            height,
            count: 0,
            base_path,
        }
    }
}

impl Renderer for ImageRenderer {
    fn render_frame(&mut self, pendulums: &DoublePendulumCollection) -> Result<(), String> {
        let configurations_len_f64 = pendulums.pendulum_configurations().len() as f64;
        let pendulum_a = pendulums.pendulum_a();
        let pendulum_b = pendulums.pendulum_b();

        let mut buffer = Blend(ImageBuffer::from_pixel(
            self.width,
            self.height,
            Rgba([0, 0, 0, 255]),
        ));

        let (x_max, y_max) = (self.width, self.height);
        let (rel_x_max, rel_y_max) = (x_max as f32 / 2.0, y_max as f32 / 2.0);
        let minimum_rel_max = f32::min(rel_x_max, rel_y_max);
        let midpoint = (rel_x_max, rel_y_max);
        let midpoint_i32 = (rel_x_max as i32, rel_y_max as i32);
        let midpoint_point = ImageprocPoint::new(midpoint_i32.0, midpoint_i32.1);
        let max_extension = pendulum_a.length() + pendulum_b.length();
        let conversion_constant = (minimum_rel_max as f64 / max_extension) * 0.95;

        let convert_point = |point: Point| {
            (
                (conversion_constant * point.x) as f32 + midpoint.0,
                (conversion_constant * -point.y) as f32 + midpoint.1,
            )
        };

        let blue = Rgba([0, 0, 255, 255]);

        struct PendulumRenderInfo<'a> {
            pendulum: &'a DoublePendulumConfiguration,
            a: (f32, f32),
            b: (f32, f32),
            a_point: ImageprocPoint<i32>,
            b_point: ImageprocPoint<i32>,
            h: f64,
        }

        let render_infos: Vec<_> = pendulums
            .pendulum_configurations()
            .iter()
            .enumerate()
            .map(|(i, pendulum)| {
                let (a_position, b_position) = pendulum.positions(pendulum_a, pendulum_b);
                let (new_a, new_b) = (convert_point(a_position), convert_point(b_position));
                let (new_a_point, new_b_point) = (
                    ImageprocPoint::new(new_a.0 as i32, new_a.1 as i32),
                    ImageprocPoint::new(new_b.0 as i32, new_b.1 as i32),
                );

                let curr_h = (360 * i) as f64 / configurations_len_f64;

                PendulumRenderInfo {
                    pendulum,
                    a: new_a,
                    b: new_b,
                    a_point: new_a_point,
                    b_point: new_b_point,
                    h: curr_h,
                }
            })
            .collect();

        for (info_1, info_2) in render_infos.iter().tuple_windows() {
            let (r, g, b, a) = hsva_to_rgba(info_1.h, 1.0, 1.0, 0.01);
            let color = Rgba([r, g, b, a]);

            let new_a = info_1.a;
            let new_b = info_1.b;

            //drawing::draw_line_segment_mut(&mut buffer, midpoint, new_a, color);
            //drawing::draw_line_segment_mut(&mut buffer, new_a, new_b, color);

            let color_weight =
                1.0 - DoublePendulumConfiguration::distance(info_1.pendulum, info_2.pendulum);
            let (r, g, b, a) = hsva_to_rgba(info_1.h, 1.0, 1.0, 0.05 * color_weight);
            let color_weighed = Rgba([r, g, b, a]);

            // TODO: line between blue pixels (also depends on distance)
            drawing::draw_polygon_mut(
                &mut buffer,
                [
                    midpoint_point,
                    info_1.a_point,
                    info_1.b_point,
                    info_2.b_point,
                    info_2.a_point,
                ]
                .as_ref(),
                color_weighed,
            );
            //buffer.draw_pixel(new_a.0 as u32, new_a.1 as u32, blue);
            //buffer.draw_pixel(new_b.0 as u32, new_b.1 as u32, blue);
        }

        /*for (i, pendulum) in pendulums.iter().enumerate() {
            let (a_position, b_position) = pendulum.positions();
            let (new_a, new_b) = (convert_point(a_position), convert_point(b_position));
            let (new_a_point, new_b_point) = (
                ImageprocPoint::new(new_a.0 as i32, new_a.1 as i32),
                ImageprocPoint::new(new_b.0 as i32, new_b.1 as i32),
            );

            let curr_h = (360 * i) as f64 / pendulums.len() as f64;

            let (r, g, b, a) = hsva_to_rgba(curr_h, 1.0, 1.0, 0.01);
            let color = Rgba([r, g, b, a]);

            drawing::draw_line_segment_mut(&mut buffer, midpoint, new_a, color);
            /*drawing::draw_polygon_mut(
                &mut buffer,
                [new_a_point, midpoint_point, new_b_point].as_ref(),
                color,
            );*/
            drawing::draw_line_segment_mut(&mut buffer, new_a, new_b, color);
            buffer.draw_pixel(new_a.0 as u32, new_a.1 as u32, blue);
            buffer.draw_pixel(new_b.0 as u32, new_b.1 as u32, blue);
        }*/

        buffer.draw_pixel(midpoint_i32.0 as u32, midpoint_i32.1 as u32, blue);

        let count = self.count;
        let base_path = self.base_path.clone();

        rayon::spawn_fifo(move || {
            buffer
                .0
                .save(base_path.join(Path::new(&format!("render_{:05}.png", count))))
                .map_err(|e| e.to_string())
                .unwrap_or_else(|e| {
                    eprintln!("panic: {}", e);
                    std::process::exit(-1);
                });
        });

        self.count += 1;

        Ok(())
    }
}
