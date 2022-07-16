use crate::private::get_api;
use crate::sys;
use std::mem::transmute;

// Note: for safety design, consult:
// * https://github.com/godotengine/godot/blob/3.x/core/rid.h
// * https://github.com/godotengine/godot/blob/3.x/modules/gdnative/include/gdnative/rid.h

/// A RID ("resource ID") is an opaque handle that refers to a Godot `Resource`.
///
/// RIDs do not grant access to the resource itself. Instead, they can be used in lower-level resource APIs
/// such as the [servers]. See also [Godot API docs for `RID`][docs].
///
/// Note that RIDs are highly unsafe to work with (especially with a Release build of Godot):
/// * They are untyped, i.e. Godot does not recognize if they represent the correct resource type.
/// * The internal handle is interpreted as a raw pointer by Godot, meaning that passing an invalid or wrongly
///   typed RID is instant undefined behavior.
///
/// For this reason, GDNative methods accepting `Rid` parameters are marked `unsafe`.
///
/// [servers]: https://docs.godotengine.org/en/stable/tutorials/optimization/using_servers.html
/// [docs]: https://docs.godotengine.org/en/stable/classes/class_rid.html
#[derive(Copy, Clone, Debug)]
pub struct Rid(pub(crate) sys::godot_rid);

impl Rid {
    /// Creates an empty, invalid RID.
    #[inline]
    pub fn new() -> Self {
        Rid::default()
    }

    /// Returns the ID of the referenced resource.
    ///
    /// # Panics
    /// When this instance is empty, i.e. `self.is_occupied()` is false.
    ///
    /// # Safety
    /// RIDs are untyped and interpreted as raw pointers by the engine.
    /// If this method is called on an invalid resource ID, the behavior is undefined.
    /// This can happen when the resource behind the RID is no longer alive.
    #[inline]
    pub unsafe fn get_id(self) -> i32 {
        assert!(self.is_occupied());
        (get_api().godot_rid_get_id)(&self.0)
    }

    /// Check if this RID is non-empty. This does **not** mean it's valid or safe to use!
    ///
    /// This simply checks if the handle has not been initialized with the empty default.
    /// It does not give any indication about whether it points to a valid resource.
    #[inline]
    pub fn is_occupied(self) -> bool {
        self.to_u64() != 0
    }

    #[inline]
    fn to_u64(self) -> u64 {
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
    // Passing `self` by value will create a temporary copy, changing the value of the resulting
    // pointer. See https://github.com/godot-rust/godot-rust/issues/562.
    #[allow(clippy::trivially_copy_pass_by_ref)]
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
        Default => godot_rid_new;
        Eq => godot_rid_operator_equal;
        Ord => godot_rid_operator_less;
    }
}
