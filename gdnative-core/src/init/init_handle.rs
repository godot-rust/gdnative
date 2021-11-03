use crate::export::user_data::UserData;
use crate::export::{class_registry, emplace, ClassBuilder, NativeClass, NativeClassMethods};
use crate::object::{GodotObject, RawObject, TRef};
use crate::private::get_api;
use std::ffi::CString;
use std::ptr;

/// A handle that can register new classes to the engine during initialization.
///
/// See [`godot_nativescript_init`](macro.godot_nativescript_init.html) and
/// [`godot_init`](macro.godot_init.html).
#[derive(Copy, Clone)]
pub struct InitHandle {
    #[doc(hidden)]
    handle: *mut libc::c_void,
}

impl InitHandle {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(handle: *mut libc::c_void) -> Self {
        InitHandle { handle }
    }

    /// Registers a new class to the engine.
    #[inline]
    pub fn add_class<C>(self)
    where
        C: NativeClassMethods,
    {
        self.add_maybe_tool_class::<C>(false)
    }

    /// Registers a new tool class to the engine.
    #[inline]
    pub fn add_tool_class<C>(self)
    where
        C: NativeClassMethods,
    {
        self.add_maybe_tool_class::<C>(true)
    }

    #[inline]
    fn add_maybe_tool_class<C>(self, is_tool: bool)
    where
        C: NativeClassMethods,
    {
        if !class_registry::register_class::<C>() {
            panic!(
                "`{type_name}` has already been registered",
                type_name = std::any::type_name::<C>()
            );
        }
        unsafe {
            let class_name = CString::new(C::class_name()).unwrap();
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
                                C::class_name(),
                            );

                            return ptr::null_mut();
                        }
                    };

                    let owner = match RawObject::<C::Base>::try_from_sys_ref(this) {
                        Some(owner) => owner,
                        None => {
                            godot_error!(
                                "gdnative-core: error constructing {}: incompatible owner type, expecting {}",
                                C::class_name(),
                                C::Base::class_name(),
                            );
                            return ptr::null_mut();
                        }
                    };

                    let val = match panic::catch_unwind(AssertUnwindSafe(|| {
                        emplace::take()
                            .unwrap_or_else(|| C::init(TRef::new(C::Base::cast_ref(owner))))
                    })) {
                        Ok(val) => val,
                        Err(_) => {
                            godot_error!(
                                "gdnative-core: error constructing {}: constructor panicked",
                                C::class_name(),
                            );
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
                            C::class_name(),
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
                    class_name.as_ptr() as *const _,
                    base_name.as_ptr() as *const _,
                    create,
                    destroy,
                );
            } else {
                (get_api().godot_nativescript_register_class)(
                    self.handle as *mut _,
                    class_name.as_ptr() as *const _,
                    base_name.as_ptr() as *const _,
                    create,
                    destroy,
                );
            }

            (get_api().godot_nativescript_set_type_tag)(
                self.handle as *mut _,
                class_name.as_ptr() as *const _,
                crate::export::type_tag::create::<C>(),
            );

            let builder = ClassBuilder::new(self.handle, class_name);

            C::register_properties(&builder);

            // register methods
            C::register(&builder);
        }
    }
}
