use std::f32::consts::TAU;
use std::ops::Rem;
use std::ops::{Range, RangeInclusive};

const CMP_EPSILON: f32 = 0.00001;

/// Coordinate system conversion: polar -> cartesian
///
/// Polar coordinates: distance `r` from the origin + angle `th` (radians).
/// Cartesian coordinate system: `x` and `y` axis.
///
/// Example:
/// ```
/// use gdnative::globalscope::*;
///
/// let (x, y) = polar2cartesian(13.0, -0.394791119699);
///
/// assert_eq!(x, 12.0);
/// assert_eq!(y, -5.0);
/// ```
#[inline]
pub fn polar2cartesian(r: f32, th: f32) -> (f32, f32) {
    let x = r * th.cos();
    let y = r * th.sin();

    (x, y)
}

/// Coordinate system conversion: cartesian -> polar
///
/// Cartesian coordinate system: `x` and `y` axis.
/// Polar coordinates: distance `r` from the origin + angle `th` (radians).
///
/// Example:
/// ```
/// use gdnative::globalscope::*;
///
/// let (r, th) = cartesian2polar(12.0, -5.0);
///
/// assert!(is_equal_approx(r, 13.0));
/// assert!(is_equal_approx(th, -0.394791119699));
/// ```
#[inline]
pub fn cartesian2polar(x: f32, y: f32) -> (f32, f32) {
    let r = x.hypot(y);
    let th = y.atan2(x);

    (r, th)
}

/// Converts from decibels to linear energy (audio).
#[inline]
pub fn db2linear(decibels: f32) -> f32 {
    f32::exp(decibels * 0.115_129_255)
}

/// Converts from linear energy to decibels (audio).
///
/// This can be used to implement volume sliders that behave as expected (since volume isn't linear).
#[inline]
pub fn linear2db(linear_energy: f32) -> f32 {
    linear_energy.ln() * 0.115_129_255
}

/// Position of the first non-zero digit, after the decimal point.
///
/// Note that the maximum return value is `10`, which is a design decision in the implementation.
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(step_decimals(5.0), 0);
/// assert_eq!(step_decimals(12.0004), 4);
/// assert_eq!(step_decimals(0.000000004), 9);
/// ```
#[inline]
pub fn step_decimals(step: f32) -> i32 {
    const MAXN: usize = 10;
    const SD: [f32; MAXN] = [
        0.9999, // somehow compensate for floating point error
        0.09999,
        0.009999,
        0.0009999,
        0.00009999,
        0.000009999,
        0.0000009999,
        0.00000009999,
        0.000000009999,
        0.0000000009999,
    ];

    let abs = step.abs();
    let int_abs: i32 = step as i32;
    let decs: f32 = abs - (int_abs as f32); // strip away integer part;
    for (i, item) in SD.iter().enumerate().take(MAXN) {
        if decs >= *item {
            return i.try_into().unwrap();
        }
    }
    0
}

/// Moves `range.start()` toward `range.end()` by the `delta` value.
///
/// Use a negative `delta` value `range.end()` to move away.
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(move_toward(10.0..=5.0, 4.), 6.);
/// assert_eq!(move_toward(10.0..=5.0, -1.5), 11.5);
/// assert_eq!(move_toward(4.0..=8.0, 1.0), 5.0);
/// assert_eq!(move_toward(4.0..=8.0, 5.0), 8.0);
/// assert_eq!(move_toward(8.0..=4.0, 1.0), 7.0);
/// assert_eq!(move_toward(8.0..=4.0, 5.0), 4.0);
/// ```
#[inline]
pub fn move_toward(range: RangeInclusive<f32>, delta: f32) -> f32 {
    if (range.end() - range.start()).abs() <= delta {
        *range.end()
    } else {
        range.start() + (range.end() - range.start()).signum() * delta
    }
}

/// Returns an "eased" value of x based on an easing function defined with `curve`.
///
/// This easing function is based on an `exponent`. The curve can be any floating-point number,
/// with specific values leading to the following behaviors:
///
/// Value range | Effect
/// :---: | ---
/// `s < -1` | Ease in-out
/// `s == -1` | Linear
/// `-1 < s < 0` | Ease out-in
/// `s == 0` | Constant
/// `0 < s < 1` | Ease out
/// `s == 1` | Linear
/// `s > 1` | Ease in
///
/// See also [`smoothstep`]. If you need to perform more advanced transitions, use `Tween` or `AnimationPlayer`.
///
/// Curve values cheatsheet:  
/// ![Image](https://raw.githubusercontent.com/godotengine/godot-docs/3.4/img/ease_cheatsheet.png)
#[inline]
pub fn ease(s: f32, curve: f32) -> f32 {
    let s = s.clamp(0.0, 1.0);
    if curve > 0.0 {
        if curve < 1.0 {
            1.0 - (1.0 - s).powf(1.0 / curve)
        } else {
            s.powf(curve)
        }
    } else if curve < 0.0 {
        //inout ease

        if s < 0.5 {
            (s * 2.0).powf(-curve) * 0.5
        } else {
            (1.0 - (1.0 - (s - 0.5) * 2.0).powf(-curve)) * 0.5 + 0.5
        }
    } else {
        0.0 // no ease (raw)
    }
}

/// Linearly interpolates between two values, by the factor defined in weight.
///
/// To perform interpolation, weight should be between 0.0 and 1.0 (inclusive).
/// However, values outside this range are allowed and can be used to perform extrapolation.
/// ```
/// use gdnative::globalscope::*;
/// assert_eq!(lerp(0.0..=4.0, 0.75), 3.0);
/// ```
#[inline]
pub fn lerp(range: RangeInclusive<f32>, weight: f32) -> f32 {
    range.start() + (range.end() - range.start()) * weight
}

/// Linearly interpolates between two angles (in radians), by a normalized value.
///
/// Similar to lerp, but interpolates correctly when the angles wrap around `TAU`.
/// To perform eased interpolation with `lerp_angle`, combine it with `ease` or `smoothstep`.
/// ```
/// use std::f32::consts::{PI, TAU};
/// use gdnative::globalscope::lerp_angle;
///
/// assert_eq!(lerp_angle(-PI..PI, 0.0), -PI);
/// assert_eq!(lerp_angle(-PI..PI, 1.0), -PI);
/// assert_eq!(lerp_angle(PI..-PI, 0.0), PI);
/// assert_eq!(lerp_angle(PI..-PI, 1.0), PI);
/// assert_eq!(lerp_angle(0.0..TAU, 0.0), 0.0);
/// assert_eq!(lerp_angle(0.0..TAU, 1.0), 0.0);
/// assert_eq!(lerp_angle(TAU..0.0, 0.0), TAU);
/// assert_eq!(lerp_angle(TAU..0.0, 1.0), TAU);
/// ```
#[inline]
pub fn lerp_angle(range: Range<f32>, amount: f32) -> f32 {
    let difference = f32::rem(range.end - range.start, TAU);
    let distance = f32::rem(2.0 * difference, TAU) - difference;

    range.start + distance * amount
}

/// Returns the floating-point modulus of `a/b` that wraps equally in positive and negative.
///
/// The result, if not zero, has the same sign as `b`.
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(fposmod(7.0, 3.0), 1.0);
/// assert_eq!(fposmod(-7.0, 3.0), 2.0);
/// assert_eq!(fposmod(7.0, -3.0), -2.0);
/// assert_eq!(fposmod(-7.0, -3.0), -1.0);
///
/// assert_eq!(fposmod(6.0, 3.0), 0.0);
/// assert_eq!(fposmod(-6.0, 3.0), 0.0);
/// assert_eq!(fposmod(6.0, -3.0), 0.0);
/// assert_eq!(fposmod(-6.0, -3.0), 0.0);
/// ```
#[inline]
pub fn fposmod(a: f32, b: f32) -> f32 {
    let mut value = a % b;
    if value < 0.0 && b > 0.0 || value > 0.0 && b < 0.0 {
        value += b;
    }
    value
}

/// Find linear interpolation weight from interpolated values.
///
/// Returns an interpolation or extrapolation factor considering the range specified in `range.start()` and `range.end()`,
/// and the interpolated value specified in `weight`.
///
/// The returned value will be between `0.0` and `1.0` if `weight` is between `range.start()` and `range.end()` (inclusive).
///
/// If `weight` is located outside this range, then an extrapolation factor will be returned
/// (return value lower than `0.0` or greater than `1.0`).
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(inverse_lerp(20.0..=30.0, 27.5), 0.75);
/// ```
#[inline]
pub fn inverse_lerp(range: RangeInclusive<f32>, value: f32) -> f32 {
    (value - range.start()) / (range.end() - range.start())
}

/// Smooth (Hermite) interpolation.
///
/// Returns the result of smoothly interpolating the value of `s` between `0` and `1`, based on where `s` lies
/// with respect to the edges `from` and `to`.
///
/// The return value is `0` if `s <= from`, and `1` if `s >= to`.  
///
/// If `s` lies between `from` and `to`, the returned value follows an S-shaped curve that maps `s` between `0` and `1`.  
/// This S-shaped curve is the cubic Hermite interpolator, given by `f(y) = 3*y^2 - 2*y^3` where `y = (x-from) / (to-from)`.
///
/// Compared to [`ease()`] with a curve value of `-1.6521`, `smoothstep()` returns the smoothest possible curve with no
/// sudden changes in the derivative.
///
/// If you need to perform more advanced transitions, use `Tween` or `AnimationPlayer`.
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(smoothstep(0.0, 2.0, -5.0), 0.0);
/// assert_eq!(smoothstep(0.0, 2.0, 0.5), 0.15625);
/// assert_eq!(smoothstep(0.0, 2.0, 1.0), 0.5);
/// assert_eq!(smoothstep(0.0, 2.0, 2.0), 1.0);
/// ```
#[inline]
pub fn smoothstep(from: f32, to: f32, s: f32) -> f32 {
    if is_equal_approx(from, to) {
        return from;
    }
    let s = ((s - from) / (to - from)).clamp(0.0, 1.0);
    s * s * (3.0 - 2.0 * s)
}

/// Returns `true` if `a` and `b` are approximately equal to each other.
///
/// Here, approximately equal means that `a` and `b` are within a small internal epsilon of each other,
/// which scales with the magnitude of the numbers.
///
/// Infinity values of the same sign are considered equal.
#[inline]
pub fn is_equal_approx(a: f32, b: f32) -> bool {
    if a == b {
        return true;
    }
    let mut tolerance = CMP_EPSILON * a.abs();
    if tolerance < CMP_EPSILON {
        tolerance = CMP_EPSILON;
    }
    (a - b).abs() < tolerance
}

/// Returns true if `s` is zero or almost zero.
///
/// This method is faster than using is_equal_approx with one value as zero.
#[inline]
pub fn is_zero_approx(s: f32) -> bool {
    s.abs() < CMP_EPSILON
}

/// Returns the nearest equal or larger power of 2 for an integer value.
///
/// In other words, returns the smallest value a where `a = pow(2, n)` such that `value <= a` for some non-negative integer `n`.
///
/// This behaves like [`u32::next_power_of_two()`] for `value >= 1`.
///
/// **Warning:** This function returns 0 rather than 1 for non-positive values of `value`
/// (in reality, 1 is the smallest integer power of 2).
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(nearest_po2(3), 4);
/// assert_eq!(nearest_po2(4), 4);
/// assert_eq!(nearest_po2(5), 8);
/// assert_eq!(nearest_po2(0), 0);
/// assert_eq!(nearest_po2(-1), 0);
/// ```
#[inline]
pub fn nearest_po2(value: i32) -> u32 {
    if value <= 0 {
        return 0;
    }
    (value as u32).next_power_of_two()
}

/// Returns the integer modulus of `a/b` that wraps equally in positive and negative.
///
/// The result, if not zero, has the same sign as `b`.
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(posmod(7, 3), 1);
/// assert_eq!(posmod(-7, 3), 2);
/// assert_eq!(posmod(7, -3), -2);
/// assert_eq!(posmod(-7, -3), -1);
///
/// assert_eq!(posmod(6, 3), 0);
/// assert_eq!(posmod(-6, 3), 0);
/// assert_eq!(posmod(6, -3), 0);
/// assert_eq!(posmod(-6, -3), 0);
/// ```
#[inline]
pub fn posmod(a: i32, b: i32) -> i32 {
    let mut value = a % b;
    if value < 0 && b > 0 || value > 0 && b < 0 {
        value += b;
    }
    value
}

/// Maps a value from `range_from` to `range_to`, using linear interpolation.
///
/// # Example:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(range_lerp(75.0, 0.0..=100.0, -1.0..=1.0), 0.5);
/// ```
#[inline]
pub fn range_lerp(
    value: f32,
    range_from: RangeInclusive<f32>,
    range_to: RangeInclusive<f32>,
) -> f32 {
    lerp(range_to, inverse_lerp(range_from, value))
}

/// Snaps float value `s` to a given `step`.
///
/// This can also be used to round a floating point number to an arbitrary number of decimals.
/// ```
/// use gdnative::globalscope::*;
/// use std::f32::consts::E; // Euler constant, 2.71828
///
/// assert_eq!(stepify(100.0, 32.0), 96.0);
/// assert_eq!(stepify(E, 0.01), 2.72);
/// ```
#[inline]
pub fn stepify(mut value: f32, step: f32) -> f32 {
    if step != 0.0 {
        value = (value / step + 0.5).floor() * step;
    }
    value
}

/// Wraps float value between `min` and `max`.
///
/// Usable for creating loop-alike behavior or infinite surfaces.
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
/// use std::f32::consts::{TAU, PI};
///
/// // Custom range
/// assert_eq!(wrapf(3.2, 0.5..2.5), 1.2);
///
/// // Full circle
/// let angle = 3.0 * PI;
/// assert!(is_equal_approx(wrapf(angle, 0.0..TAU), PI));
/// ```
///
/// If the range start is 0, this is equivalent to [`fposmod()`], so prefer using that instead.
///
/// Note that unlike GDScript's method, the range must be non-empty and non-inverted.
///
/// # Panics
/// If the range is empty, i.e. `range.start` >= `range.end`.
#[inline]
pub fn wrapf(value: f32, range: Range<f32>) -> f32 {
    assert!(
        !range.is_empty(),
        "wrapf expects non-empty, non-inverted range; passed {}..{}",
        range.start,
        range.end
    );

    let range_diff = range.end - range.start;
    value - range_diff * ((value - range.start) / range_diff).floor()
}

/// Wraps integer value between `min` and `max`.
///
/// Usable for creating loop-alike behavior or infinite surfaces.
///
/// # Examples:
/// ```
/// use gdnative::globalscope::*;
///
/// assert_eq!(wrapi(5, 3..5), 3);
/// assert_eq!(wrapi(1, -1..2), 1);
/// assert_eq!(wrapi(-1, 2..4), 3);
/// ```
///
/// If the range start is 0, this is equivalent to [`posmod()`], so prefer using that instead.
///
/// Note that unlike GDScript's method, the range must be non-empty and non-inverted.
///
/// # Panics
/// If the range is empty, i.e. `range.start` >= `range.end`.
#[inline]
pub fn wrapi(value: i32, range: Range<i32>) -> i32 {
    assert!(
        !range.is_empty(),
        "wrapf expects non-empty, non-inverted range; passed {}..{}",
        range.start,
        range.end
    );

    let range_diff = range.end - range.start;
    range.start + (value - range.start % range_diff + range_diff) % range_diff
}
