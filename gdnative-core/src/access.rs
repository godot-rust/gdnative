//! Maybe unaligned pool array access

use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
/// An pool array access that may be unaligned.
pub struct MaybeUnaligned<G> {
    guard: G,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
/// An pool array access that is (assumed to be) aligned.
pub struct Aligned<G> {
    guard: G,
}

#[derive(Debug)]
/// An pool array write access with an owned aligned copy. The data is written back when this is
/// dropped.
pub struct Owned<G>
where
    G: Guard + WritePtr,
    G::Target: Copy,
{
    guard: G,
    owned: Vec<G::Target>,
}

/// Trait for array access guards
#[doc(hidden)]
pub unsafe trait Guard: Drop {
    type Target;
    fn len(&self) -> usize;
    fn read_ptr(&self) -> *const Self::Target;
}

/// Marker trait for write access guards
#[doc(hidden)]
pub unsafe trait WritePtr: Guard {}

impl<G: Guard> MaybeUnaligned<G> {
    pub(crate) fn new(guard: G) -> Self {
        MaybeUnaligned { guard }
    }

    /// Assumes that an access is aligned. It is undefined behavior to Deref the resulting
    /// access if the underlying pointer is not aligned to `G::Target`.
    pub unsafe fn assume_aligned(self) -> Aligned<G> {
        Aligned { guard: self.guard }
    }

    /// Tries to convert to an aligned access. Returns `None` if the underlying pointer is not
    /// aligned.
    pub fn try_into_aligned(self) -> Option<Aligned<G>> {
        if self.guard.read_ptr() as usize % mem::align_of::<G::Target>() == 0 {
            unsafe { Some(self.assume_aligned()) }
        } else {
            None
        }
    }

    /// Copies the data out of this access into a `Vec`.
    pub fn to_vec(&self) -> Vec<G::Target>
    where
        G::Target: Copy,
    {
        let len = self.guard.len();
        let mut vec = Vec::with_capacity(len);
        unsafe {
            let mut src = self.guard.read_ptr();
            for _ in 0..len {
                vec.push(ptr::read_unaligned(src));
                src = src.add(1);
            }
        }
        assert_eq!(len, vec.len());
        vec
    }

    /// Converts to an access backed by an owned, aligned copy of the data. The data is written
    /// back when the access is dropped.
    pub fn into_owned(self) -> Owned<G>
    where
        G: WritePtr,
        G::Target: Copy,
    {
        let vec = self.to_vec();
        Owned {
            guard: self.guard,
            owned: vec,
        }
    }
}

impl<G: Guard> Aligned<G> {
    pub fn as_slice(&self) -> &[G::Target] {
        unsafe {
            let ptr = self.guard.read_ptr();
            let len = self.guard.len();
            slice::from_raw_parts(ptr, len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [G::Target]
    where
        G: WritePtr,
    {
        unsafe {
            let ptr = self.guard.read_ptr() as *mut G::Target;
            let len = self.guard.len();
            slice::from_raw_parts_mut(ptr, len)
        }
    }
}

impl<G: Guard> Deref for Aligned<G> {
    type Target = [G::Target];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<G: Guard + WritePtr> DerefMut for Aligned<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<G> Owned<G>
where
    G: Guard + WritePtr,
    G::Target: Copy,
{
    pub fn as_slice(&self) -> &[G::Target] {
        debug_assert_eq!(
            self.guard.len(),
            self.owned.len(),
            "owned vec should be exactly as large as guard.len"
        );
        self.owned.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [G::Target] {
        debug_assert_eq!(
            self.guard.len(),
            self.owned.len(),
            "owned vec should be exactly as large as guard.len"
        );
        self.owned.as_mut_slice()
    }
}

impl<G> Deref for Owned<G>
where
    G: Guard + WritePtr,
    G::Target: Copy,
{
    type Target = [G::Target];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<G> DerefMut for Owned<G>
where
    G: Guard + WritePtr,
    G::Target: Copy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<G> Drop for Owned<G>
where
    G: Guard + WritePtr,
    G::Target: Copy,
{
    fn drop(&mut self) {
        unsafe {
            let mut dst = self.guard.read_ptr() as *mut G::Target;
            for o in self.owned.iter() {
                ptr::write_unaligned(dst, *o);
                dst = dst.add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PtrGuard<T> {
        ptr: *const T,
        len: usize,
    }

    impl<T> Drop for PtrGuard<T> {
        fn drop(&mut self) {}
    }

    unsafe impl<T> Guard for PtrGuard<T> {
        type Target = T;
        fn len(&self) -> usize {
            self.len
        }
        fn read_ptr(&self) -> *const T {
            self.ptr
        }
    }

    unsafe impl<T> WritePtr for PtrGuard<T> {}

    #[test]
    fn it_detects_unaligned_ptrs() {
        let vec: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let aligned = vec.as_ptr();
        let unaligned = unsafe { (aligned as *const u8).add(1) as *const i64 };

        assert_eq!(
            Some(vec![1, 2, 3, 4, 5, 6]),
            MaybeUnaligned::new(PtrGuard {
                ptr: aligned,
                len: 6,
            })
            .try_into_aligned()
            .map(|slice| Vec::from(&*slice))
        );

        assert!(MaybeUnaligned::new(PtrGuard {
            ptr: unaligned,
            len: 1,
        })
        .try_into_aligned()
        .is_none());
    }

    #[test]
    fn it_can_copy_back_owned() {
        let mut arr: [u8; 512] = [0; 512];

        let unaligned_ptr = unsafe {
            let mut ptr = &mut arr[0] as *mut u8;
            for _ in 0..(512 - 64) {
                if ptr as usize % mem::align_of::<i64>() != 0 {
                    break;
                }
                ptr = ptr.add(1);
            }
            assert!(ptr as usize % mem::align_of::<i64>() != 0);
            ptr as *mut i64
        };

        {
            let access = MaybeUnaligned::new(PtrGuard {
                ptr: unaligned_ptr,
                len: 8,
            });

            let mut write = access.into_owned();
            let slice = write.as_mut_slice();
            assert_eq!(8, slice.len());
            for i in 0..8 {
                slice[i] = (i * 2) as i64;
            }
        }

        let access = MaybeUnaligned::new(PtrGuard {
            ptr: unaligned_ptr,
            len: 8,
        });

        let vec = access.to_vec();

        assert_eq!(vec![0, 2, 4, 6, 8, 10, 12, 14], vec);
    }
}
