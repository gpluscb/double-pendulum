use std::f64::consts::PI;
use std::ops::Add;

pub const GRAVITY: f64 = 100.0;
pub const TWO_PI: f64 = 2.0 * PI;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// https://rosettacode.org/wiki/Color_wheel#Rust
pub fn hsva_to_rgba(h: f64, s: f64, v: f64, a: f64) -> (u8, u8, u8, u8) {
    let hp = h / 60.0;
    let c = s * v;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let m = v - c;
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    if hp <= 1.0 {
        r = c;
        g = x;
    } else if hp <= 2.0 {
        r = x;
        g = c;
    } else if hp <= 3.0 {
        g = c;
        b = x;
    } else if hp <= 4.0 {
        g = x;
        b = c;
    } else if hp <= 5.0 {
        r = x;
        b = c;
    } else {
        r = c;
        b = x;
    }
    r += m;
    g += m;
    b += m;
    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    )
}

pub fn normalize_angle(angle: f64) -> f64 {
    let mut result = angle % TWO_PI;
    if result > PI {
        result -= TWO_PI;
    }
    if result < -PI {
        result += TWO_PI;
    }

    result
}

pub fn normalize_angle_mut(angle: &mut f64) {
    *angle = normalize_angle(*angle);
}

#[test]
fn test() {
    macro_rules! assert_ca_eq_f64 {
        ($a:expr, $b:expr) => {{
            let a = $a;
            let b = $b;
            assert!(
                (a - b).abs() < f64::EPSILON,
                "Assertion failed: `{}` ({}) is not circa equal to `{}` ({})",
                stringify!($a),
                a,
                stringify!($b),
                b,
            )
        }};
    }

    assert_ca_eq_f64!(normalize_angle(TWO_PI), 0.0);
    assert_ca_eq_f64!(normalize_angle(2.0 * TWO_PI), 0.0);
    assert_ca_eq_f64!(normalize_angle(0.0), 0.0);
    assert_ca_eq_f64!(normalize_angle(PI), PI);
    assert_ca_eq_f64!(normalize_angle(PI + TWO_PI), PI);
    assert_ca_eq_f64!(normalize_angle(1.5 * PI), -0.5 * PI);
    assert_ca_eq_f64!(normalize_angle(1.5 * PI + TWO_PI), -0.5 * PI);
    assert_ca_eq_f64!(normalize_angle(1.5 * PI - TWO_PI), -0.5 * PI);
    assert_ca_eq_f64!(normalize_angle(-TWO_PI), 0.0);
    assert_ca_eq_f64!(normalize_angle(2.0 * -TWO_PI), 0.0);
    assert_ca_eq_f64!(normalize_angle(-PI), -PI);
    assert_ca_eq_f64!(normalize_angle(-PI + TWO_PI), PI);
    assert_ca_eq_f64!(normalize_angle(-PI - TWO_PI), -PI);
    assert_ca_eq_f64!(normalize_angle(-1.5 * PI), 0.5 * PI);
    assert_ca_eq_f64!(normalize_angle(-1.5 * PI + TWO_PI), 0.5 * PI);
    assert_ca_eq_f64!(normalize_angle(-1.5 * PI - TWO_PI), 0.5 * PI);
}
