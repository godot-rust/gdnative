use sys;
use get_api;
use libc::c_void;
use sys::godot_rid;
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

    pub fn new_with_resource(&mut self, from: *mut c_void) {
        unsafe { (get_api().godot_rid_new_with_resource)(&mut self.0, from) }
    }

    pub fn operator_less(&mut self, b: *mut godot_rid) -> bool {
        unsafe { (get_api().godot_rid_operator_less)(&mut self.0, b) }
    }

    pub fn is_valid(&self) -> bool {
        self.to_u64() != 0
    }

    fn to_u64(&self) -> u64 {
        unsafe { transmute(self.0) }
    }
}

impl_basic_traits!{
    for Rid as godot_rid {
        Eq => godot_rid_operator_equal;
        Default => godot_rid_new;
    }
}
