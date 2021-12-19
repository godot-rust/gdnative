use std::any::TypeId;
use std::mem::{align_of, size_of};

use indexmap::IndexSet;

use crate::export::NativeClass;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Tag {
    type_id: TypeId,
}

impl Tag {
    #[inline]
    fn of<T>() -> Self
    where
        T: NativeClass,
    {
        Tag {
            type_id: TypeId::of::<T>(),
        }
    }
}

/// Whether the type tag can be transmuted to `usize`. `true` if the layouts are compatible
/// on the platform, and `type-tag-fallback` is not enabled.
const USE_TRANSMUTE: bool = cfg!(not(feature = "type-tag-fallback"))
    && size_of::<Tag>() == size_of::<usize>()
    && align_of::<Tag>() == align_of::<usize>();

/// Keep track of allocated type tags so they can be freed on cleanup. This should only be
/// accessed from one thread at a time.
static mut TAGS: Option<IndexSet<Tag, ahash::RandomState>> = None;

/// Top bit of `usize`. Used to prevent producing null type tags which might have special
/// meaning assigned.
const MAGIC: usize = 1usize.rotate_right(1);
/// Rest of `usize`.
const MAGIC_MASK: usize = !MAGIC;

/// Create a new type tag for type `T`. This should only be called from `InitHandle`.
#[inline]
pub(crate) unsafe fn create<T>() -> *const libc::c_void
where
    T: NativeClass,
{
    let tag = Tag::of::<T>();

    if USE_TRANSMUTE {
        // Safety: USE_TRANSMUTE is only true if layouts match
        *(&tag as *const Tag as *const *const libc::c_void)
    } else {
        // Safety: InitHandle is not Send or Sync, so this will only be called from one thread
        let tags = TAGS.get_or_insert_with(IndexSet::default);
        let (idx, _) = tags.insert_full(tag);
        // So we don't produce nulls. We're just assuming that 2^31 types will be
        // enough for everyone here.
        (idx | MAGIC) as *const libc::c_void
    }
}

/// Returns `true` if `tag` corresponds to type `T`. `tag` must be one returned by `create`.
#[inline]
pub(crate) unsafe fn check<T>(tag: *const libc::c_void) -> bool
where
    T: NativeClass,
{
    if USE_TRANSMUTE {
        // Safety: USE_TRANSMUTE is only true if layouts match
        Tag::of::<T>() == *(&tag as *const *const libc::c_void as *const Tag)
    } else {
        let tags = TAGS.as_ref().expect("tag should be created by `create`");
        let idx = tag as usize;
        let tag = tags
            .get_index(idx & MAGIC_MASK)
            .expect("tag should be created by `create`");
        Tag::of::<T>() == *tag
    }
}

/// Perform any cleanup actions if required. Should only be called from
/// `crate::cleanup_internal_state`. `create` and `check` shouldn't be called after this.
#[inline]
pub(crate) unsafe fn cleanup() {
    // Safety: By the time cleanup is called, create shouldn't be called again
    if let Some(tags) = TAGS.take() {
        drop(tags);
    }
}
