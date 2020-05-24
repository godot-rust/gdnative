//! Types and functionalities to declare and initialize gdnative classes.
//!
//! ## API endpoints
//!
//! Three endpoints are automatically invoked by the engine during startup and shutdown:
//!
//! - [`godot_gdnative_init`](macro.godot_gdnative_init.html),
//! - [`godot_nativescript_init`](macro.godot_nativescript_init.html),
//! - [`godot_gdnative_terminate`](macro.godot_gdnative_terminate.html),
//!
//! All three must be present.
//!
//! ## Registering a class using the `godot_class` macro
//!
//! See the [spinning_cube example](https://github.com/GodotNativeTools/godot-rust/tree/master/examples/spinning_cube)
//! in the repositiory.
//!
//! ## Registering a class manually
//!
//! See the [manually_registered example](https://github.com/GodotNativeTools/godot-rust/tree/master/examples/manually_registered)
//! in the repositiory.
//!

use super::*;

use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

use libc;

use crate::private::get_api;
use crate::NativeClass;

use crate::Variant;

pub mod property;

pub use self::property::{Export, ExportInfo, PropertyBuilder, Usage as PropertyUsage};

/// A handle that can register new classes to the engine during initialization.
///
/// See [`godot_nativescript_init`](macro.godot_nativescript_init.html).
#[derive(Copy, Clone)]
pub struct InitHandle {
    #[doc(hidden)]
    handle: *mut libc::c_void,
}

impl InitHandle {
    #[doc(hidden)]
    pub unsafe fn new(handle: *mut libc::c_void) -> Self {
        InitHandle { handle }
    }

    /// Registers a new class to the engine.
    pub fn add_class<C>(&self)
    where
        C: NativeClassMethods,
    {
        self.add_maybe_tool_class::<C>(false)
    }

    /// Registers a new tool class to the engine.
    pub fn add_tool_class<C>(&self)
    where
        C: NativeClassMethods,
    {
        self.add_maybe_tool_class::<C>(true)
    }

    fn add_maybe_tool_class<C>(&self, is_tool: bool)
    where
        C: NativeClassMethods,
    {
        unsafe {
            let class_name = CString::new(C::class_name()).unwrap();
            let base_name = CString::new(C::Base::class_name()).unwrap();

            let create = {
                unsafe extern "C" fn constructor<C: NativeClass>(
                    this: *mut sys::godot_object,
                    _method_data: *mut libc::c_void,
                ) -> *mut libc::c_void {
                    use std::panic::{self, AssertUnwindSafe};

                    let owner = match crate::object::godot_cast::<C::Base>(this) {
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

                    let val = match panic::catch_unwind(AssertUnwindSafe(|| C::init(owner))) {
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
                ) -> () {
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
                crate::type_tag::create::<C>(),
            );

            let mut builder = ClassBuilder {
                init_handle: self.handle,
                class_name,
                _marker: PhantomData,
            };

            C::register_properties(&mut builder);

            // register methods
            C::register(&mut builder);
        }
    }
}

pub type ScriptMethodFn = unsafe extern "C" fn(
    *mut sys::godot_object,
    *mut libc::c_void,
    *mut libc::c_void,
    libc::c_int,
    *mut *mut sys::godot_variant,
) -> sys::godot_variant;

pub type ScriptConstructorFn =
    unsafe extern "C" fn(*mut sys::godot_object, *mut libc::c_void) -> *mut libc::c_void;

pub type ScriptDestructorFn =
    unsafe extern "C" fn(*mut sys::godot_object, *mut libc::c_void, *mut libc::c_void) -> ();

pub enum RpcMode {
    Disabled,
    Remote,
    Sync,
    Mater,
    Slave,
}

pub struct ScriptMethodAttributes {
    pub rpc_mode: RpcMode,
}

pub struct ScriptMethod<'l> {
    pub name: &'l str,
    pub method_ptr: Option<ScriptMethodFn>,
    pub attributes: ScriptMethodAttributes,

    pub method_data: *mut libc::c_void,
    pub free_func: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
}

pub struct ClassDescriptor<'l> {
    pub name: &'l str,
    pub base_class: &'l str,
    pub constructor: Option<ScriptConstructorFn>,
    pub destructor: Option<ScriptDestructorFn>,
}

#[derive(Debug)]
pub struct ClassBuilder<C> {
    #[doc(hidden)]
    pub init_handle: *mut libc::c_void,
    class_name: CString,
    _marker: PhantomData<C>,
}

impl<C: NativeClass> ClassBuilder<C> {
    pub fn add_method_advanced(&self, method: ScriptMethod) {
        let method_name = CString::new(method.name).unwrap();
        let attr = sys::godot_method_attributes {
            rpc_type: sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_DISABLED,
        };

        let method_desc = sys::godot_instance_method {
            method: method.method_ptr,
            method_data: method.method_data,
            free_func: method.free_func,
        };

        unsafe {
            (get_api().godot_nativescript_register_method)(
                self.init_handle,
                self.class_name.as_ptr() as *const _,
                method_name.as_ptr() as *const _,
                attr,
                method_desc,
            );
        }
    }

    pub fn add_method(&self, name: &str, method: ScriptMethodFn) {
        self.add_method_advanced(ScriptMethod {
            name: name,
            method_ptr: Some(method),
            attributes: ScriptMethodAttributes {
                rpc_mode: RpcMode::Disabled,
            },
            method_data: ptr::null_mut(),
            free_func: None,
        });
    }

    /// Returns a `PropertyBuilder` which can be used to add a property to the class being
    /// registered.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```ignore
    /// builder
    ///     .add_property("foo")
    ///     .default(0.0)
    ///     .with_hint((-10.0..=30.0).into())
    ///     .with_getter(MyType::get_foo)
    ///     .with_setter(MyType::set_foo)
    ///     .done();
    /// ```
    pub fn add_property<'a, T>(&'a self, name: &'a str) -> PropertyBuilder<'a, C, T>
    where
        T: Export,
    {
        PropertyBuilder::new(self, name)
    }

    pub fn add_signal(&self, signal: Signal) {
        unsafe {
            let name = GodotString::from_str(signal.name);
            let owned = signal
                .args
                .iter()
                .map(|arg| {
                    let arg_name = GodotString::from_str(arg.name);
                    let hint_string = arg.export_info.hint_string.clone();
                    (arg, arg_name, hint_string)
                })
                .collect::<Vec<_>>();
            let mut args = owned
                .iter()
                .map(|(arg, arg_name, hint_string)| sys::godot_signal_argument {
                    name: arg_name.to_sys(),
                    type_: arg.default.get_type() as i32,
                    hint: arg.export_info.hint_kind,
                    hint_string: hint_string.to_sys(),
                    usage: arg.usage.to_sys(),
                    default_value: arg.default.to_sys(),
                })
                .collect::<Vec<_>>();
            (get_api().godot_nativescript_register_signal)(
                self.init_handle,
                self.class_name.as_ptr(),
                &sys::godot_signal {
                    name: name.to_sys(),
                    num_args: args.len() as i32,
                    args: args.as_mut_ptr(),
                    num_default_args: 0,
                    default_args: ptr::null_mut(),
                },
            );
        }
    }
}

pub struct Signal<'l> {
    pub name: &'l str,
    pub args: &'l [SignalArgument<'l>],
}

pub struct SignalArgument<'l> {
    pub name: &'l str,
    pub default: Variant,
    pub export_info: ExportInfo,
    pub usage: PropertyUsage,
}
