use crate::NativeClass;
use std::any::TypeId;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Tag {
    type_id: TypeId,
}

impl Tag {
    fn of<T>() -> Self
    where
        T: NativeClass,
    {
        Tag {
            type_id: TypeId::of::<T>(),
        }
    }
}

#[cfg(target_pointer_width = "32")]
pub(crate) use self::boxed_type_tag::*;

#[cfg(target_pointer_width = "64")]
pub(crate) use self::transmuted_type_tag::*;

/// Type tags implemented as boxed pointers. This is required for 32-bit targets because `size_t`
/// is only 32-bits wide there, while `TypeId` is always 64-bit.
#[cfg(target_pointer_width = "32")]
mod boxed_type_tag {
    use super::Tag;
    use crate::NativeClass;
    use std::boxed::Box;

    /// Keep track of allocated type tags so they can be freed on cleanup
    static mut TAGS: Option<Vec<*const Tag>> = None;

    /// Create a new type tag for type `T`. This should only be called from `InitHandle`.
    pub(crate) unsafe fn create<T>() -> *const libc::c_void
    where
        T: NativeClass,
    {
        // Safety: InitHandle is not Send or Sync, so this will only be called from one thread
        let tags = TAGS.get_or_insert_with(Vec::new);
        let type_tag = Box::into_raw(Box::new(Tag::of::<T>()));
        tags.push(type_tag);
        type_tag as *const libc::c_void
    }

    /// Returns `true` if `tag` corresponds to type `T`. `tag` must be one returned by `create`.
    pub(crate) unsafe fn check<T>(tag: *const libc::c_void) -> bool
    where
        T: NativeClass,
    {
        Tag::of::<T>() == *(tag as *const Tag)
    }

    /// Perform any cleanup actions if required. Should only be called from
    /// `crate::cleanup_internal_state`. `create` and `check` shouldn't be called after this.
    pub(crate) unsafe fn cleanup() {
        // Safety: By the time cleanup is called, create shouldn't be called again
        if let Some(tags) = TAGS.take() {
            for ptr in tags.into_iter() {
                std::mem::drop(Box::from_raw(ptr as *mut Tag))
            }
        }
    }
}

/// Type tags implemented as transmutes. This is faster on 64-bit targets, and require no
/// allocation, as `TypeId` is `Copy`, and fits in a `size_t` there. This may break in the
/// (probably very unlikely) event that:
///
/// - `TypeId`'s size changes (possible in Rust 1.x as `TypeId` is opaque).
/// - `TypeId` loses `Copy` (only possible in Rust 2.0+).
///
/// Both will be compile errors: `transmute` should fail if the sizes mismatch, and the wrapper
/// type `Tag` derives `Copy`.
#[cfg(target_pointer_width = "64")]
mod transmuted_type_tag {
    use super::Tag;
    use crate::NativeClass;

    /// Create a new type tag for type `T`. This should only be called from `InitHandle`.
    pub(crate) unsafe fn create<T>() -> *const libc::c_void
    where
        T: NativeClass,
    {
        std::mem::transmute::<Tag, *const libc::c_void>(Tag::of::<T>())
    }

    /// Returns `true` if `tag` corresponds to type `T`. `tag` must be one returned by `create`.
    pub(crate) unsafe fn check<T>(tag: *const libc::c_void) -> bool
    where
        T: NativeClass,
    {
        Tag::of::<T>() == std::mem::transmute::<*const libc::c_void, Tag>(tag)
    }

    /// Perform any cleanup actions if required. Should only be called from
    /// `crate::cleanup_internal_state`. `create` and `check` shouldn't be called after this.
    pub(crate) unsafe fn cleanup() {
        // do nothing
    }
}
