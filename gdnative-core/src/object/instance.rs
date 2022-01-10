use std::ptr::NonNull;

use crate::core_types::{
    FromVariant, FromVariantError, GodotString, OwnedToVariant, ToVariant, Variant,
};
use crate::export::user_data::{Map, MapMut, MapOwned, UserData};
use crate::export::{class_registry, emplace, NativeClass};
use crate::object::bounds::{
    AssumeSafeLifetime, LifetimeConstraint, RefImplBound, SafeAsRaw, SafeDeref,
};
use crate::object::memory::{ManuallyManaged, RefCounted};
use crate::object::ownership::{NonUniqueOwnership, Ownership, Shared, ThreadLocal, Unique};
use crate::object::{GodotObject, Instanciable, QueueFree, RawObject, Ref, TRef};
use crate::private::{get_api, ReferenceCountedClassPlaceholder};

/// A persistent reference to a GodotObject with a rust NativeClass attached.
///
/// `Instance`s can be worked on directly using `map` and `map_mut` if the base object is
/// reference-counted. Otherwise, use `assume_safe` to obtain a temporary `TInstance`.
///
/// See the type-level documentation on `Ref` for more information on typed thread accesses.
#[derive(Debug)]
pub struct Instance<T: NativeClass, Own: Ownership = Shared> {
    owner: Ref<T::Base, Own>,
    script: T::UserData,
}

/// A reference to a GodotObject with a rust NativeClass attached that is assumed safe during
/// a certain lifetime.
///
/// See the type-level documentation on `Ref` for more information on typed thread accesses.
#[derive(Debug)]
pub struct TInstance<'a, T: NativeClass, Own: Ownership = Shared> {
    owner: TRef<'a, T::Base, Own>,
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

            // `set_library` should be called before `set_class_name` to trigger class registration
            // before trying to fetch the class name, in case the first NativeScript instance of this
            // library is being constructed from Rust.
            let mut args: [*const libc::c_void; 1] = [crate::private::get_gdnative_library_sys()];
            (gd_api.godot_method_bind_ptrcall)(
                nativescript_methods.set_library,
                native_script.sys().as_ptr(),
                args.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            let script_class_name = class_registry::class_name::<T>()
                .map(GodotString::from)
                .unwrap_or_else(|| {
                    panic!(
                        "`{type_name}` must be registered before it can be used; call `handle.add_class::<{type_name}>()` in your `nativescript_init` callback",
                        type_name = std::any::type_name::<T>(),
                    );
                });

            let mut args: [*const libc::c_void; 1] = [script_class_name.sys() as *const _];
            (gd_api.godot_method_bind_ptrcall)(
                nativescript_methods.set_class_name,
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
                .to_object::<T::Base>()
                .expect("the engine should return a base object of the correct type")
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

impl<T: NativeClass, Own: Ownership> Instance<T, Own> {
    /// Returns the base object, dropping the script wrapper.
    #[inline]
    pub fn into_base(self) -> Ref<T::Base, Own> {
        self.owner
    }

    /// Returns the script wrapper.
    #[inline]
    pub fn into_script(self) -> T::UserData {
        self.script
    }

    /// Returns the base object and the script wrapper.
    #[inline]
    pub fn decouple(self) -> (Ref<T::Base, Own>, T::UserData) {
        (self.owner, self.script)
    }

    /// Returns a reference to the base object.
    #[inline]
    pub fn base(&self) -> &Ref<T::Base, Own> {
        &self.owner
    }

    /// Returns a reference to the script wrapper.
    #[inline]
    pub fn script(&self) -> &T::UserData {
        &self.script
    }

    /// Convert to a nullable raw pointer. Used for AsArg.
    pub(super) fn as_base_ptr(&self) -> *mut sys::godot_object {
        self.owner.as_ptr()
    }
}

impl<T: NativeClass, Own: Ownership> Instance<T, Own>
where
    RefImplBound: SafeAsRaw<<T::Base as GodotObject>::Memory, Own>,
{
    /// Try to downcast `Ref<T::Base, Own>` to `Instance<T>`, without changing the reference
    /// count if reference-counted.
    ///
    /// # Errors
    ///
    /// Returns the original `Ref` if the cast failed.
    #[inline]
    pub fn try_from_base(owner: Ref<T::Base, Own>) -> Result<Self, Ref<T::Base, Own>> {
        let user_data = match try_get_user_data_ptr::<T>(owner.as_raw()) {
            Some(user_data) => user_data,
            None => return Err(owner),
        };

        let script = unsafe { T::UserData::clone_from_user_data_unchecked(user_data) };

        Ok(Instance { owner, script })
    }

    /// Try to downcast `Ref<T::Base, Own>` to `Instance<T>`, without changing the reference
    /// count if reference-counted. Shorthand for `Self::try_from_base().ok()`.
    #[inline]
    pub fn from_base(owner: Ref<T::Base, Own>) -> Option<Self> {
        Self::try_from_base(owner).ok()
    }
}

impl<T: NativeClass, Own: Ownership> Instance<T, Own>
where
    RefImplBound: SafeDeref<<T::Base as GodotObject>::Memory, Own>,
{
    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    #[inline]
    pub fn map<F, U>(&self, op: F) -> Result<U, <T::UserData as Map>::Err>
    where
        T::UserData: Map,
        F: FnOnce(&T, TRef<'_, T::Base, Own>) -> U,
    {
        self.script.map(|script| op(script, self.owner.as_ref()))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    #[inline]
    pub fn map_mut<F, U>(&self, op: F) -> Result<U, <T::UserData as MapMut>::Err>
    where
        T::UserData: MapMut,
        F: FnOnce(&mut T, TRef<'_, T::Base, Own>) -> U,
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
        F: FnOnce(T, TRef<'_, T::Base, Own>) -> U,
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
    pub unsafe fn assume_safe<'a, 'r>(&'r self) -> TInstance<'a, T, Shared>
    where
        AssumeSafeLifetime<'a, 'r>: LifetimeConstraint<<T::Base as GodotObject>::Memory>,
    {
        TInstance {
            owner: self.owner.assume_safe(),
            script: self.script.clone(),
        }
    }
}

impl<T: NativeClass> Instance<T, Shared>
where
    T::Base: GodotObject<Memory = ManuallyManaged>,
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
    T::Base: GodotObject<Memory = ManuallyManaged>,
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
    T::Base: GodotObject<Memory = ManuallyManaged> + QueueFree,
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
    T::Base: GodotObject<Memory = RefCounted>,
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
    T::Base: GodotObject<Memory = RefCounted>,
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

impl<'a, T: NativeClass, Own: Ownership> TInstance<'a, T, Own> {
    /// Returns a reference to the base object with the same lifetime.
    #[inline]
    pub fn base(&self) -> TRef<'a, T::Base, Own> {
        self.owner
    }

    /// Returns a reference to the script wrapper.
    #[inline]
    pub fn script(&self) -> &T::UserData {
        &self.script
    }

    /// Try to downcast `TRef<'a, T::Base, Own>` to `TInstance<T>`.
    #[inline]
    pub fn try_from_base(owner: TRef<'a, T::Base, Own>) -> Option<Self> {
        let user_data = try_get_user_data_ptr::<T>(owner.as_raw())?;
        unsafe { Some(Self::from_raw_unchecked(owner, user_data)) }
    }

    /// Pairs an `owner` and `user_data` without checking validity. Internal interface.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_raw_unchecked(
        owner: TRef<'a, T::Base, Own>,
        user_data: *mut libc::c_void,
    ) -> Self {
        let script = T::UserData::clone_from_user_data_unchecked(user_data);
        TInstance { owner, script }
    }

    /// Convert to a nullable raw pointer. Used for AsArg.
    pub(super) fn as_base_ptr(&self) -> *mut sys::godot_object {
        self.owner.as_ptr()
    }
}

impl<'a, T: NativeClass, Own: NonUniqueOwnership> TInstance<'a, T, Own> {
    /// Persists this into a persistent `Instance` with the same thread access, without cloning
    /// the userdata wrapper.
    ///
    /// This is only available for non-`Unique` accesses.
    #[inline]
    pub fn claim(self) -> Instance<T, Own> {
        Instance {
            owner: self.owner.claim(),
            script: self.script,
        }
    }
}

/// Methods for instances with reference-counted base classes.
impl<'a, T: NativeClass, Own: Ownership> TInstance<'a, T, Own>
where
    T::Base: GodotObject,
{
    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value.
    #[inline]
    pub fn map<F, U>(&self, op: F) -> Result<U, <T::UserData as Map>::Err>
    where
        T::UserData: Map,
        F: FnOnce(&T, TRef<'_, T::Base, Own>) -> U,
    {
        self.script.map(|script| op(script, self.owner))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value.
    #[inline]
    pub fn map_mut<F, U>(&self, op: F) -> Result<U, <T::UserData as MapMut>::Err>
    where
        T::UserData: MapMut,
        F: FnOnce(&mut T, TRef<'_, T::Base, Own>) -> U,
    {
        self.script.map_mut(|script| op(script, self.owner))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value.
    #[inline]
    pub fn map_owned<F, U>(&self, op: F) -> Result<U, <T::UserData as MapOwned>::Err>
    where
        T::UserData: MapOwned,
        F: FnOnce(T, TRef<'_, T::Base, Own>) -> U,
    {
        self.script.map_owned(|script| op(script, self.owner))
    }
}

impl<T, Own: Ownership> Clone for Instance<T, Own>
where
    T: NativeClass,
    Ref<T::Base, Own>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Instance {
            owner: self.owner.clone(),
            script: self.script.clone(),
        }
    }
}

impl<'a, T, Own: Ownership> Clone for TInstance<'a, T, Own>
where
    T: NativeClass,
{
    #[inline]
    fn clone(&self) -> Self {
        TInstance {
            owner: self.owner,
            script: self.script.clone(),
        }
    }
}

impl<T, Own: Ownership> ToVariant for Instance<T, Own>
where
    T: NativeClass,
    Ref<T::Base, Own>: ToVariant,
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
    T::Base: GodotObject<Memory = RefCounted>,
{
    #[inline]
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        let owner = Ref::<T::Base, Shared>::from_variant(variant)?;
        Self::from_base(owner).ok_or(FromVariantError::InvalidInstance {
            expected: class_registry::class_name_or_default::<T>(),
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

        if !crate::export::type_tag::check::<T>(type_tag) {
            return None;
        }

        Some((api.godot_nativescript_get_userdata)(owner_ptr))
    }
}
