use crate::sys;
use crate::get_api;
use std::cmp::Ordering;
use std::mem::transmute;
use std::cmp::{Eq, PartialEq};

/// The RID type is used to access the unique integer ID of a resource.
/// They are opaque, so they do not grant access to the associated resource by themselves.
#[derive(Copy, Clone, Debug)]
pub struct Rid(pub(crate) sys::godot_rid);

impl Rid {
    pub fn new() -> Self {
        Rid::default()
    }

    pub fn get_id(&self) -> i32 {
        unsafe { (get_api().godot_rid_get_id)(&self.0) }
    }

    pub fn operator_less(&self, b: &Rid) -> bool {
        unsafe { (get_api().godot_rid_operator_less)(&self.0, &b.0) }
    }

    pub fn is_valid(&self) -> bool {
        self.to_u64() != 0
    }

    fn to_u64(&self) -> u64 {
        unsafe { transmute::<_, usize>(self.0) as _ }
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_rid {
        &self.0
    }

    #[doc(hidden)]
    pub fn mut_sys(&mut self) -> *mut sys::godot_rid {
        &mut self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_rid) -> Self {
        Rid(sys)
    }
}

impl_basic_traits!{
    for Rid as godot_rid {
        Eq => godot_rid_operator_equal;
        Default => godot_rid_new;
    }
}

impl PartialOrd for Rid {
    fn partial_cmp(&self, other: &Rid) -> Option<Ordering> {
        unsafe {
            let native = (get_api().godot_rid_operator_less)(&self.0, &other.0);

            if native {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        }
    }
}