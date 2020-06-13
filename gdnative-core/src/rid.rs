use crate::private::get_api;
use crate::sys;
use std::cmp::Ordering;
use std::cmp::{Eq, PartialEq};
use std::mem::transmute;

/// The RID type is used to access the unique integer ID of a resource.
/// They are opaque, so they do not grant access to the associated resource by themselves.
#[derive(Copy, Clone, Debug)]
pub struct Rid(pub(crate) sys::godot_rid);

impl Rid {
    #[inline]
    pub fn new() -> Self {
        Rid::default()
    }

    #[inline]
    pub fn get_id(&self) -> i32 {
        unsafe { (get_api().godot_rid_get_id)(&self.0) }
    }

    #[inline]
    pub fn operator_less(&self, b: &Rid) -> bool {
        unsafe { (get_api().godot_rid_operator_less)(&self.0, &b.0) }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.to_u64() != 0
    }

    #[inline]
    fn to_u64(&self) -> u64 {
        unsafe {
            // std::mem::transmute needs source and destination types to have the same size. On 32
            // bit systems sizeof(void *) != size_of<u64>() so this fails to compile. The bindings
            // define godot_rid as (a newtype of) [u8; u8size] or [u8; u4size] depending on
            // architecture word size so transmuting to usize should always work.
            transmute::<_, usize>(self.0) as _
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_rid {
        &self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut sys::godot_rid {
        &mut self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_rid) -> Self {
        Rid(sys)
    }
}

impl_basic_traits_as_sys! {
    for Rid as godot_rid {
        Eq => godot_rid_operator_equal;
        Default => godot_rid_new;
    }
}

impl PartialOrd for Rid {
    #[inline]
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
