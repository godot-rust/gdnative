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
//! threads by the engine or other scripts. Thus, to maintain safety, wrapper types are to be used
//! to make sure that the Rust rules for references are always held for the `self` argument, and
//! no UB can occur because we freed `owner` or put another script on it.
//!
//! ## Choosing a wrapper type
//!
//! The default user data wrapper used by derive macro, when no `user_data` attribute is present,
//! is defined as the type alias `DefaultUserData<T>`. Currently, it is `LocalCellData<T>`. This
//! may change in the future, and changes to it will not be considered breaking changes.
//!
//! To choose another wrapper type, put the `#[user_data]` attribute on your `NativeScript` type
//! if you are using the derive macro:
//!
//! ```ignore
//! #[derive(NativeClass)]
//! #[inherit(gdnative::api::Node)]
//! #[user_data(gdnative::export::user_data::MutexData<HelloWorld>)]
//! struct HelloWorld;
//! ```
//!
//! ...or, if you are implementing `NativeScript` manually, set the `UserData` associated type
//! to the type you choose.
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
//! ### Use a `LocalCellData<T>` when:
//!
//! - Your `NativeClass` type is not `Send`, and you will only ever use it from the thread where
//!   it's originally created.
//!
//! ### Use `Aether<T>` when:
//!
//! - Your `NativeClass` type is a zero-sized type (ZST) that is `Copy + Default`.
//! - You don't need to do anything special in `Drop`.

use parking_lot::{Mutex, RwLock};
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use std::time::Duration;

use crate::export::NativeClass;

/// Trait for customizable user-data wrappers.
///
/// See module-level documentation for detailed explanation on user-data.
pub unsafe trait UserData: Sized + Clone {
    type Target: NativeClass;

    /// Creates a new owned wrapper from a `NativeClass` instance.
    ///
    /// This operation must never fail.
    fn new(val: Self::Target) -> Self;

    /// Takes a native instance and returns an opaque pointer that can be used to recover it.
    ///
    /// This gives "ownership" to the engine.
    ///
    /// This operation must never fail.
    fn into_user_data(self) -> *const libc::c_void;

    /// Takes an opaque pointer produced by `into_user_data` and "consumes" it to produce the
    /// original instance, keeping the reference count.
    ///
    /// This should be used when "ownership" is taken from the engine, i.e. destructors.
    /// Use elsewhere can lead to premature drops of the instance contained inside.
    ///
    /// `ptr` is guaranteed to be non-null.
    ///
    /// This operation must never fail.
    ///
    /// # Safety
    ///
    /// `ptr` must be pointing to valid data of the correct type.
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self;

    /// Takes an opaque pointer produced by `into_user_data` and "clones" it to produce the
    /// original instance, increasing the reference count.
    ///
    /// This should be used when user data is "borrowed" from the engine.
    ///
    /// `ptr` is guaranteed to be non-null.
    ///
    /// This operation must never fail.
    ///
    /// # Safety
    ///
    /// `ptr` must be pointing to valid data of the correct type.
    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self;
}

/// Trait for wrappers that can be mapped immutably.
pub trait Map: UserData {
    type Err: std::error::Error;

    /// Maps a `&T` to `U`. Called for methods that take `&self`.
    ///
    /// Implementations of this method must not panic. Failures should be indicated by
    /// returning `Err`.
    fn map<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(&Self::Target) -> U;
}

/// Trait for wrappers that can be mapped mutably.
pub trait MapMut: UserData {
    type Err: std::error::Error;

    /// Maps a `&mut T` to `U`. Called for methods that take `&mut self`.
    ///
    /// Implementations of this method must not panic. Failures should be indicated by
    /// returning `Err`.
    fn map_mut<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(&mut Self::Target) -> U;
}

/// Trait for wrappers that can be mapped once.
pub trait MapOwned: UserData {
    type Err: std::error::Error;

    /// Maps a `T` to `U`. Called for methods that take `self`. This method may fail with
    /// an error if it is called more than once on the same object.
    ///
    /// Implementations of this method must not panic. Failures should be indicated by
    /// returning `Err`.
    fn map_owned<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(Self::Target) -> U;
}

/// The default user data wrapper used by derive macro, when no `user_data` attribute is present.
/// This may change in the future.
pub type DefaultUserData<T> = LocalCellData<T>;

/// Error type indicating that an operation can't fail.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[allow(clippy::exhaustive_enums)] // explicitly uninhabited
pub enum Infallible {}

impl std::error::Error for Infallible {}
impl std::fmt::Display for Infallible {
    #[inline]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!("uninhabited enum")
    }
}

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
#[non_exhaustive]
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[non_exhaustive]
pub enum LockFailed {
    Pessimistic,
    Timeout(Duration),
}

impl std::fmt::Display for LockFailed {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockFailed::Timeout(wait) => write!(f, "failed to acquire lock within {wait:?}"),
            LockFailed::Pessimistic => write!(f, "failed to acquire lock, it was already held"),
        }
    }
}
impl std::error::Error for LockFailed {}

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

    #[inline]
    fn new(val: Self::Target) -> Self {
        MutexData {
            lock: Arc::new(Mutex::new(val)),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.lock) as *const libc::c_void
    }

    #[inline]
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        MutexData {
            lock: Arc::from_raw(ptr as *const Mutex<T>),
            _marker: PhantomData,
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    fn map_mut<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&mut T) -> U,
    {
        let mut guard = match OPT::DEADLOCK_POLICY {
            DeadlockPolicy::Allow => self.lock.lock(),
            DeadlockPolicy::Pessimistic => self.lock.try_lock().ok_or(LockFailed::Pessimistic)?,
            DeadlockPolicy::Timeout(dur) => self
                .lock
                .try_lock_for(dur)
                .ok_or(LockFailed::Timeout(dur))?,
        };

        Ok(op(&mut *guard))
    }
}

impl<T, OPT> Clone for MutexData<T, OPT> {
    #[inline]
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

    #[inline]
    fn new(val: Self::Target) -> Self {
        RwLockData {
            lock: Arc::new(RwLock::new(val)),
            _marker: PhantomData,
        }
    }

    #[inline]
    fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.lock) as *const libc::c_void
    }

    #[inline]
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        RwLockData {
            lock: Arc::from_raw(ptr as *const RwLock<T>),
            _marker: PhantomData,
        }
    }

    #[inline]
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

    #[inline]
    fn map<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&T) -> U,
    {
        let guard = match OPT::DEADLOCK_POLICY {
            DeadlockPolicy::Allow => self.lock.read(),
            DeadlockPolicy::Pessimistic => self.lock.try_read().ok_or(LockFailed::Pessimistic)?,
            DeadlockPolicy::Timeout(dur) => self
                .lock
                .try_read_for(dur)
                .ok_or(LockFailed::Timeout(dur))?,
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

    #[inline]
    fn map_mut<F, U>(&self, op: F) -> Result<U, LockFailed>
    where
        F: FnOnce(&mut T) -> U,
    {
        let mut guard = match OPT::DEADLOCK_POLICY {
            DeadlockPolicy::Allow => self.lock.write(),
            DeadlockPolicy::Pessimistic => self.lock.try_write().ok_or(LockFailed::Pessimistic)?,
            DeadlockPolicy::Timeout(dur) => self
                .lock
                .try_write_for(dur)
                .ok_or(LockFailed::Timeout(dur))?,
        };

        Ok(op(&mut *guard))
    }
}

impl<T, OPT> Clone for RwLockData<T, OPT> {
    #[inline]
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

impl<T> ArcData<T> {
    /// Returns the internal `Arc<T>`. Useful for API's that require an `Arc`
    /// directly, or for coercing it into a trait object.
    ///
    /// Note that this removes
    /// the restriction of only being able to access the `NativeClass` instance
    /// temporarily through the `Map` trait; however, it should be exactly as safe
    /// as permanently storing an owned `ArcData` and then calling `.map()` on
    /// it later.
    #[inline]
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

unsafe impl<T> UserData for ArcData<T>
where
    T: NativeClass + Send + Sync,
{
    type Target = T;

    #[inline]
    fn new(val: Self::Target) -> Self {
        ArcData(Arc::new(val))
    }

    #[inline]
    fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.0) as *const libc::c_void
    }

    #[inline]
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        ArcData(Arc::from_raw(ptr as *const T))
    }

    #[inline]
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

    #[inline]
    fn map<F, U>(&self, op: F) -> Result<U, Infallible>
    where
        F: FnOnce(&T) -> U,
    {
        Ok(op(&*self.0))
    }
}

impl<T> Clone for ArcData<T> {
    #[inline]
    fn clone(&self) -> Self {
        ArcData(self.0.clone())
    }
}

/// User-data wrapper analogous to a `Arc<RefCell<T>>`, that is restricted to the thread
/// where it was originally created. The destructor of `T` is not guaranteed to be run if
/// this is actually shared across multiple threads.
///
/// This works by checking `ThreadId` before touching the underlying reference. If the id
/// doesn't match the original thread, `map` and `map_mut` will return an error.
///
/// Since it isn't possible to control where the last reference is dropped, the destructor
/// isn't run if the last reference is dropped on a different thread than the one where the
/// wrapper was originally created. To ensure that values are cleaned up properly, keep the
/// game single-threaded, or use a real thread-safe wrapper such as [`MutexData`] instead.
#[derive(Debug)]
pub struct LocalCellData<T> {
    inner: Arc<local_cell::LocalCell<T>>,
}

pub use self::local_cell::LocalCellError;

mod local_cell {
    use std::cell::{Ref, RefCell, RefMut};
    use std::mem::ManuallyDrop;
    use std::thread::{self, ThreadId};

    #[derive(Debug)]
    pub struct LocalCell<T> {
        thread_id: ThreadId,
        cell: RefCell<ManuallyDrop<T>>,
    }

    impl<T> Drop for LocalCell<T> {
        fn drop(&mut self) {
            if self.thread_id == thread::current().id() {
                unsafe {
                    ManuallyDrop::drop(self.cell.get_mut());
                }
            }
        }
    }

    /// Error indicating that a borrow has failed.
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    #[non_exhaustive]
    pub enum LocalCellError {
        DifferentThread {
            original: ThreadId,
            current: ThreadId,
        },
        BorrowFailed,
    }

    impl std::fmt::Display for LocalCellError {
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                LocalCellError::DifferentThread { original, current } => write!(
                    f,
                    "accessing from the wrong thread, expected {original:?} found {current:?}"
                ),
                LocalCellError::BorrowFailed => write!(
                    f,
                    "borrow failed; a &mut reference was requested, but one already exists. The cause is likely a re-entrant call \
                    (e.g. a GDNative Rust method calls to GDScript, which again calls a Rust method on the same object)"
                ),
            }
        }
    }

    impl std::error::Error for LocalCellError {}

    impl<T> LocalCell<T> {
        #[inline]
        pub fn new(val: T) -> Self {
            LocalCell {
                thread_id: thread::current().id(),
                cell: RefCell::new(ManuallyDrop::new(val)),
            }
        }

        #[inline]
        fn inner(&self) -> Result<&RefCell<ManuallyDrop<T>>, LocalCellError> {
            let current = thread::current().id();

            if self.thread_id == current {
                Ok(&self.cell)
            } else {
                Err(LocalCellError::DifferentThread {
                    original: self.thread_id,
                    current,
                })
            }
        }

        #[inline]
        pub fn try_borrow(&self) -> Result<Ref<ManuallyDrop<T>>, LocalCellError> {
            let inner = self.inner()?;
            inner.try_borrow().map_err(|_| LocalCellError::BorrowFailed)
        }

        #[inline]
        pub fn try_borrow_mut(&self) -> Result<RefMut<ManuallyDrop<T>>, LocalCellError> {
            let inner = self.inner()?;
            inner
                .try_borrow_mut()
                .map_err(|_| LocalCellError::BorrowFailed)
        }
    }

    // Implementing Send + Sync is ok because the cell is guarded from access outside the
    // original thread.
    unsafe impl<T> Send for LocalCell<T> {}
    unsafe impl<T> Sync for LocalCell<T> {}
}

unsafe impl<T> UserData for LocalCellData<T>
where
    T: NativeClass,
{
    type Target = T;

    #[inline]
    fn new(val: Self::Target) -> Self {
        LocalCellData {
            inner: Arc::new(local_cell::LocalCell::new(val)),
        }
    }

    #[inline]
    fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.inner) as *const libc::c_void
    }

    #[inline]
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        LocalCellData {
            inner: Arc::from_raw(ptr as *const local_cell::LocalCell<T>),
        }
    }

    #[inline]
    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        let borrowed = Arc::from_raw(ptr as *const local_cell::LocalCell<T>);
        let arc = borrowed.clone();
        mem::forget(borrowed);
        LocalCellData { inner: arc }
    }
}

impl<T> Map for LocalCellData<T>
where
    T: NativeClass,
{
    type Err = LocalCellError;

    #[inline]
    fn map<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(&Self::Target) -> U,
    {
        self.inner.try_borrow().map(|r| op(&r))
    }
}

impl<T> MapMut for LocalCellData<T>
where
    T: NativeClass,
{
    type Err = LocalCellError;

    #[inline]
    fn map_mut<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(&mut Self::Target) -> U,
    {
        self.inner.try_borrow_mut().map(|mut w| op(&mut w))
    }
}

impl<T> Clone for LocalCellData<T> {
    #[inline]
    fn clone(&self) -> Self {
        LocalCellData {
            inner: self.inner.clone(),
        }
    }
}

/// Special user-data wrapper intended for zero-sized types, that does not perform any
/// allocation or synchronization at runtime. Does not implement `MapMut`.
///
/// `Aether` produces a value using `Default` each time it's mapped. This is most useful when
/// used with auto-load scripts to simulate static functions, since actual static functions
/// can't be exported in GDNative.
#[derive(Copy, Debug)]
pub struct Aether<T> {
    _marker: PhantomData<T>,
}

unsafe impl<T> Send for Aether<T> {}
unsafe impl<T> Sync for Aether<T> {}

impl<T> Clone for Aether<T> {
    #[inline]
    fn clone(&self) -> Self {
        Aether::default()
    }
}

impl<T> Default for Aether<T> {
    #[inline]
    fn default() -> Self {
        Aether {
            _marker: PhantomData,
        }
    }
}

unsafe impl<T> UserData for Aether<T>
where
    T: NativeClass + Copy + Default,
{
    type Target = T;

    #[inline]
    fn new(_val: Self::Target) -> Self {
        Aether::default()
    }

    #[inline]
    fn into_user_data(self) -> *const libc::c_void {
        1 as *const libc::c_void
    }

    #[inline]
    unsafe fn consume_user_data_unchecked(_ptr: *const libc::c_void) -> Self {
        Aether::default()
    }

    #[inline]
    unsafe fn clone_from_user_data_unchecked(_ptr: *const libc::c_void) -> Self {
        Aether::default()
    }
}

impl<T> Map for Aether<T>
where
    T: NativeClass + Copy + Default,
{
    type Err = Infallible;

    #[inline]
    fn map<F, U>(&self, op: F) -> Result<U, Infallible>
    where
        F: FnOnce(&T) -> U,
    {
        Ok(op(&Default::default()))
    }
}

/// Special user-data wrapper intended for objects that can only be used once. Only
/// implements `MapOwned`.
pub struct Once<T>(Arc<atomic_take::AtomicTake<T>>);

impl<T> Clone for Once<T> {
    #[inline]
    fn clone(&self) -> Self {
        Once(Arc::clone(&self.0))
    }
}

unsafe impl<T> UserData for Once<T>
where
    T: NativeClass + Send,
{
    type Target = T;

    #[inline]
    fn new(val: Self::Target) -> Self {
        Once(Arc::new(atomic_take::AtomicTake::new(val)))
    }

    #[inline]
    fn into_user_data(self) -> *const libc::c_void {
        Arc::into_raw(self.0) as *const _
    }

    #[inline]
    unsafe fn consume_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        Once(Arc::from_raw(ptr as *const _))
    }

    #[inline]
    unsafe fn clone_from_user_data_unchecked(ptr: *const libc::c_void) -> Self {
        let borrowed = Arc::from_raw(ptr as *const _);
        let arc = Arc::clone(&borrowed);
        mem::forget(borrowed);
        Once(arc)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ValueTaken;

impl std::error::Error for ValueTaken {}
impl Display for ValueTaken {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "this object has already been used once")
    }
}

impl<T> MapOwned for Once<T>
where
    T: NativeClass + Send,
{
    type Err = ValueTaken;

    /// Maps a `T` to `U`. Called for methods that take `self`. This method may fail with
    /// an error if it is called more than once on the same object.
    ///
    /// Implementations of this method must not panic. Failures should be indicated by
    /// returning `Err`.
    #[inline]
    fn map_owned<F, U>(&self, op: F) -> Result<U, Self::Err>
    where
        F: FnOnce(Self::Target) -> U,
    {
        let v = self.0.take().ok_or(ValueTaken)?;
        Ok(op(v))
    }
}
