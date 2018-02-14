use sys;
use get_api;
use std::mem::transmute;
use std::cmp::{PartialEq, Eq};
use std::hash::{Hash, Hasher};

/// Resource Id.
#[derive(Copy, Clone, Debug, Default)]
pub struct Rid(pub(crate) sys::godot_rid);

impl Rid {
    pub fn get_id(&self) -> i32 {
        unsafe { (get_api().godot_rid_get_id)(&self.0) }
    }

    pub fn is_valid(&self) -> bool {
        self.to_u64() != 0
    }

    fn to_u64(&self) -> u64 {
        unsafe { transmute(*self) }
    }
}

impl PartialEq for Rid {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (get_api().godot_rid_operator_equal)(&self.0, &other.0) }
    }
}

impl Eq for Rid {}

impl Hash for Rid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_u64().hash(state);
    }
}
