use crate::core::util::{normalize_angle, normalize_angle_mut, Point, GRAVITY};
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::time::Duration;
pub mod util;

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pendulum {
    length: f64,
    mass: f64,
}

impl Pendulum {
    pub fn new(length: f64, mass: f64) -> Self {
        Pendulum { length, mass }
    }

    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn mass(&self) -> f64 {
        self.mass
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PendulumConfiguration {
    /// Radians
    angle: f64,
    /// Radians per \[time\]
    angular_velocity: f64,
}

impl PendulumConfiguration {
    pub fn new(angle: f64, angular_velocity: f64) -> Self {
        PendulumConfiguration {
            angle,
            angular_velocity,
        }
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn angular_velocity(&self) -> f64 {
        self.angular_velocity
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DoublePendulumConfiguration {
    a: PendulumConfiguration,
    b: PendulumConfiguration,
}

impl DoublePendulumConfiguration {
    pub fn new(a: PendulumConfiguration, b: PendulumConfiguration) -> Self {
        DoublePendulumConfiguration { a, b }
    }

    pub fn a_configuration(&self) -> PendulumConfiguration {
        self.a
    }

    pub fn b_configuration(&self) -> PendulumConfiguration {
        self.b
    }

    pub fn random_configuration() -> Self {
        let mut rng = rand::thread_rng();
        let angle_a = rng.gen_range(-PI..PI);
        let angle_b = rng.gen_range(-PI..PI);
        let ang_vel_a = rng.gen_range(-PI..PI);
        let ang_vel_b = rng.gen_range(-PI..PI);

        DoublePendulumConfiguration {
            a: PendulumConfiguration {
                angle: angle_a,
                angular_velocity: ang_vel_a,
            },
            b: PendulumConfiguration {
                angle: angle_b,
                angular_velocity: ang_vel_b,
            },
        }
    }

    pub fn a_position(&self, pendulum_a: &Pendulum) -> Point {
        let angle = self.a.angle;
        let length = pendulum_a.length;

        Point {
            x: length * angle.sin(),
            y: -length * angle.cos(),
        }
    }

    pub fn positions(&self, pendulum_a: &Pendulum, pendulum_b: &Pendulum) -> (Point, Point) {
        let a_position = self.a_position(pendulum_a);

        let b_angle = self.b.angle;
        let b_length = pendulum_b.length;

        let b_offset = Point {
            x: b_length * b_angle.sin(),
            y: -b_length * b_angle.cos(),
        };

        (a_position, a_position + b_offset)
    }

    /// 0 is exactly identical, 1 is theoretical maximum distance
    pub fn distance(&self, other: &DoublePendulumConfiguration) -> f64 {
        // Get normalized angle distances (between 0 and 1)
        let angle_distance_a = f64::abs(normalize_angle(self.a.angle - other.a.angle));
        let angle_distance_b = f64::abs(normalize_angle(self.b.angle - other.b.angle));

        // Between 0 and 1
        let norm_angle_distance_a = angle_distance_a / PI;
        let norm_angle_distance_b = angle_distance_b / PI;

        assert!(
            (0.0..=1.0).contains(&norm_angle_distance_a),
            "{}",
            norm_angle_distance_a
        );
        /*if !(0.0..=1.0).contains(&norm_angle_distance_a) {
            eprintln!("assertion failure, not panicking today: norm_angle_distance_a: {}", norm_angle_distance_a);
        }*/
        assert!(
            (0.0..=1.0).contains(&norm_angle_distance_b),
            "{}",
            norm_angle_distance_b
        );
        /*if !(0.0..=1.0).contains(&norm_angle_distance_b) {
            eprintln!("assertion failure, not panicking today: norm_angle_distance_b: {}", norm_angle_distance_b);
        }*/

        norm_angle_distance_a * norm_angle_distance_b
    }

    pub fn angular_accelerations(
        &self,
        pendulum_a: &Pendulum,
        pendulum_b: &Pendulum,
    ) -> (f64, f64) {
        let mass_a = pendulum_a.mass;
        let mass_b = pendulum_b.mass;
        let angle_a = self.a.angle;
        let angle_b = self.b.angle;
        let ang_vel_a = self.a.angular_velocity;
        let ang_vel_b = self.b.angular_velocity;
        let len_a = pendulum_a.length;
        let len_b = pendulum_b.length;

        let double_mass_a = 2.0 * mass_a;
        let angle_diff = angle_a - angle_b;
        let angle_diff_cos = angle_diff.cos();
        let angle_diff_sin = angle_diff.sin();
        let double_angle_diff_sin = 2.0 * angle_diff_sin;
        let double_angle_diff = 2.0 * angle_diff;
        let doubled_angles_diff_cos = double_angle_diff.cos();
        let ang_vel_a_sq = ang_vel_a * ang_vel_a;
        let ang_vel_b_sq = ang_vel_b * ang_vel_b;

        let mass_sum = mass_a + mass_b;

        // Spanish wikipedia has the equations lol https://es.wikipedia.org/wiki/Doble_p%C3%A9ndulo#Ecuaciones_de_movimiento
        let ang_acc_a = (-GRAVITY * (double_mass_a + mass_b) * angle_a.sin()
            - mass_b * GRAVITY * f64::sin(angle_a - 2.0 * angle_b)
            - double_angle_diff_sin
                * mass_b
                * (ang_vel_b_sq * len_b + ang_vel_a_sq * len_a * angle_diff_cos))
            / (len_a * (double_mass_a + mass_b - mass_b * doubled_angles_diff_cos));

        let ang_acc_b = double_angle_diff_sin
            * (ang_vel_a_sq * len_a * mass_sum
                + GRAVITY * mass_sum * angle_a.cos()
                + ang_vel_b_sq * len_b * mass_b * angle_diff_cos)
            / (len_b * (2.0 * mass_a + mass_b - mass_b * doubled_angles_diff_cos));

        (ang_acc_a, ang_acc_b)
    }

    pub fn step(&mut self, pendulum_a: &Pendulum, pendulum_b: &Pendulum, duration: Duration) {
        let (ang_acc_a, ang_acc_b) = self.angular_accelerations(pendulum_a, pendulum_b);
        let secs = duration.as_secs_f64();

        self.a.angular_velocity += ang_acc_a * secs;
        self.b.angular_velocity += ang_acc_b * secs;
        self.a.angle += self.a.angular_velocity * secs;
        self.b.angle += self.b.angular_velocity * secs;

        normalize_angle_mut(&mut self.a.angle);
        normalize_angle_mut(&mut self.b.angle);
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DoublePendulumCollection {
    pendulum_a: Pendulum,
    pendulum_b: Pendulum,
    pendulum_configurations: Vec<DoublePendulumConfiguration>,
}

impl DoublePendulumCollection {
    pub fn new(
        pendulum_a: Pendulum,
        pendulum_b: Pendulum,
        pendulum_configurations: Vec<DoublePendulumConfiguration>,
    ) -> Self {
        DoublePendulumCollection {
            pendulum_a,
            pendulum_b,
            pendulum_configurations,
        }
    }

    pub fn pendulum_a(&self) -> &Pendulum {
        &self.pendulum_a
    }

    pub fn pendulum_b(&self) -> &Pendulum {
        &self.pendulum_b
    }

    pub fn pendulum_configurations(&self) -> &Vec<DoublePendulumConfiguration> {
        &self.pendulum_configurations
    }

    pub fn step_all(&mut self, step_time: Duration) {
        let pendulum_a = &self.pendulum_a;
        let pendulum_b = &self.pendulum_b;

        self.pendulum_configurations
            .par_iter_mut()
            .for_each(|pendulum| pendulum.step(pendulum_a, pendulum_b, step_time));
    }

    pub fn step_all_n_times(&mut self, step_time: Duration, n: u32) {
        let pendulum_a = &self.pendulum_a;
        let pendulum_b = &self.pendulum_b;

        self.pendulum_configurations
            .par_iter_mut()
            .for_each(|pendulum| {
                (0..n).for_each(|_| pendulum.step(pendulum_a, pendulum_b, step_time))
            });
    }
}
