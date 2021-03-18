use super::IsEqualApprox;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn is_equal_approx(self, to: &Self) -> bool {
        self.x.is_equal_approx(to.x)
            && self.y.is_equal_approx(to.y)
            && self.z.is_equal_approx(to.z)
            && self.w.is_equal_approx(to.w)
    }

    #[inline]
    fn glam(self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}
