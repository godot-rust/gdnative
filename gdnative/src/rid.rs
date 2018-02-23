use sys;
use get_api;
use std::mem::transmute;
use std::cmp::{PartialEq, Eq};

/// Resource Id.
#[derive(Copy, Clone, Debug)]
pub struct Rid(pub(crate) sys::godot_rid);

impl Rid {
    pub fn new() -> Self { Rid::default() }

    pub fn get_id(&self) -> i32 {
        unsafe { (get_api().godot_rid_get_id)(&self.0) }
    }

    // TODO
    // pub fn new_with_resource(&mut self, from: Type) {
    //     unimplemented!();
    // }

    // pub fn operator_less(&mut self) -> bool {
    //     unimplemented!();
    // }

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
