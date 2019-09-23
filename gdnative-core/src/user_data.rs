//! Customizable user-data wrappers.
//!
//! ## `NativeClass` and user-data
//!
//! In Godot Engine, scripted behavior is attached to base objects through "script instances":
//! objects that store script state, and allow dynamic dispatch of overridden methods. GDNative
//! exposes this to native languages as a `void *` pointer, known as "user-data", that may point
//! to anything defined by the native library in question.
//!
//! Godot is written in C++, and unlike Rust, it doesn't have the same strict reference aliasing
//! constraints. This user-data pointer can be aliased mutably, and called freely from different
//! threads by the engine or other scripts. Thus, to maintain safety, wrapper types are be used
//! to make sure that the Rust rules for references are always held for the `self` argument, and
//! no UB can occur because we freed `owner` or put another script on it.
//!
//! ## Which wrapper to use?
//!
//! ### Use a `MutexData<T>` when:
//!
//! - You don't want to handle locks explicitly.
//! - Your `NativeClass` type is only `Send`, but not `Sync`.
//!
//! ### Use a `RwLockData<T>` when:
//!
//! - You don't want to handle locks explicitly.
//! - Some of your exported methods take `&self`, and you don't need them to be exclusive.
//! - Your `NativeClass` type is `Send + Sync`.
//!
//! ### Use a `ArcData<T>` when:
//!
//! - You want safety for your methods, but can't tolerate lock overhead on each method call.
//! - You want fine grained lock control for parallelism.
//! - All your exported methods take `&self`.
//! - Your `NativeClass` type is `Send + Sync`.
//!
//! ### Use a `Tracked<_<T>>` when:
//!
//! - You want `FromVariant` for instances of the type, so you can take them as arguments.
//! - You might have multiple GDNative libraries in one project, and want safety against foreign
//!   `user_data`s that may point to arbitrary data or even invalid memory.
//! - You're fine with a global lock on instance construction, destruction, and downcasts.

use parking_lot::{Mutex, RwLock};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use std::time::Duration;

use crate::NativeClass;

/// Trait for customizable user-data wrappers.
///
/// See module-level documentation for detailed explanation on user-data.
pub unsafe trait UserData: Sized + Clone {
    type Target: NativeClass;

    /// Creates a new owned wrapper from a `NativeClass` instance.
    fn new(val: Self::Target) -> Self;

    /// Takes a native instance and returns an opaque pointer that can be used to recover it.
    ///
    /// This gives "ownership" to the engine.
    unsafe fn into_user_data(self) -> *const libc::c_void;

    /// Takes an opaque pointer produced by `into_user_data` and "consumes" it to produce the
    /// original instance, keeping the reference count.
    ///
    /// This should be used when "ownership" is taken from the engine, i.e. destructors.
    /// Use elsewhere can lead to premature drops of the instance contained inside.
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self;

    /// Takes an opaque pointer produced by `into_user_data` and "clones" it to produce the
    /// original instance, increasing the reference count.
    ///
    /// This should be used when user data is "borrowed" from the engine.
    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self;
}

/// Trait for wrappers that can be mapped immutably.
pub trait Map: UserData {
    type Err: Debug;

    /// Maps a `&T` to `U`. Called for methods that take `&self`.
    fn map<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(&Self::Target) -> U;
}

/// Trait for wrappers that can be mapped mutably.
pub trait MapMut: UserData {
    type Err: Debug;

    /// Maps a `&mut T` to `U`. Called for methods that take `&mut self`.
    fn map_mut<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(&mut Self::Target) -> U;
}

/// Trait for user-data wrappers that have a type-checked constructor.
pub unsafe trait TryClone: UserData {
    /// Checked version of "cloning" constructor. This is allowed to spuriously fail, but never
    /// return an invalid result.
    unsafe fn try_clone_from_user_data(ptr: *const libc::c_void) -> Option<Self>;
}

/// Marker trait for user-data wrappers that produce distinct pointers for each Godot instance.
///
/// There is no way for the compiler to test this property, so the trait is unsafe to implement.
/// See documentation on `Tracked` for more information on why this is needed.
pub unsafe trait UniquePtr: UserData {}

/// The default user data wrapper used by derive macro, when no `user_data` attribute is present.
/// This may change in the future.
pub type DefaultUserData<T> = MutexData<T, DefaultLockPolicy>;

pub use tracked::Tracked;

/// Error type indicating that an operation can't fail.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Infallible {}

/// Policies to deal with potential deadlocks
///
/// As Godot allows mutable pointer aliasing, doing certain things in exported method bodies may
/// lead to the engine calling another method on `owner`, leading to another locking attempt
/// within the same thread:
///
/// - Variant calls on anything may dispatch to a script method.
/// - Anything that could emit signals, that are connected to in a non-deferred manner.
///
/// As there is no universal way to deal with such situations, behavior of locking wrappers can
/// be customized using this enum.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum DeadlockPolicy {
    /// Block on all locks. Deadlocks are possible.
    Allow,

    /// Never block on any locks. Methods will return Nil immediately if the lock isn't
    /// available. Deadlocks are prevented.
    Pessimistic,

    /// Block on locks for at most `Duration`. Methods return Nil on timeout. Deadlocks are
    /// prevented.
    Timeout(Duration),
}

/// Trait defining associated constants for locking wrapper options
///
/// This is required because constant generics ([RFC 2000][rfc-2000]) isn't available in stable
/// rust yet.
///
/// See also `DeadlockPolicy`.
///
/// [rfc-2000]: https://github.com/rust-lang/rfcs/blob/master/text/2000-const-generics.md
pub trait LockOptions {
    const DEADLOCK_POLICY: DeadlockPolicy;
}

/// Default lock policy that may change in future versions.
///
/// Currently, it has a deadlock policy of `Allow`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct DefaultLockPolicy;

impl LockOptions for DefaultLockPolicy {
    const DEADLOCK_POLICY: DeadlockPolicy = DeadlockPolicy::Allow;
}

/// Error indicating that a lock wasn't obtained.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct LockFailed;

/// User-data wrapper encapsulating a `Arc<Mutex<T>>`.
///
/// The underlying `Mutex` may change in the future. The current implementation is
/// `parking_lot`.
#[derive(Debug)]
pub struct MutexData<T, OPT = DefaultLockPolicy> {
    lock: Arc<Mutex<T>>,
    _marker: PhantomData<OPT>,
}

unsafe impl<T, OPT> UserData for MutexData<T, OPT>
where
    T: NativeClass + Send,
    OPT: LockOptions,
{
    type Target = T;

    fn new(val: Self::Target) -> Self {
        MutexData {
            lock: Arc::new(Mutex::new(val)),
            _marker: PhantomData,
        }
    }

    unsafe fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.lock) as *const libc::c_void
    }

    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        MutexData {
            lock: Arc::from_raw(ptr as *const Mutex<T>),
            _marker: PhantomData,
        }
    }

    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        let borrowed = Arc::from_raw(ptr as *const Mutex<T>);
        let lock = borrowed.clone();
        mem::forget(borrowed);
        MutexData {
            lock,
            _marker: PhantomData,
        }
    }
}

impl<T, OPT> Map for MutexData<T, OPT>
where
    T: NativeClass + Send,
    OPT: LockOptions,
{
    type Err = LockFailed;

    fn map<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&T) -> U,
    {
        self.map_mut(|val| op(val))
    }
}

impl<T, OPT> MapMut for MutexData<T, OPT>
where
    T: NativeClass + Send,
    OPT: LockOptions,
{
    type Err = LockFailed;

    fn map_mut<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&mut T) -> U,
    {
        let mut guard = match OPT::DEADLOCK_POLICY {
            DeadlockPolicy::Allow => self.lock.lock(),
            DeadlockPolicy::Pessimistic => self.lock.try_lock().ok_or(LockFailed)?,
            DeadlockPolicy::Timeout(dur) => self.lock.try_lock_for(dur).ok_or(LockFailed)?,
        };

        Ok(op(&mut *guard))
    }
}

unsafe impl<T, OPT> UniquePtr for MutexData<T, OPT>
where
    T: NativeClass + Send,
    OPT: LockOptions,
{
}

impl<T, OPT> Clone for MutexData<T, OPT> {
    fn clone(&self) -> Self {
        MutexData {
            lock: self.lock.clone(),
            _marker: PhantomData,
        }
    }
}

/// User-data wrapper encapsulating a `Arc<RwLock<T>>`.
///
/// The underlying `RwLock` may change in the future. The current implementation is
/// `parking_lot`.
#[derive(Debug)]
pub struct RwLockData<T, OPT = DefaultLockPolicy> {
    lock: Arc<RwLock<T>>,
    _marker: PhantomData<OPT>,
}

unsafe impl<T, OPT> UserData for RwLockData<T, OPT>
where
    T: NativeClass + Send + Sync,
    OPT: LockOptions,
{
    type Target = T;

    fn new(val: Self::Target) -> Self {
        RwLockData {
            lock: Arc::new(RwLock::new(val)),
            _marker: PhantomData,
        }
    }

    unsafe fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.lock) as *const libc::c_void
    }

    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        RwLockData {
            lock: Arc::from_raw(ptr as *const RwLock<T>),
            _marker: PhantomData,
        }
    }

    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        let borrowed = Arc::from_raw(ptr as *const RwLock<T>);
        let lock = borrowed.clone();
        mem::forget(borrowed);
        RwLockData {
            lock,
            _marker: PhantomData,
        }
    }
}

impl<T, OPT> Map for RwLockData<T, OPT>
where
    T: NativeClass + Send + Sync,
    OPT: LockOptions,
{
    type Err = LockFailed;

    fn map<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&T) -> U,
    {
        let guard = match OPT::DEADLOCK_POLICY {
            DeadlockPolicy::Allow => self.lock.read(),
            DeadlockPolicy::Pessimistic => self.lock.try_read().ok_or(LockFailed)?,
            DeadlockPolicy::Timeout(dur) => self.lock.try_read_for(dur).ok_or(LockFailed)?,
        };

        Ok(op(&*guard))
    }
}

impl<T, OPT> MapMut for RwLockData<T, OPT>
where
    T: NativeClass + Send + Sync,
    OPT: LockOptions,
{
    type Err = LockFailed;

    fn map_mut<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&mut T) -> U,
    {
        let mut guard = match OPT::DEADLOCK_POLICY {
            DeadlockPolicy::Allow => self.lock.write(),
            DeadlockPolicy::Pessimistic => self.lock.try_write().ok_or(LockFailed)?,
            DeadlockPolicy::Timeout(dur) => self.lock.try_write_for(dur).ok_or(LockFailed)?,
        };

        Ok(op(&mut *guard))
    }
}

unsafe impl<T, OPT> UniquePtr for RwLockData<T, OPT>
where
    T: NativeClass + Send + Sync,
    OPT: LockOptions,
{
}

impl<T, OPT> Clone for RwLockData<T, OPT> {
    fn clone(&self) -> Self {
        RwLockData {
            lock: self.lock.clone(),
            _marker: PhantomData,
        }
    }
}

/// User-data wrapper encapsulating a `Arc<T>`. Does not implement `MapMut`.
#[derive(Debug)]
pub struct ArcData<T>(Arc<T>);

unsafe impl<T> UserData for ArcData<T>
where
    T: NativeClass + Send + Sync,
{
    type Target = T;

    fn new(val: Self::Target) -> Self {
        ArcData(Arc::new(val))
    }

    unsafe fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.0) as *const libc::c_void
    }

    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        ArcData(Arc::from_raw(ptr as *const T))
    }

    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        let borrowed = Arc::from_raw(ptr as *const T);
        let arc = borrowed.clone();
        mem::forget(borrowed);
        ArcData(arc)
    }
}

impl<T> Map for ArcData<T>
where
    T: NativeClass + Send + Sync,
{
    type Err = Infallible;

    fn map<F, U>(&self, op: F) -> Result<U, Infallible>
    where
        F: FnOnce(&T) -> U,
    {
        Ok(op(&*self.0))
    }
}

unsafe impl<T> UniquePtr for ArcData<T> where T: NativeClass + Send + Sync {}

impl<T> Clone for ArcData<T> {
    fn clone(&self) -> Self {
        ArcData(self.0.clone())
    }
}

mod tracked {
    //! User-data wrapper adapter with type checking via tracking

    use std::any::TypeId;
    use std::collections::{HashMap, HashSet};

    use parking_lot::RwLock;

    use super::{Map, MapMut, TryClone, UniquePtr, UserData};

    lazy_static! {
        static ref TRACKED_POINTERS: RwLock<HashMap<TypeId, HashSet<usize>>> =
            { RwLock::new(HashMap::new()) };
    }

    /// A `TryClone` user-data wrapper adapter that allows instance downcasting by tracking
    /// user-data pointers in a global `HashMap`.
    ///
    /// The `HashMap` is accessed through an `RwLock` on construction, destruction, and downcasts.
    /// There is no additional runtime cost on calls from Godot.
    ///
    /// The cast is incomplete, as in, not all valid values will pass the type check.
    /// Specifically:
    ///
    /// - Null pointers will always fail to type check, despite being valid pointers for ZSTs.
    ///   This is usually fine because user-data wrappers usually need some state, and are
    ///   unlikely to produce a null pointer.
    /// - If the user-data is consumed, then even if the wrapped data is not dropped yet, further
    ///   attempts to check the same pointer will fail. This is fine because it can have no valid
    ///   owner in this case.
    /// - If multiple instances of the underlying wrapper produce the same user-data pointer,
    ///   (e.g. a singleton, or a type that pulls values out of aether), then all further
    ///   instances will fail to check by the time the first instance is consumed. To prevent
    ///   this from happening, the marker trait `UniquePtr` is used as a bound on the inner
    ///   wrapper. Violations of `UniquePtr`'s protocol will trigger debug assertions.
    #[derive(Clone, Debug)]
    pub struct Tracked<UD> {
        data: UD,
    }

    unsafe impl<UD> UserData for Tracked<UD>
    where
        UD: UserData + UniquePtr,
        UD::Target: 'static,
    {
        type Target = UD::Target;

        fn new(val: Self::Target) -> Self {
            // This only creates an instance owned by Rust, so no valid objects can exist yet.
            Tracked { data: UD::new(val) }
        }

        unsafe fn into_user_data(self) -> *const libc::c_void {
            let ptr = self.data.into_user_data() as *const libc::c_void;
            {
                // Only when the ownership is passed to Godot, does it become possible for an
                // Godot object to be a valid instance of UD::Target. So, the pointer is added
                // to the map at this point.
                let mut ptr_map = TRACKED_POINTERS.write();
                let ptrs = ptr_map
                    .entry(TypeId::of::<UD::Target>())
                    .or_insert_with(HashSet::new);
                let ptr_is_new = ptrs.insert(ptr as usize);
                debug_assert!(
                    ptr_is_new,
                    "pointer obtained from into_user_data should not be in the set"
                );
            }
            ptr
        }

        unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
            {
                // When the ownership is taken back from Godot, there can't be valid objects that
                // should still be considered an instance of UD::Target. Thus, the pointer is
                // removed from the map.
                let mut ptr_map = TRACKED_POINTERS.write();
                match ptr_map.get_mut(&TypeId::of::<UD::Target>()) {
                    Some(ptrs) => {
                        let ptr_is_there = ptrs.remove(&(ptr as usize));
                        debug_assert!(ptr_is_there, "pointer should be in the set of UD::Target");
                    }
                    None => {
                        debug_assert!(false, "pointer set should have been created by now");
                    }
                }
            }
            Tracked {
                data: UD::consume_user_data_unchecked(ptr),
            }
        }

        unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self {
            Tracked {
                data: UD::clone_from_user_data_unchecked(ptr),
            }
        }
    }

    impl<UD> Map for Tracked<UD>
    where
        UD: UserData + Map + UniquePtr,
        UD::Target: 'static,
    {
        type Err = UD::Err;

        fn map<F, U>(&self, op: F) -> Result<U, UD::Err>
        where
            F: FnOnce(&UD::Target) -> U,
        {
            self.data.map(op)
        }
    }

    impl<UD> MapMut for Tracked<UD>
    where
        UD: UserData + MapMut + UniquePtr,
        UD::Target: 'static,
    {
        type Err = UD::Err;

        fn map_mut<F, U>(&self, op: F) -> Result<U, UD::Err>
        where
            F: FnOnce(&mut UD::Target) -> U,
        {
            self.data.map_mut(op)
        }
    }

    unsafe impl<UD> TryClone for Tracked<UD>
    where
        UD: UserData + UniquePtr,
        UD::Target: 'static,
    {
        unsafe fn try_clone_from_user_data(ptr: *const libc::c_void) -> Option<Self> {
            if ptr.is_null() {
                return None;
            }

            let result = {
                let ptr_map = TRACKED_POINTERS.read();
                let ptrs = ptr_map.get(&TypeId::of::<UD::Target>())?;

                // The inner user data must be constructed before the read guard is dropped,
                // because otherwise it might be consumed between the check and construction.
                if ptrs.contains(&(ptr as usize)) {
                    Some(Tracked {
                        data: UD::clone_from_user_data_unchecked(ptr),
                    })
                } else {
                    None
                }
            };

            result
        }
    }
}
