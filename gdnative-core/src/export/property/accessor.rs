//! Types and traits for property accessors.
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::core_types::{FromVariant, ToVariant, Variant};
use crate::export::user_data::{Map, MapMut, UserData};
use crate::export::{class_registry, NativeClass};
use crate::object::{GodotObject, RawObject, TRef};

/// Trait for raw property setters.
///
/// This is an internal interface. User code should not use this directly.
pub unsafe trait RawSetter<C, T> {
    #[doc(hidden)]
    unsafe fn into_godot_function(self) -> sys::godot_property_set_func;
}

/// Trait for raw property getters.
///
/// This is an internal interface. User code should not use this directly.
pub unsafe trait RawGetter<C, T> {
    #[doc(hidden)]
    unsafe fn into_godot_function(self) -> sys::godot_property_get_func;
}

#[derive(Debug)]
pub struct Setter<SelfArg, F> {
    func: F,
    _self_arg: PhantomData<SelfArg>,
}

impl<SelfArg, F> Setter<SelfArg, F> {
    #[inline]
    pub fn new(func: F) -> Self {
        Setter {
            func,
            _self_arg: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Getter<SelfArg, RetKind, F> {
    func: F,
    _self_arg: PhantomData<SelfArg>,
    _ret_kind: PhantomData<RetKind>,
}

impl<SelfArg, RetKind, F> Getter<SelfArg, RetKind, F> {
    #[inline]
    pub fn new(func: F) -> Self {
        Getter {
            func,
            _self_arg: PhantomData,
            _ret_kind: PhantomData,
        }
    }
}

/// Marker type for accessors that take `&self` as their first arguments.
pub struct Shr;
/// Marker type for accessors that take `&mut self` as their first arguments.
pub struct Mut;

/// Helper trait for setters, generic over `self` argument mutability.
pub trait MapSet<C: NativeClass, F, T> {
    type Err: Debug;
    fn map_set(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
        value: T,
    ) -> Result<(), Self::Err>;
}

impl<C, F, T> MapSet<C, F, T> for Shr
where
    C: NativeClass,
    C::UserData: Map,
    F: 'static + Fn(&C, TRef<C::Base>, T),
{
    type Err = <C::UserData as Map>::Err;
    #[inline]
    fn map_set(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
        value: T,
    ) -> Result<(), Self::Err> {
        user_data.map(|rust_ty| op(rust_ty, owner, value))
    }
}

impl<C, F, T> MapSet<C, F, T> for Mut
where
    C: NativeClass,
    C::UserData: MapMut,
    F: 'static + Fn(&mut C, TRef<C::Base>, T),
{
    type Err = <C::UserData as MapMut>::Err;
    #[inline]
    fn map_set(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
        value: T,
    ) -> Result<(), Self::Err> {
        user_data.map_mut(|rust_ty| op(rust_ty, owner, value))
    }
}

/// Marker type for getters that return owned values.
pub struct Owned;
/// Marker type for getters that return references.
pub struct Ref;

/// Helper trait for setters, generic over `self` argument mutability and return kind.
pub trait MapGet<C: NativeClass, F, T> {
    type Err: Debug;
    fn map_get(user_data: &C::UserData, op: &F, owner: TRef<C::Base>)
        -> Result<Variant, Self::Err>;
}

impl<C, F, T> MapGet<C, F, T> for (Shr, Owned)
where
    C: NativeClass,
    C::UserData: Map,
    T: ToVariant,
    F: 'static + Fn(&C, TRef<C::Base>) -> T,
{
    type Err = <C::UserData as Map>::Err;
    #[inline]
    fn map_get(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
    ) -> Result<Variant, Self::Err> {
        user_data.map(|rust_ty| op(rust_ty, owner).to_variant())
    }
}

impl<C, F, T> MapGet<C, F, T> for (Shr, Ref)
where
    C: NativeClass,
    C::UserData: Map,
    T: ToVariant,
    F: 'static + for<'r> Fn(&'r C, TRef<C::Base>) -> &'r T,
{
    type Err = <C::UserData as Map>::Err;
    #[inline]
    fn map_get(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
    ) -> Result<Variant, Self::Err> {
        user_data.map(|rust_ty| op(rust_ty, owner).to_variant())
    }
}

impl<C, F, T> MapGet<C, F, T> for (Mut, Owned)
where
    C: NativeClass,
    C::UserData: MapMut,
    T: ToVariant,
    F: 'static + Fn(&mut C, TRef<C::Base>) -> T,
{
    type Err = <C::UserData as MapMut>::Err;
    #[inline]
    fn map_get(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
    ) -> Result<Variant, Self::Err> {
        user_data.map_mut(|rust_ty| op(rust_ty, owner).to_variant())
    }
}

impl<C, F, T> MapGet<C, F, T> for (Mut, Ref)
where
    C: NativeClass,
    C::UserData: MapMut,
    T: ToVariant,
    F: 'static + for<'r> Fn(&'r mut C, TRef<C::Base>) -> &'r T,
{
    type Err = <C::UserData as MapMut>::Err;
    #[inline]
    fn map_get(
        user_data: &C::UserData,
        op: &F,
        owner: TRef<C::Base>,
    ) -> Result<Variant, Self::Err> {
        user_data.map_mut(|rust_ty| op(rust_ty, owner).to_variant())
    }
}

unsafe impl<SelfArg, F, C, T> RawSetter<C, T> for Setter<SelfArg, F>
where
    C: NativeClass,
    T: FromVariant,
    SelfArg: MapSet<C, F, T>,
{
    #[inline]
    unsafe fn into_godot_function(self) -> sys::godot_property_set_func {
        let mut set = sys::godot_property_set_func::default();
        let data = Box::new(self.func);
        set.method_data = Box::into_raw(data) as *mut _;

        extern "C" fn invoke<SelfArg, C, F, T>(
            this: *mut sys::godot_object,
            method: *mut libc::c_void,
            class: *mut libc::c_void,
            val: *mut sys::godot_variant,
        ) where
            C: NativeClass,
            T: FromVariant,
            SelfArg: MapSet<C, F, T>,
        {
            if class.is_null() {
                godot_error!(
                    "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
                    class_registry::class_name_or_default::<C>(),
                );
                return;
            }

            let this = match NonNull::new(this) {
                Some(this) => this,
                None => {
                    godot_error!(
                        "gdnative-core: owner pointer for {} is null",
                        class_registry::class_name_or_default::<C>(),
                    );
                    return;
                }
            };

            let result = std::panic::catch_unwind(|| unsafe {
                let user_data = C::UserData::clone_from_user_data_unchecked(class as *const _);
                let owner = TRef::new(C::Base::cast_ref(RawObject::from_sys_ref_unchecked(this)));
                let func = &*(method as *const F);

                match T::from_variant(Variant::cast_ref(val)) {
                    Ok(val) => {
                        if let Err(err) = SelfArg::map_set(&user_data, func, owner, val) {
                            godot_error!("gdnative-core: cannot call property setter: {:?}", err);
                        }
                    }
                    Err(err) => {
                        godot_error!("Incorrect type passed to property: {}", err);
                    }
                }
            });

            result.unwrap_or_else(|e| {
                godot_error!("gdnative-core: property setter panicked (check stderr for output)");
                crate::private::print_panic_error(e);
            })
        }
        set.set_func = Some(invoke::<SelfArg, C, F, T>);

        extern "C" fn free_func<F>(data: *mut libc::c_void) {
            unsafe {
                drop(Box::from_raw(data as *mut F));
            }
        }
        set.free_func = Some(free_func::<F>);

        set
    }
}

unsafe impl<SelfArg, RetKind, F, C, T> RawGetter<C, T> for Getter<SelfArg, RetKind, F>
where
    C: NativeClass,
    T: ToVariant,
    (SelfArg, RetKind): MapGet<C, F, T>,
{
    #[inline]
    unsafe fn into_godot_function(self) -> sys::godot_property_get_func {
        let mut get = sys::godot_property_get_func::default();
        let data = Box::new(self.func);
        get.method_data = Box::into_raw(data) as *mut _;

        extern "C" fn invoke<SelfArg, RetKind, C, F, T>(
            this: *mut sys::godot_object,
            method: *mut libc::c_void,
            class: *mut libc::c_void,
        ) -> sys::godot_variant
        where
            C: NativeClass,
            T: ToVariant,
            (SelfArg, RetKind): MapGet<C, F, T>,
        {
            if class.is_null() {
                godot_error!(
                    "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
                    class_registry::class_name_or_default::<C>(),
                );
                return Variant::nil().leak();
            }

            let this = match NonNull::new(this) {
                Some(this) => this,
                None => {
                    godot_error!(
                        "gdnative-core: owner pointer for {} is null",
                        class_registry::class_name_or_default::<C>(),
                    );
                    return Variant::nil().leak();
                }
            };

            let result = std::panic::catch_unwind(|| unsafe {
                let user_data = C::UserData::clone_from_user_data_unchecked(class as *const _);
                let owner = TRef::new(C::Base::cast_ref(RawObject::from_sys_ref_unchecked(this)));
                let func = &*(method as *const F);

                match <(SelfArg, RetKind)>::map_get(&user_data, func, owner) {
                    Ok(variant) => variant.leak(),
                    Err(err) => {
                        godot_error!("gdnative-core: cannot call property getter: {:?}", err);
                        Variant::nil().leak()
                    }
                }
            });

            result.unwrap_or_else(|e| {
                godot_error!("gdnative-core: property getter panicked (check stderr for output)");
                crate::private::print_panic_error(e);
                Variant::nil().leak()
            })
        }
        get.get_func = Some(invoke::<SelfArg, RetKind, C, F, T>);

        extern "C" fn free_func<F>(data: *mut libc::c_void) {
            unsafe {
                drop(Box::from_raw(data as *mut F));
            }
        }
        get.free_func = Some(free_func::<F>);

        get
    }
}
