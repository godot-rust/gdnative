use std::f32::consts::TAU;

use crate::core_types::Vector2;
const CMP_EPSILON: f32 = 0.00001;

/// Converts a 2D point expressed in the `cartesian` coordinate system (X and Y axis)
///  to the `polar` coordinate system (a distance from the origin and an angle `(in radians)`).
#[inline]
pub fn cartasian2polar(x: f32, y: f32) -> Vector2 {
    Vector2::new((x * x + y * y).sqrt(), y.atan2(x))
}
/// Converts from `decibels` to linear energy (audio).
#[inline]
pub fn db2linear(db: f32) -> f32 {
    f32::exp(db * 0.115_129_255)
}
/// Returns the `position` of the first `non-zero` digit, after the decimal point.
/// Note that the `maximum` return `value` is `10`, which is a design decision in the implementation.
/// # Examples:
/// ```rust
/// assert_eq!(step_decimals(5.0), 0);
/// assert_eq!(step_decimals(12.0004), 4);
/// assert_eq!(step_decimals(0.000000004), 9);
/// ```
#[inline]
pub fn step_decimals(p_step: f32) -> i32 {
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

    let abs = p_step.abs();
    let int_abs: i32 = p_step as i32;
    let decs: f32 = abs - (int_abs as f32); // strip away integer part;
    for (i, item) in SD.iter().enumerate().take(MAXN) {
        if decs >= *item {
            return i.try_into().unwrap();
        }
    }
    0
}
/// Moves `from` toward `to` by the `delta` `value`.
/// Use a negative `delta` value `to` move away.
/// # Examples:
/// ```rust
/// assert_eq!(move_toward(10., 5., 4.), 6.);
/// assert_eq!(move_toward(10., 5., -1.5), 11.5);
/// ```
#[inline]
pub fn move_toward(p_from: f32, p_to: f32, p_delta: f32) -> f32 {
    if (p_to - p_from).abs() <= p_delta {
        p_to
    } else {
        p_from + (p_to - p_from).signum() * p_delta
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
/// Returns the floating-point remainder of `a/b`, keeping the sign of `a`.
/// # Examples:
/// ```rust
/// assert_eq!(fmod(7.0, 5.5), 1.5);
/// ```
#[inline]
pub fn fmod(numer: f32, denom: f32) -> f32 {
    numer - (f32::trunc(numer / denom) * denom)
}
/// Linearly interpolates between two values by the factor defined in weight.
/// To perform interpolation, weight should be between 0.0 and 1.0 (inclusive).
/// However, values outside this range are allowed and can be used to perform extrapolation.
/// ```rust
/// assert_eq!(lerp(0.0, 4.0, 0.75), 3.0);
/// ```
#[inline]
pub fn lerp(from: f32, to: f32, weight: f32) -> f32 {
    from + (to - from) * weight
}
/// Linearly interpolates between two angles (in radians) by a normalized value.
/// Similar to lerp, but interpolates correctly when the angles wrap around `TAU`.
/// To perform eased interpolation with `lerp_angle`, combine it with `ease` or `smoothstep`.
#[inline]
pub fn lerp_angle(angle_from: f32, angle_to: f32, amount: f32) -> f32 {
    let difference = fmod(angle_to - angle_from, TAU);
    let distance = fmod(2.0 * difference, TAU) - difference;
    angle_from + distance * amount
}
/// Returns the floating-point modulus of `a/b` that wraps equally in `positive` and `negative`.
/// # Examples:
/// ```rust
/// assert_eq!(fposmod(-1.5, 1.5), 0.0);
/// assert_eq!(fposmod(-1.0, 1.5), 0.5);
/// assert_eq!(fposmod(-0.5, 1.5), 1.0);
/// assert_eq!(fposmod(0.0, 1.5), 0.0);
/// ```
#[inline]
pub fn fposmod(p_x: f32, p_y: f32) -> f32 {
    let mut value = fmod(p_x, p_y);
    if ((value < 0.0) && (p_y > 0.0)) || ((value > 0.0) && (p_y < 0.0)) {
        value += p_y;
    }

    value += 0.0;
    value
}

/// Returns an interpolation or extrapolation factor considering the range specified in `from` and `to`,
/// and the interpolated value specified in `weight`.
/// The returned value will be between `0.0` and `1.0` if `weight` is between `from` and `to` (inclusive).
/// If `weight` is located outside this range,
/// then an extrapolation factor will be returned (return value lower than `0.0` or greater than `1.0`).
/// # Examples:
/// ```rust
/// assert_eq!(inverse_lerp(20.0, 30.0, 27.5), 0.75);
/// ```
#[inline]
pub fn inverse_lerp(p_from: f32, p_to: f32, p_value: f32) -> f32 {
    (p_value - p_from) / (p_to - p_from)
}
/// Returns the result of smoothly interpolating the value of `s` between `0` and `1`, based on the where `s` lies with respect to the edges `from` and `to`.
/// The return value is `0` if `s <= from`, and `1` if `s >= to`. If `s` lies between `from` and `to`, the returned value follows an S-shaped curve that maps `s` between `0` and `1`.
/// This S-shaped curve is the cubic Hermite interpolator, given by `f(y) = 3*y^2 - 2*y^3` where `y = (x-from) / (to-from)`.
/// Compared to ease with a curve value of `-1.6521`, smoothstep returns the smoothest possible curve with no sudden changes in the derivative.
/// If you need to perform more advanced transitions, use Tween or AnimationPlayer.
/// # Examples:
/// ```rust
/// assert_eq!(smoothstep(0.0, 2.0, -5.0), 0.0);
/// assert_eq!(smoothstep(0.0, 2.0, 0.5), 0.15625);
/// assert_eq!(smoothstep(0.0, 2.0, 1.0), 0.5);
/// assert_eq!(smoothstep(0.0, 2.0, 2.0), 1.0);
/// ```
#[inline]
pub fn smoothstep(p_from: f32, p_to: f32, p_s: f32) -> f32 {
    if is_equal_approx(p_from, p_to) {
        return p_from;
    }
    let s = ((p_s - p_from) / (p_to - p_from)).clamp(0.0, 1.0);
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
pub fn polar2cartesian(r: f32, th: f32) -> Vector2 {
    Vector2::new(r * th.cos(), r * th.sin())
}
/// Returns the integer modulus of a/b that wraps equally in positive and negative.
/// # Examples:
/// ```rust
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
/// Maps a value from range [istart, istop] to [ostart, ostop].
/// # Example:
/// ```rust
/// assert_eq!(range_lerp(75.0, 0.0, 100.0, -1.0, 1.0), 0.5);
/// ```
#[inline]
pub fn range_lerp(value: f32, istart: f32, istop: f32, ostart: f32, ostop: f32) -> f32 {
    lerp(ostart, ostop, inverse_lerp(istart, istop, value))
}
/// Snaps float value s to a given step.
/// This can also be used to round a floating point number to an arbitrary number of decimals.
/// ```rust
/// assert_eq!(stepify(100.0, 32.0), 96.0);
/// assert_eq!(stepify(3.14159, 0.01), 3.14);
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
/// ```rust
/// //Infinite loop between 5.0 and 9.9
/// let value = wrapf(value + 0.1, 5.0, 10.0);
/// //Infinite rotation (in radians)
/// let angle = wrapf(angle + 0.1, 0.0, TAU);
/// //Infinite rotation (in radians)
/// let angle = wrapf(angle + 0.1, -PI, PI);
/// ```
/// # Note:
/// If min is 0, this is equivalent to fposmod, so prefer using that instead.
/// wrapf is more flexible than using the fposmod approach by giving the user control over the minimum value.
#[inline]
pub fn wrapf(value: f32, min: f32, max: f32) -> f32 {
    let range: f32 = max - min;
    if is_zero_approx(range) {
        return min;
    }
    value - (range * ((value - min) / range).floor())
}
/// Wraps integer value between min and max.
/// Usable for creating loop-alike behavior or infinite surfaces.
/// ```rust
/// //Infinite loop between 5 and 9
/// let frame = wrapi(frame + 1, 5, 10);
/// //result is -2
/// let result = wrapi(-6, -5, -1);
/// ```
/// # Note:
/// If min is 0, this is equivalent to posmod, so prefer using that instead.
/// wrapi is more flexible than using the posmod approach by giving the user control over the minimum value.
#[inline]
pub fn wrapi(value: i32, min: i32, max: i32) -> i32 {
    let range = max - min;
    if range == 0 {
        return min;
    }
    min + ((((value - min) % range) + range) % range)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn step_decimals_test() {
        assert_eq!(step_decimals(5.0), 0);
        assert_eq!(step_decimals(12.0004), 4);
        assert_eq!(step_decimals(0.000000004), 9);
    }
    #[test]
    fn move_toward_test() {
        assert_eq!(move_toward(10., 5., 4.), 6.);
        assert_eq!(move_toward(10., 5., -1.5), 11.5);
    }
    #[test]
    fn fmod_test() {
        assert_eq!(fmod(7.0, 5.5), 1.5);
    }
    #[test]
    fn lerp_test() {
        assert_eq!(lerp(0.0, 4.0, 0.75), 3.0);
    }
    #[test]
    fn fposmod_test() {
        assert_eq!(fposmod(-1.5, 1.5), 0.0);
        assert_eq!(fposmod(-1.0, 1.5), 0.5);
        assert_eq!(fposmod(-0.5, 1.5), 1.0);
        assert_eq!(fposmod(0.0, 1.5), 0.0);
    }

    #[test]
    fn inverse_lerp_test() {
        assert_eq!(inverse_lerp(20.0, 30.0, 27.5), 0.75);
    }

    #[test]
    fn smoothstep_test() {
        assert_eq!(smoothstep(0.0, 2.0, -5.0), 0.0);
        assert_eq!(smoothstep(0.0, 2.0, 0.5), 0.15625);
        assert_eq!(smoothstep(0.0, 2.0, 1.0), 0.5);
        assert_eq!(smoothstep(0.0, 2.0, 2.0), 1.0);
    }

    #[test]
    fn nearest_po2_test() {
        assert_eq!(nearest_po2(3), 4);
        assert_eq!(nearest_po2(4), 4);
        assert_eq!(nearest_po2(5), 8);
        assert_eq!(nearest_po2(0), 0);
        assert_eq!(nearest_po2(-1), 0);
    }
    #[test]
    fn posmod_test() {
        const VALS: [i32; 7] = [0, 1, 2, 0, 1, 2, 0];
        for i in (-3..4).enumerate() {
            assert_eq!(posmod(i.1, 3), VALS[i.0]);
        }
    }
    #[test]
    fn range_lerp_test() {
        assert_eq!(range_lerp(75.0, 0.0, 100.0, -1.0, 1.0), 0.5);
    }
    #[test]
    fn stepify_test() {
        assert_eq!(stepify(100.0, 32.0), 96.0);
        assert_eq!(stepify(3.14159, 0.01), 3.1399999);
    }

    #[test]
    fn wrapi_test() {
        assert_eq!(wrapi(-6, -5, -1), -2);
    }
}
