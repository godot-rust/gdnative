use std::ptr::NonNull;

use crate::core_types::{
    FromVariant, FromVariantError, GodotString, OwnedToVariant, ToVariant, Variant,
};
use crate::nativescript::init::ClassBuilder;
use crate::nativescript::{Map, MapMut, MapOwned, UserData};
use crate::object::{
    AssumeSafeLifetime, LifetimeConstraint, QueueFree, RawObject, Ref, RefImplBound, SafeAsRaw,
    SafeDeref, TRef,
};
use crate::object::{GodotObject, Instanciable};
use crate::private::{get_api, ReferenceCountedClassPlaceholder};
use crate::ref_kind::{ManuallyManaged, RefCounted};
use crate::thread_access::{NonUniqueThreadAccess, Shared, ThreadAccess, ThreadLocal, Unique};

use super::emplace;

/// Trait used for describing and initializing a Godot script class.
///
/// This trait is used to provide data and functionality to the
/// "data-part" of the class, such as name, initialization and information
/// about exported properties.
///
/// A derive macro is available for this trait. See documentation on the
/// `NativeClass` macro for detailed usage and examples.
///
/// For exported methods, see the [`NativeClassMethods`] trait.
///
/// [`NativeClassMethods`]: ./trait.NativeClassMethods.html
pub trait NativeClass: Sized + 'static {
    /// Base type of the class.
    ///
    /// In Godot, scripting languages can define "script instances" which can be
    /// attached to objects. Because of the dynamic nature, the intended "inheritance"
    /// is not easily implementable properly.
    ///
    /// Instead, delegation is used and most calls to a Godot object query the script instance
    /// first. This way, some methods can be "overwritten" and new ones can be exposed.
    ///
    /// This only works when using so called "variant calls", since the querying of the script
    /// instance is performed there.
    /// When not using variant calls, any direct(*) calls have to be made to the Godot object
    /// directly.
    ///
    /// The base type describes the "most general" type of object this script class can be
    /// attached to.
    ///
    /// *(\*)*: GDNative enables use of "ptrcall"s, which are wrappers for function pointers.
    /// Those do not do explicit checks for script implementations **unless the method
    /// implementation does**.
    type Base: GodotObject;

    /// User-data wrapper type of the class.
    ///
    /// See module-level documentation on `user_data` for more info.
    type UserData: UserData<Target = Self>;

    /// The name of the class.
    ///
    /// In GDNative+NativeScript many classes can be defined in one dynamic library.
    /// To identify which class has to be used, a library-unique name has to be given.
    fn class_name() -> &'static str;

    /// Function that creates a value of `Self`, used for the script-instance. The default
    /// implementation simply panics.
    ///
    /// This function has a reference to the owner object as a parameter, which can be used to
    /// set state on the owner upon creation or to query values
    ///
    /// It is possible to declare script classes without zero-argument constructors. Instances
    /// of such scripts can only be created from Rust using `Instance::emplace`. See
    /// documentation on `Instance::emplace` for an example.
    #[inline]
    fn init(_owner: TRef<'_, Self::Base, Shared>) -> Self {
        panic!(
            "{} does not have a zero-argument constructor",
            Self::class_name()
        )
    }

    /// Register any exported properties to Godot.
    #[inline]
    fn register_properties(_builder: &ClassBuilder<Self>) {}

    /// Convenience method to create an `Instance<Self, Unique>`. This is a new `Self::Base`
    /// with the script attached.
    ///
    /// If `Self::Base` is manually-managed, then the resulting `Instance` must be passed to
    /// the engine or manually freed with `Instance::free`. Otherwise, the base object will be
    /// leaked.
    ///
    /// Must be called after the library is initialized.
    #[inline]
    fn new_instance() -> Instance<Self, Unique>
    where
        Self::Base: Instanciable,
    {
        Instance::new()
    }

    /// Convenience method to emplace `self` into an `Instance<Self, Unique>`. This is a new
    /// `Self::Base` with the script attached.
    ///
    /// If `Self::Base` is manually-managed, then the resulting `Instance` must be passed to
    /// the engine or manually freed with `Instance::free`. Otherwise, the base object will be
    /// leaked.
    ///
    /// Must be called after the library is initialized.
    #[inline]
    fn emplace(self) -> Instance<Self, Unique>
    where
        Self::Base: Instanciable,
    {
        Instance::emplace(self)
    }
}

/// Trait used to provide information of Godot-exposed methods of a script class.
pub trait NativeClassMethods: NativeClass {
    /// Function that registers all exposed methods to Godot.
    fn register(builder: &ClassBuilder<Self>);
}

/// Trait for types that can be used as the `owner` arguments of exported methods. This trait
/// is sealed and has no public interface.
///
/// # Safety
///
/// Whenever a NativeScript methods is called, it's assumed that the owner is safe to use.
/// When calling a method that may call non-thread-safe methods on its owner from non-Rust
/// code, the official [thread-safety guidelines][thread-safety] must be followed to prevent
/// undefined behavior.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub trait OwnerArg<'a, T: GodotObject, Access: ThreadAccess + 'static>: private::Sealed {
    #[doc(hidden)]
    fn from_safe_ref(owner: TRef<'a, T, Access>) -> Self;
}

impl<'a, T> private::Sealed for &'a T where T: GodotObject {}
impl<'a, T, Access> OwnerArg<'a, T, Access> for &'a T
where
    T: GodotObject,
    Access: ThreadAccess + 'static,
{
    #[inline]
    fn from_safe_ref(owner: TRef<'a, T, Access>) -> Self {
        owner.as_ref()
    }
}

impl<'a, T, Access> private::Sealed for TRef<'a, T, Access>
where
    T: GodotObject,
    Access: ThreadAccess + 'static,
{
}
impl<'a, T, Access> OwnerArg<'a, T, Access> for TRef<'a, T, Access>
where
    T: GodotObject,
    Access: ThreadAccess + 'static,
{
    #[inline]
    fn from_safe_ref(owner: TRef<'a, T, Access>) -> Self {
        owner
    }
}

/// A persistent reference to a GodotObject with a rust NativeClass attached.
///
/// `Instance`s can be worked on directly using `map` and `map_mut` if the base object is
/// reference-counted. Otherwise, use `assume_safe` to obtain a temporary `RefInstance`.
///
/// See the type-level documentation on `Ref` for more information on typed thread accesses.
#[derive(Debug)]
pub struct Instance<T: NativeClass, Access: ThreadAccess> {
    owner: Ref<T::Base, Access>,
    script: T::UserData,
}

/// A reference to a GodotObject with a rust NativeClass attached that is assumed safe during
/// a certain lifetime.
///
/// See the type-level documentation on `Ref` for more information on typed thread accesses.
#[derive(Debug)]
pub struct RefInstance<'a, T: NativeClass, Access: ThreadAccess> {
    owner: TRef<'a, T::Base, Access>,
    script: T::UserData,
}

impl<T: NativeClass> Instance<T, Unique> {
    /// Creates a `T::Base` with the script `T` attached. Both `T::Base` and `T` must have zero
    /// argument constructors.
    ///
    /// If `T::Base` is manually-managed, then the resulting `Instance` must be passed to
    /// the engine or manually freed with `Instance::free`. Otherwise, the base object will be
    /// leaked.
    ///
    /// Must be called after the library is initialized.
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self
    where
        T::Base: Instanciable,
    {
        Self::maybe_emplace(None)
    }

    /// Creates a `T::Base` with a given instance of the script `T` attached. `T::Base` must
    /// have a zero-argument constructor.
    ///
    /// This may be used to create instances of scripts that do not have zero-argument
    /// constructors:
    ///
    /// ```ignore
    /// // This type does not have a zero-argument constructor. As a result, `Instance::new`
    /// // will panic and `Foo.new` from GDScript will result in errors when the object is used.
    /// #[derive(NativeScript)]
    /// #[inherit(Reference)]
    /// #[no_constructor]
    /// struct MyIntVector(i64, i64);
    ///
    /// #[methods]
    /// impl MyIntVector {
    ///     // - snip -
    /// }
    ///
    /// // With `Instance::emplace`, however, we can expose "constructors" from a factory
    /// // auto-load for our script type.
    /// #[derive(NativeScript)]
    /// #[inherit(Node)]
    /// #[user_data(Aether<Self>)]
    /// struct MyIntVectorFactory;
    ///
    /// #[methods]
    /// impl MyIntVectorFactory {
    ///     #[export]
    ///     fn make(&self, _owner: &Node, x: i64, y: i64) -> Instance<MyIntVector, Unique> {
    ///         Instance::emplace(MyIntVector(x, y))
    ///     }
    /// }
    /// ```
    ///
    /// If `T::Base` is manually-managed, then the resulting `Instance` must be passed to
    /// the engine or manually freed with `Instance::free`. Otherwise, the base object will be
    /// leaked.
    ///
    /// Must be called after the library is initialized.
    #[inline]
    pub fn emplace(script: T) -> Self
    where
        T::Base: Instanciable,
    {
        Self::maybe_emplace(Some(script))
    }

    fn maybe_emplace(script: Option<T>) -> Self
    where
        T::Base: Instanciable,
    {
        unsafe {
            let gd_api = get_api();
            let nativescript_methods = crate::private::NativeScriptMethodTable::get(gd_api);

            assert_ne!(
                std::ptr::null(),
                nativescript_methods.set_class_name,
                "NativeScript::set_class_name must be available"
            );
            assert_ne!(
                std::ptr::null(),
                nativescript_methods.set_library,
                "NativeScript::set_library must be available"
            );
            assert_ne!(
                std::ptr::null(),
                nativescript_methods.new,
                "NativeScript::new must be available"
            );

            // The API functions take NUL-terminated C strings. &CStr is not used for its runtime cost.
            let class_name = b"NativeScript\0".as_ptr() as *const libc::c_char;
            let ctor = (gd_api.godot_get_class_constructor)(class_name).unwrap();

            let native_script =
                NonNull::new(ctor()).expect("NativeScript constructor should not return null");
            let native_script =
                RawObject::<ReferenceCountedClassPlaceholder>::from_sys_ref_unchecked(
                    native_script,
                );
            native_script.init_ref_count();

            let script_class_name = GodotString::from(T::class_name());
            let mut args: [*const libc::c_void; 1] = [script_class_name.sys() as *const _];
            (gd_api.godot_method_bind_ptrcall)(
                nativescript_methods.set_class_name,
                native_script.sys().as_ptr(),
                args.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            let mut args: [*const libc::c_void; 1] = [crate::private::get_gdnative_library_sys()];
            (gd_api.godot_method_bind_ptrcall)(
                nativescript_methods.set_library,
                native_script.sys().as_ptr(),
                args.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            if let Some(script) = script {
                emplace::place(script);
            }

            let mut args: [*const sys::godot_variant; 0] = [];
            let variant = (gd_api.godot_method_bind_call)(
                nativescript_methods.new,
                native_script.sys().as_ptr(),
                args.as_mut_ptr(),
                0,
                std::ptr::null_mut(),
            );

            assert!(
                emplace::take::<T>().is_none(),
                "emplacement value should be taken by the constructor wrapper (this is a bug in the bindings)",
            );

            let variant = Variant::from_sys(variant);

            let owner = variant
                .try_to_object::<T::Base>()
                .expect("base object should be of the correct type (is the script registered?)")
                .assume_unique();

            let script_ptr =
                (gd_api.godot_nativescript_get_userdata)(owner.sys()) as *const libc::c_void;

            assert_ne!(
                std::ptr::null(),
                script_ptr,
                "script instance should not be null (did the constructor fail?)"
            );

            let script = T::UserData::clone_from_user_data_unchecked(script_ptr);

            native_script.unref();

            Instance { owner, script }
        }
    }
}

impl<T: NativeClass, Access: ThreadAccess> Instance<T, Access> {
    /// Returns the base object, dropping the script wrapper.
    #[inline]
    pub fn into_base(self) -> Ref<T::Base, Access> {
        self.owner
    }

    /// Returns the script wrapper.
    #[inline]
    pub fn into_script(self) -> T::UserData {
        self.script
    }

    /// Returns the base object and the script wrapper.
    #[inline]
    pub fn decouple(self) -> (Ref<T::Base, Access>, T::UserData) {
        (self.owner, self.script)
    }

    /// Returns a reference to the base object.
    #[inline]
    pub fn base(&self) -> &Ref<T::Base, Access> {
        &self.owner
    }

    /// Returns a reference to the script wrapper.
    #[inline]
    pub fn script(&self) -> &T::UserData {
        &self.script
    }
}

impl<T: NativeClass, Access: ThreadAccess> Instance<T, Access>
where
    RefImplBound: SafeAsRaw<<T::Base as GodotObject>::RefKind, Access>,
{
    /// Try to downcast `Ref<T::Base, Access>` to `Instance<T>`, without changing the reference
    /// count if reference-counted.
    ///
    /// # Errors
    ///
    /// Returns the original `Ref` if the cast failed.
    #[inline]
    pub fn try_from_base(owner: Ref<T::Base, Access>) -> Result<Self, Ref<T::Base, Access>> {
        let user_data = match try_get_user_data_ptr::<T>(owner.as_raw()) {
            Some(user_data) => user_data,
            None => return Err(owner),
        };

        let script = unsafe { T::UserData::clone_from_user_data_unchecked(user_data) };

        Ok(Instance { owner, script })
    }

    /// Try to downcast `Ref<T::Base, Access>` to `Instance<T>`, without changing the reference
    /// count if reference-counted. Shorthand for `Self::try_from_base().ok()`.
    #[inline]
    pub fn from_base(owner: Ref<T::Base, Access>) -> Option<Self> {
        Self::try_from_base(owner).ok()
    }
}

impl<T: NativeClass, Access: ThreadAccess> Instance<T, Access>
where
    RefImplBound: SafeDeref<<T::Base as GodotObject>::RefKind, Access>,
{
    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    #[inline]
    pub fn map<F, U>(&self, op: F) -> Result<U, <T::UserData as Map>::Err>
    where
        T::UserData: Map,
        F: FnOnce(&T, TRef<'_, T::Base, Access>) -> U,
    {
        self.script.map(|script| op(script, self.owner.as_ref()))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    #[inline]
    pub fn map_mut<F, U>(&self, op: F) -> Result<U, <T::UserData as MapMut>::Err>
    where
        T::UserData: MapMut,
        F: FnOnce(&mut T, TRef<'_, T::Base, Access>) -> U,
    {
        self.script
            .map_mut(|script| op(script, self.owner.as_ref()))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    #[inline]
    pub fn map_owned<F, U>(&self, op: F) -> Result<U, <T::UserData as MapOwned>::Err>
    where
        T::UserData: MapOwned,
        F: FnOnce(T, TRef<'_, T::Base, Access>) -> U,
    {
        self.script
            .map_owned(|script| op(script, self.owner.as_ref()))
    }
}

/// Methods for instances with manually-managed base classes.
impl<T: NativeClass> Instance<T, Shared> {
    /// Assume that `self` is safe to use.
    ///
    /// This is *not* guaranteed to be a no-op at runtime.
    ///
    /// # Safety
    ///
    /// It's safe to call `assume_safe` only if the constraints of `Ref::assume_safe`
    /// are satisfied for the base object.
    #[inline]
    pub unsafe fn assume_safe<'a, 'r>(&'r self) -> RefInstance<'a, T, Shared>
    where
        AssumeSafeLifetime<'a, 'r>: LifetimeConstraint<<T::Base as GodotObject>::RefKind>,
    {
        RefInstance {
            owner: self.owner.assume_safe(),
            script: self.script.clone(),
        }
    }
}

impl<T: NativeClass> Instance<T, Shared>
where
    T::Base: GodotObject<RefKind = ManuallyManaged>,
{
    /// Returns `true` if the pointer currently points to a valid object of the correct type.
    /// **This does NOT guarantee that it's safe to use this pointer.**
    ///
    /// # Safety
    ///
    /// This thread must have exclusive access to the object during the call.
    #[inline]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn is_instance_sane(&self) -> bool {
        self.owner.is_instance_sane()
    }
}

impl<T: NativeClass> Instance<T, Unique>
where
    T::Base: GodotObject<RefKind = ManuallyManaged>,
{
    /// Frees the base object and user-data wrapper.
    ///
    /// Same as `self.into_base().free()`.
    #[inline]
    pub fn free(self) {
        self.into_base().free()
    }
}

impl<T: NativeClass> Instance<T, Unique>
where
    T::Base: GodotObject<RefKind = ManuallyManaged> + QueueFree,
{
    /// Queues the base object and user-data wrapper for deallocation in the near future.
    /// This should be preferred to `free` for `Node`s.
    ///
    /// Same as `self.into_base().queue_free()`.
    #[inline]
    pub fn queue_free(self) {
        self.into_base().queue_free()
    }
}

impl<T: NativeClass> Instance<T, Unique> {
    /// Coverts into a `Shared` instance.
    #[inline]
    pub fn into_shared(self) -> Instance<T, Shared> {
        Instance {
            owner: self.owner.into_shared(),
            script: self.script,
        }
    }
}

impl<T: NativeClass> Instance<T, Unique>
where
    T::Base: GodotObject<RefKind = RefCounted>,
{
    /// Coverts into a `ThreadLocal` instance.
    #[inline]
    pub fn into_thread_local(self) -> Instance<T, ThreadLocal> {
        Instance {
            owner: self.owner.into_thread_local(),
            script: self.script,
        }
    }
}

impl<T: NativeClass> Instance<T, Shared> {
    /// Assume that `self` is the unique reference to the underlying base object.
    ///
    /// This is guaranteed to be a no-op at runtime if `debug_assertions` is disabled. Runtime
    /// sanity checks may be added in debug builds to help catch bugs.
    ///
    /// # Safety
    ///
    /// Calling `assume_unique` when `self` isn't the unique reference is instant undefined
    /// behavior. This is a much stronger assumption than `assume_safe` and should be used with
    /// care.
    #[inline]
    pub unsafe fn assume_unique(self) -> Instance<T, Unique> {
        Instance {
            owner: self.owner.assume_unique(),
            script: self.script,
        }
    }
}

impl<T: NativeClass> Instance<T, Shared>
where
    T::Base: GodotObject<RefKind = RefCounted>,
{
    /// Assume that all references to the underlying object is local to the current thread.
    ///
    /// This is guaranteed to be a no-op at runtime.
    ///
    /// # Safety
    ///
    /// Calling `assume_thread_local` when there are references on other threads is instant
    /// undefined behavior. This is a much stronger assumption than `assume_safe` and should
    /// be used with care.
    #[inline]
    pub unsafe fn assume_thread_local(self) -> Instance<T, ThreadLocal> {
        Instance {
            owner: self.owner.assume_thread_local(),
            script: self.script,
        }
    }
}

impl<'a, T: NativeClass, Access: ThreadAccess> RefInstance<'a, T, Access> {
    /// Returns a reference to the base object with the same lifetime.
    #[inline]
    pub fn base(&self) -> TRef<'a, T::Base, Access> {
        self.owner
    }

    /// Returns a reference to the script wrapper.
    #[inline]
    pub fn script(&self) -> &T::UserData {
        &self.script
    }

    /// Try to downcast `TRef<'a, T::Base, Access>` to `RefInstance<T>`.
    #[inline]
    pub fn try_from_base(owner: TRef<'a, T::Base, Access>) -> Option<Self> {
        let user_data = try_get_user_data_ptr::<T>(owner.as_raw())?;
        unsafe { Some(Self::from_raw_unchecked(owner, user_data)) }
    }

    /// Pairs an `owner` and `user_data` without checking validity. Internal interface.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_unchecked(
        owner: TRef<'a, T::Base, Access>,
        user_data: *mut libc::c_void,
    ) -> Self {
        let script = T::UserData::clone_from_user_data_unchecked(user_data);
        RefInstance { owner, script }
    }
}

impl<'a, T: NativeClass, Access: NonUniqueThreadAccess> RefInstance<'a, T, Access> {
    /// Persists this into a persistent `Instance` with the same thread access, without cloning
    /// the userdata wrapper.
    ///
    /// This is only available for non-`Unique` accesses.
    #[inline]
    pub fn claim(self) -> Instance<T, Access> {
        Instance {
            owner: self.owner.claim(),
            script: self.script,
        }
    }
}

/// Methods for instances with reference-counted base classes.
impl<'a, T: NativeClass, Access: ThreadAccess> RefInstance<'a, T, Access>
where
    T::Base: GodotObject,
{
    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value.
    #[inline]
    pub fn map<F, U>(&self, op: F) -> Result<U, <T::UserData as Map>::Err>
    where
        T::UserData: Map,
        F: FnOnce(&T, TRef<'_, T::Base, Access>) -> U,
    {
        self.script.map(|script| op(script, self.owner))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value.
    #[inline]
    pub fn map_mut<F, U>(&self, op: F) -> Result<U, <T::UserData as MapMut>::Err>
    where
        T::UserData: MapMut,
        F: FnOnce(&mut T, TRef<'_, T::Base, Access>) -> U,
    {
        self.script.map_mut(|script| op(script, self.owner))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value.
    #[inline]
    pub fn map_owned<F, U>(&self, op: F) -> Result<U, <T::UserData as MapOwned>::Err>
    where
        T::UserData: MapOwned,
        F: FnOnce(T, TRef<'_, T::Base, Access>) -> U,
    {
        self.script.map_owned(|script| op(script, self.owner))
    }
}

impl<T, Access: ThreadAccess> Clone for Instance<T, Access>
where
    T: NativeClass,
    Ref<T::Base, Access>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Instance {
            owner: self.owner.clone(),
            script: self.script.clone(),
        }
    }
}

impl<'a, T, Access: ThreadAccess> Clone for RefInstance<'a, T, Access>
where
    T: NativeClass,
{
    #[inline]
    fn clone(&self) -> Self {
        RefInstance {
            owner: self.owner,
            script: self.script.clone(),
        }
    }
}

impl<T, Access: ThreadAccess> ToVariant for Instance<T, Access>
where
    T: NativeClass,
    Ref<T::Base, Access>: ToVariant,
{
    #[inline]
    fn to_variant(&self) -> Variant {
        self.owner.to_variant()
    }
}

impl<T> OwnedToVariant for Instance<T, Unique>
where
    T: NativeClass,
{
    #[inline]
    fn owned_to_variant(self) -> Variant {
        self.into_base().owned_to_variant()
    }
}

impl<T> FromVariant for Instance<T, Shared>
where
    T: NativeClass,
    T::Base: GodotObject<RefKind = RefCounted>,
{
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        let owner = Ref::<T::Base, Shared>::from_variant(variant)?;
        Self::from_base(owner).ok_or(FromVariantError::InvalidInstance {
            expected: T::class_name(),
        })
    }
}

fn try_get_user_data_ptr<T: NativeClass>(owner: &RawObject<T::Base>) -> Option<*mut libc::c_void> {
    unsafe {
        let api = get_api();

        let owner_ptr = owner.sys().as_ptr();

        let type_tag = (api.godot_nativescript_get_type_tag)(owner_ptr);
        if type_tag.is_null() {
            return None;
        }

        if !crate::nativescript::type_tag::check::<T>(type_tag) {
            return None;
        }

        Some((api.godot_nativescript_get_userdata)(owner_ptr))
    }
}

mod private {
    pub trait Sealed {}
}
