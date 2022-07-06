//! `fmod` is `%` operator on `f32`;
//! ```rust
//! use std::ops::Rem;
//! assert_eq!(f32::rem(12.0, 10.0), 2.0);
//! ```

use std::f32::consts::TAU;
use std::ops::Rem;
use std::ops::{Range, RangeInclusive};

const CMP_EPSILON: f32 = 0.00001;

/// Converts a 2D point expressed in the `cartesian` coordinate system (X and Y axis)
///  to the `polar` coordinate system (a distance from the origin and an angle `(in radians)`).
#[inline]
pub fn cartasian2polar(x: f32, y: f32) -> (f32, f32) {
    ((x * x + y * y).sqrt(), y.atan2(x))
}

/// Converts from `decibels` to linear energy (audio).
#[inline]
pub fn db2linear(db: f32) -> f32 {
    f32::exp(db * 0.115_129_255)
}

/// Returns the `position` of the first `non-zero` digit, after the decimal point.
/// Note that the `maximum` return `value` is `10`, which is a design decision in the implementation.
/// # Examples:
/// ```
/// use gdnative_core::globalscope::step_decimals;
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

/// Moves `range.start()` toward `range.end()` by the `delta` `value`.
/// Use a negative `delta` value `range.end()` move away.
/// # Examples:
/// ```
/// use gdnative_core::globalscope::move_toward;
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
/// This easing function is based on an `exponent`. The curve can be any floating-point number,
/// with specific values leading to the following behaviors:
#[inline]
pub fn ease(mut s: f32, curve: f32) -> f32 {
    if s < 0.0 {
        s = 0.0;
    } else if s > 1.0 {
        s = 1.0;
    }
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

/// Linearly interpolates between two values by the factor defined in weight.
/// To perform interpolation, weight should be between 0.0 and 1.0 (inclusive).
/// However, values outside this range are allowed and can be used to perform extrapolation.
/// ```
/// use gdnative_core::globalscope::lerp;
/// assert_eq!(lerp(0.0..=4.0, 0.75), 3.0);
/// ```
#[inline]
pub fn lerp(range: RangeInclusive<f32>, weight: f32) -> f32 {
    range.start() + (range.end() - range.start()) * weight
}

/// Linearly interpolates between two angles (in radians) by a normalized value.
/// Similar to lerp, but interpolates correctly when the angles wrap around `TAU`.
/// To perform eased interpolation with `lerp_angle`, combine it with `ease` or `smoothstep`
/// use std::f32::consts::{PI, TAU};
/// use gdnative::globalscope::lerp_angle;
///
/// assert_eq!(lerp_angle(-PI..PI, 0.0), -PI);
/// assert_eq!(lerp_angle(-PI..PI, 1.0), -PI);
/// assert_eq!(lerp_angle(PI..-PI, 0.0), PI);
/// assert_eq!(lerp_angle(PI..-PI, 1.0), PI);
/// assert_eq!(lerp_angle(0.0..TAU, 0.0), 0.0);
/// assert_eq!(lerp_angle(0.0..TAU, 1.0), 0.0);
/// assert_eq!(lerp_angle(TAU..0, 0.0), TAU);
/// assert_eq!(lerp_angle(TAU..0, 1.0), TAU);
#[inline]
pub fn lerp_angle(range: Range<f32>, amount: f32) -> f32 {
    let difference = f32::rem(range.end - range.start, TAU);

    let distance = f32::rem(2.0 * difference, TAU) - difference;
    range.start + distance * amount
}

/// Returns the floating-point modulus of `a/b` that wraps equally in `positive` and `negative`.
/// # Examples:
/// ```rust
/// use gdnative_core::globalscope::fposmod;
/// assert_eq!(fposmod(-1.5, 1.5), 0.0);
/// assert_eq!(fposmod(-1.0, 1.5), 0.5);
/// assert_eq!(fposmod(-0.5, 1.5), 1.0);
/// assert_eq!(fposmod(0.0, 1.5), 0.0);
/// ```
#[inline]
pub fn fposmod(x: f32, y: f32) -> f32 {
    let mut value = f32::rem(x, y);
    if ((value < 0.0) && (y > 0.0)) || ((value > 0.0) && (y < 0.0)) {
        value += y;
    }

    value += 0.0;
    value
}

/// Returns an interpolation or extrapolation factor considering the range specified in `range.start()` and `range.end()`,
/// and the interpolated value specified in `weight`.
/// The returned value will be between `0.0` and `1.0` if `weight` is between `range.start()` and `range.end()` (inclusive).
/// If `weight` is located outside this range,
/// then an extrapolation factor will be returned (return value lower than `0.0` or greater than `1.0`).
/// # Examples:
/// ```rust
/// use gdnative_core::globalscope::inverse_lerp;
/// assert_eq!(inverse_lerp(20.0..=30.0, 27.5), 0.75);
/// ```
#[inline]
pub fn inverse_lerp(range: RangeInclusive<f32>, value: f32) -> f32 {
    (value - range.start()) / (range.end() - range.start())
}

/// Returns the result of smoothly interpolating the value of `s` between `0` and `1`, based on the where `s` lies with respect to the edges `from` and `to`.
/// The return value is `0` if `s <= from`, and `1` if `s >= to`. If `s` lies between `from` and `to`, the returned value follows an S-shaped curve that maps `s` between `0` and `1`.
/// This S-shaped curve is the cubic Hermite interpolator, given by `f(y) = 3*y^2 - 2*y^3` where `y = (x-from) / (to-from)`.
/// Compared to ease with a curve value of `-1.6521`, smoothstep returns the smoothest possible curve with no sudden changes in the derivative.
/// If you need to perform more advanced transitions, use Tween or AnimationPlayer.
/// # Examples:
/// ```rust
/// use gdnative_core::globalscope::smoothstep;
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
/// Here, approximately equal means that `a` and `sb` are within a small internal epsilon of each other,
/// which scales with the magnitude of the numbers.
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

/// Returns true if s is zero or almost zero.
/// This method is faster than using is_equal_approx with one value as zero.
#[inline]
pub fn is_zero_approx(s: f32) -> bool {
    s.abs() < CMP_EPSILON
}

/// Converts from linear energy to decibels (audio).
/// This can be used to implement volume sliders that behave as expected (since volume isn't linear).
#[inline]
pub fn linear2db(nrg: f32) -> f32 {
    nrg.ln() * 0.115_129_255
}

/// Returns the nearest equal or larger power of 2 for integer value.
/// In other words, returns the smallest value a where `a = pow(2, n)` such that `value <= a` for some non-negative integer `n`.
/// # Examples:
/// ```rust
/// use gdnative_core::globalscope::nearest_po2;
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

/// Converts a 2D point expressed in the polar coordinate system
/// (a distance from the origin r and an angle th (radians)) to the cartesian coordinate system (X and Y axis).
#[inline]
pub fn cartesian2polar(r: f32, th: f32) -> (f32, f32) {
    (r * th.cos(), r * th.sin())
}

/// Returns the integer modulus of a/b that wraps equally in positive and negative.
/// # Examples:
/// ```rust
/// use gdnative_core::globalscope::posmod;
/// const VALS: [i32; 7] = [0, 1, 2, 0, 1, 2, 0];
/// for i in (-3..4).enumerate() {
///     assert_eq!(posmod(i.1, 3), VALS[i.0]);
/// }
/// ```
#[inline]
pub fn posmod(a: i32, b: i32) -> i32 {
    let mut value = a % b;
    if ((value < 0) && (b > 0)) || ((value > 0) && (b < 0)) {
        value += b;
    }
    value
}

/// Maps a value from range `range.from` to `range_to`.
/// # Example:
/// ```rust
/// use gdnative_core::globalscope::range_lerp;
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

/// Snaps float value s to a given step.
/// This can also be used to round a floating point number to an arbitrary number of decimals.
/// ```rust
/// use gdnative_core::globalscope::stepify;
/// assert_eq!(stepify(100.0, 32.0), 96.0);
/// assert_eq!(stepify(3.14159, 0.01), 3.1399999);
/// ```
#[inline]
pub fn stepify(mut value: f32, step: f32) -> f32 {
    if step != 0.0 {
        value = (value / step + 0.5).floor() * step;
    }
    value
}

/// Wraps float value between min and max.
/// Usable for creating loop-alike behavior or infinite surfaces.
/// # Examples :
/// ```rust
/// use gdnative_core::globalscope::wrapf;
/// use std::f32::consts::{TAU, PI};
///
/// //Infinite loop between 5.0 and 9.9
/// let value = 1.5;
/// let angle = 0.70707;
/// let value = wrapf(value + 0.1, 5.0..10.0);
/// //Infinite rotation (in radians)
/// let angle = wrapf(angle + 0.1, 0.0..TAU);
/// //Infinite rotation (in radians)
/// let angle = wrapf(angle + 0.1, -PI..PI);
/// ```
/// # Tests :
/// ```rust
/// use gdnative_core::globalscope::wrapf;
/// use std::f32::consts::{TAU, PI};
///
/// let value = 0.5;
/// assert_eq!(wrapf(value + 0.1, 5.0..0.0), 5.6);
/// let angle = PI/4.0;
/// assert_eq!(wrapf(angle, 0.0..TAU), 0.7853982);
/// assert_eq!(wrapf(1.0, 0.5..1.5), 1.0);
/// assert_eq!(wrapf(0.75, -0.5..0.5), -0.25);
/// ```
///
/// # Note:
/// If min is 0, this is equivalent to fposmod, so prefer using that instead.
/// wrapf is more flexible than using the fposmod approach by giving the user control over the minimum value.
#[inline]
pub fn wrapf(value: f32, range: Range<f32>) -> f32 {
    let range_diff: f32 = range.end - range.start;
    if is_zero_approx(range_diff) {
        return range.start;
    }
    value - (range_diff * ((value - range.start / range_diff).floor()))
}

/// Wraps integer value between min and max.
/// Usable for creating loop-alike behavior or infinite surfaces.
/// # Example :
/// ```rust
/// use gdnative_core::globalscope::wrapi;
///
/// //Infinite loop between 5 and 9
/// let frame = 10;
/// let frame = wrapi(frame + 1, 5..10);
/// //result is -2
/// let result = wrapi(-6, -5..-1);
/// ```
/// # Tests :
/// ```rust
/// use gdnative_core::globalscope::wrapi;
///
/// assert_eq!(wrapi(1, -1..2), 1);
/// assert_eq!(wrapi(-1, 2..4), 3);
/// assert_eq!(wrapi(1, 2..-1), 1);
/// ```
/// # Note:
/// If min is 0, this is equivalent to posmod, so prefer using that instead.
/// wrapi is more flexible than using the posmod approach by giving the user control over the minimum value.
#[inline]
pub fn wrapi(value: i32, range: Range<i32>) -> i32 {
    let range_diff = range.end - range.start;
    if range_diff == 0 {
        return range.start;
    }
    range.start + (((value - range.start % range_diff) + range_diff) % range_diff)
}
