#[cfg(not(feature = "std"))]
pub fn sqrt(x: f32) -> f32 {
    if x < 0.0 { return f32::NAN; }
    if x == 0.0 { return 0.0; }
    let mut z = x;
    for _ in 0..10 {
        z = 0.5 * (z + x / z);
    }
    z
}

#[cfg(not(feature = "std"))]
pub fn sin(x: f32) -> f32 {
    let mut x = x;
    // Reduce to -PI..PI (very rough)
    let pi = 3.1415926535;
    let two_pi = 2.0 * pi;
    while x > pi { x -= two_pi; }
    while x < -pi { x += two_pi; }
    
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0
}

#[cfg(not(feature = "std"))]
pub fn cos(x: f32) -> f32 {
    let mut x = x;
    let pi = 3.1415926535;
    let two_pi = 2.0 * pi;
    while x > pi { x -= two_pi; }
    while x < -pi { x += two_pi; }

    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    1.0 - x2 / 2.0 + x4 / 24.0 - x6 / 720.0
}

#[cfg(not(feature = "std"))]
pub fn round(x: f32) -> f32 {
    (x + 0.5).floor()
}

#[cfg(not(feature = "std"))]
pub fn floor(x: f32) -> f32 {
    let i = x as i32;
    if x < 0.0 && x != i as f32 {
        (i - 1) as f32
    } else {
        i as f32
    }
}

#[cfg(not(feature = "std"))]
pub fn ceil(x: f32) -> f32 {
    let i = x as i32;
    if x > 0.0 && x != i as f32 {
        (i + 1) as f32
    } else {
        i as f32
    }
}

#[cfg(not(feature = "std"))]
pub fn powi(base: f32, exp: i32) -> f32 {
    let mut res = 1.0;
    let mut b = base;
    let mut e = exp;
    if e < 0 {
        b = 1.0 / b;
        e = -e;
    }
    while e > 0 {
        if e % 2 == 1 {
            res *= b;
        }
        b *= b;
        e /= 2;
    }
    res
}

#[cfg(not(feature = "std"))]
pub fn atan2(y: f32, x: f32) -> f32 {
    // Very rough approximation
    if x == 0.0 {
        if y > 0.0 { return 1.5707963; }
        if y < 0.0 { return -1.5707963; }
        return 0.0;
    }
    let z = y / x;
    let at = z / (1.0 + 0.28 * z * z);
    if x < 0.0 && y >= 0.0 { return at + 3.14159265; }
    if x < 0.0 && y < 0.0 { return at - 3.14159265; }
    at
}

#[cfg(not(feature = "std"))]
pub fn acos(x: f32) -> f32 {
    // Approximation: acos(x) = pi/2 - asin(x) ~= pi/2 - x
    1.5707963 - x 
}
