use crate::export::user_data::UserData;
use crate::export::{
    class_registry, emplace, ClassBuilder, NativeClass, NativeClassMethods, StaticallyNamed,
};
use crate::object::{GodotObject, RawObject, TRef};
use crate::private::get_api;
use std::borrow::Cow;
use std::ffi::CString;
use std::ptr;

use super::InitLevel;

/// A handle that can register new classes to the engine during initialization.
///
/// See [`godot_nativescript_init`](macro.godot_nativescript_init.html) and
/// [`godot_init`](macro.godot_init.html).
#[derive(Copy, Clone)]
pub struct InitHandle {
    handle: *mut libc::c_void,
    init_level: InitLevel,
}

#[allow(deprecated)] // Remove once init(), register_properties() and register() have been renamed
impl InitHandle {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(handle: *mut libc::c_void, init_level: InitLevel) -> Self {
        InitHandle { handle, init_level }
    }

    /// Registers a new class to the engine.
    #[inline]
    pub fn add_class<C>(self)
    where
        C: NativeClassMethods + StaticallyNamed,
    {
        self.add_class_with::<C>(|_| {})
    }

    /// Registers a new class to the engine.
    #[inline]
    pub fn add_class_with<C>(self, f: impl FnOnce(&ClassBuilder<C>))
    where
        C: NativeClassMethods + StaticallyNamed,
    {
        self.add_maybe_tool_class_as_with::<C>(Cow::Borrowed(C::CLASS_NAME), false, |builder| {
            C::nativeclass_register_monomorphized(builder);
            f(builder);
        })
    }

    /// Registers a new tool class to the engine.
    #[inline]
    pub fn add_tool_class<C>(self)
    where
        C: NativeClassMethods + StaticallyNamed,
    {
        self.add_tool_class_with::<C>(|_| {})
    }

    /// Registers a new tool class to the engine.
    #[inline]
    pub fn add_tool_class_with<C>(self, f: impl FnOnce(&ClassBuilder<C>))
    where
        C: NativeClassMethods + StaticallyNamed,
    {
        self.add_maybe_tool_class_as_with::<C>(Cow::Borrowed(C::CLASS_NAME), true, |builder| {
            C::nativeclass_register_monomorphized(builder);
            f(builder);
        })
    }

    /// Registers a new class to the engine
    ///
    /// If the type implements [`StaticallyTyped`], that name is ignored in favor of the
    /// name provided at registration.
    #[inline]
    pub fn add_class_as<C>(self, name: String)
    where
        C: NativeClassMethods,
    {
        self.add_class_as_with::<C>(name, |_| {})
    }

    /// Registers a new class to the engine
    ///
    /// If the type implements [`StaticallyTyped`], that name is ignored in favor of the
    /// name provided at registration.
    #[inline]
    pub fn add_class_as_with<C>(self, name: String, f: impl FnOnce(&ClassBuilder<C>))
    where
        C: NativeClassMethods,
    {
        self.add_maybe_tool_class_as_with::<C>(Cow::Owned(name), false, f)
    }

    /// Registers a new tool class to the engine
    ///
    /// If the type implements [`StaticallyTyped`], that name is ignored in favor of the
    /// name provided at registration.
    #[inline]
    pub fn add_tool_class_as<C>(self, name: String)
    where
        C: NativeClassMethods,
    {
        self.add_tool_class_as_with::<C>(name, |_| {})
    }

    /// Registers a new tool class to the engine
    ///
    /// If the type implements [`StaticallyTyped`], that name is ignored in favor of the
    /// name provided at registration.
    #[inline]
    pub fn add_tool_class_as_with<C>(self, name: String, f: impl FnOnce(&ClassBuilder<C>))
    where
        C: NativeClassMethods,
    {
        self.add_maybe_tool_class_as_with::<C>(Cow::Owned(name), true, f)
    }

    #[inline]
    fn add_maybe_tool_class_as_with<C>(
        self,
        name: Cow<'static, str>,
        is_tool: bool,
        f: impl FnOnce(&ClassBuilder<C>),
    ) where
        C: NativeClassMethods,
    {
        let c_class_name = CString::new(&*name).unwrap();

        match class_registry::register_class_as::<C>(name, self.init_level) {
            Ok(true) => {}
            Ok(false) => return,
            Err(e) => {
                godot_error!("gdnative-core: ignoring new registration: {e}");
                return;
            }
        };

        unsafe {
            let base_name = CString::new(C::Base::class_name()).unwrap();

            let create = {
                unsafe extern "C" fn constructor<C: NativeClass>(
                    this: *mut sys::godot_object,
                    _method_data: *mut libc::c_void,
                ) -> *mut libc::c_void {
                    use std::panic::{self, AssertUnwindSafe};

                    let this = match ptr::NonNull::new(this) {
                        Some(this) => this,
                        None => {
                            godot_error!(
                                "gdnative-core: error constructing {}: owner pointer is null",
                                class_registry::class_name_or_default::<C>(),
                            );

                            return ptr::null_mut();
                        }
                    };

                    let owner = match RawObject::<C::Base>::try_from_sys_ref(this) {
                        Some(owner) => owner,
                        None => {
                            godot_error!(
                                "gdnative-core: error constructing {}: incompatible owner type, expecting {}",
                                class_registry::class_name_or_default::<C>(),
                                C::Base::class_name(),
                            );
                            return ptr::null_mut();
                        }
                    };

                    let val = match panic::catch_unwind(AssertUnwindSafe(|| {
                        emplace::take().unwrap_or_else(|| {
                            C::nativeclass_init(TRef::new(C::Base::cast_ref(owner)))
                        })
                    })) {
                        Ok(val) => val,
                        Err(e) => {
                            godot_error!(
                                "gdnative-core: error constructing {}: constructor panicked",
                                class_registry::class_name_or_default::<C>(),
                            );
                            crate::private::print_panic_error(e);
                            return ptr::null_mut();
                        }
                    };

                    let wrapper = C::UserData::new(val);
                    C::UserData::into_user_data(wrapper) as *mut _
                }

                sys::godot_instance_create_func {
                    create_func: Some(constructor::<C>),
                    method_data: ptr::null_mut(),
                    free_func: None,
                }
            };

            let destroy = {
                unsafe extern "C" fn destructor<C: NativeClass>(
                    _this: *mut sys::godot_object,
                    _method_data: *mut libc::c_void,
                    user_data: *mut libc::c_void,
                ) {
                    if user_data.is_null() {
                        godot_error!(
                            "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
                            class_registry::class_name_or_default::<C>(),
                        );
                        return;
                    }

                    let wrapper = C::UserData::consume_user_data_unchecked(user_data);
                    drop(wrapper)
                }

                sys::godot_instance_destroy_func {
                    destroy_func: Some(destructor::<C>),
                    method_data: ptr::null_mut(),
                    free_func: None,
                }
            };

            if is_tool {
                (get_api().godot_nativescript_register_tool_class)(
                    self.handle as *mut _,
                    c_class_name.as_ptr() as *const _,
                    base_name.as_ptr() as *const _,
                    create,
                    destroy,
                );
            } else {
                (get_api().godot_nativescript_register_class)(
                    self.handle as *mut _,
                    c_class_name.as_ptr() as *const _,
                    base_name.as_ptr() as *const _,
                    create,
                    destroy,
                );
            }

            (get_api().godot_nativescript_set_type_tag)(
                self.handle as *mut _,
                c_class_name.as_ptr() as *const _,
                crate::export::type_tag::create::<C>(),
            );

            let builder = ClassBuilder::new(self.handle, c_class_name);

            C::nativeclass_register_properties(&builder);

            // register methods
            C::nativeclass_register(&builder);

            f(&builder);
        }
    }
}
