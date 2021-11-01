//! Low-level API to register and export GDNative classes, methods and properties.
//!
//! ## Init and exit hooks
//!
//! Three endpoints are automatically invoked by the engine during startup and shutdown:
//!
//! - [`godot_gdnative_init`],
//! - [`godot_nativescript_init`],
//! - [`godot_gdnative_terminate`],
//!
//! All three must be present. To quickly define all three endpoints using the default names,
//! use [`godot_init`].
//!
//! ## Registering script classes
//!
//! [`InitHandle`] is the registry of all your exported symbols.
//! To register script classes, call [`InitHandle::add_class`] or [`InitHandle::add_tool_class`]
//! in your [`godot_nativescript_init`] or [`godot_init`] callback:
//!
//! ```ignore
//! use gdnative::prelude::*;
//!
//! fn init(handle: InitHandle) {
//!     handle.add_class::<HelloWorld>();
//! }
//!
//! godot_init!(init);
//! ```
//!
//! For full examples, see [`examples`](https://github.com/godot-rust/godot-rust/tree/master/examples)
//! in the godot-rust repository.

// Temporary for unsafe method registration
#![allow(deprecated)]

use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

use crate::core_types::{GodotString, ToVariant, Variant};
use crate::nativescript::{class_registry, emplace};
use crate::nativescript::{user_data::UserData, NativeClass, NativeClassMethods};
use crate::object::{GodotObject, NewRef, RawObject, TRef};
use crate::private::get_api;

pub use method::*;
pub use property::*;

mod method;
mod property;

//pub use self::method::{
//    Method, MethodBuilder, RpcMode, ScriptMethod, ScriptMethodAttributes, ScriptMethodFn, Varargs,
//};
//pub use self::property::{ExportInfo, PropertyBuilder, Usage as PropertyUsage};

/// Trait for exportable types.
pub trait Export: ToVariant {
    /// A type-specific hint type that is valid for the type being exported.
    ///
    /// If this type shows up as `NoHint`, a private, uninhabitable type indicating
    /// that there are no hints available for the time being, users *must* use `None`
    /// for properties of this type. This ensures that it will not be a breaking change
    /// to add a hint for the type later, since it supports no operations and cannot
    /// be named directly in user code.
    type Hint;

    /// Returns `ExportInfo` given an optional typed hint.
    fn export_info(hint: Option<Self::Hint>) -> ExportInfo;
}

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
                crate::nativescript::type_tag::create::<C>(),
            );

            let builder = ClassBuilder {
                init_handle: self.handle,
                class_name,
                _marker: PhantomData,
            };

            C::register_properties(&builder);

            // register methods
            C::register(&builder);
        }
    }
}

#[derive(Debug)]
pub struct ClassBuilder<C> {
    init_handle: *mut libc::c_void,
    class_name: CString,
    _marker: PhantomData<C>,
}

impl<C: NativeClass> ClassBuilder<C> {
    #[inline]
    #[deprecated(note = "Unsafe registration is deprecated. Use `build_method` instead.")]
    pub fn add_method_advanced(&self, method: ScriptMethod) {
        let method_name = CString::new(method.name).unwrap();

        let rpc = match method.attributes.rpc_mode {
            RpcMode::Master => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_MASTER,
            RpcMode::Remote => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_REMOTE,
            RpcMode::Puppet => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_PUPPET,
            RpcMode::RemoteSync => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_REMOTESYNC,
            RpcMode::Disabled => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_DISABLED,
            RpcMode::MasterSync => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_MASTERSYNC,
            RpcMode::PuppetSync => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_PUPPETSYNC,
        };

        let attr = sys::godot_method_attributes { rpc_type: rpc };

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

    #[inline]
    #[deprecated(note = "Unsafe registration is deprecated. Use `build_method` instead.")]
    pub fn add_method_with_rpc_mode(&self, name: &str, method: ScriptMethodFn, rpc_mode: RpcMode) {
        self.add_method_advanced(ScriptMethod {
            name,
            method_ptr: Some(method),
            attributes: ScriptMethodAttributes { rpc_mode },
            method_data: ptr::null_mut(),
            free_func: None,
        });
    }

    #[inline]
    #[deprecated(note = "Unsafe registration is deprecated. Use `build_method` instead.")]
    pub fn add_method(&self, name: &str, method: ScriptMethodFn) {
        self.add_method_with_rpc_mode(name, method, RpcMode::Disabled);
    }

    /// Returns a `MethodBuilder` which can be used to add a method to the class being
    /// registered.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use gdnative::prelude::*;
    /// use gdnative::nativescript::export::{RpcMode, Varargs};
    ///
    /// #[derive(NativeClass)]
    /// #[register_with(Self::my_register)]
    /// #[no_constructor]
    /// struct MyType {}
    ///
    /// // Note: no #[methods] required
    /// impl MyType {
    ///     fn my_method(&self) -> i64 { 42 }
    ///
    ///     fn my_register(builder: &ClassBuilder<MyType>) {
    ///         builder
    ///             .build_method("my_method", MyMethod)
    ///             .with_rpc_mode(RpcMode::RemoteSync)
    ///             .done();
    ///     }
    /// }
    ///
    /// // Now, wrap the method (this can do anything and does not need to actually call a method)
    /// struct MyMethod;
    /// impl Method<MyType> for MyMethod {
    ///     fn call(&self, this: RefInstance<'_, MyType, Shared>, _args: Varargs<'_>) -> Variant {
    ///         this.map(|obj: &MyType, _| {
    ///             let result = obj.my_method();
    ///             Variant::from_i64(result)
    ///         }).expect("method call succeeds")
    ///     }
    /// }
    /// ```
    ///
    #[inline]
    pub fn build_method<'a, F: Method<C>>(
        &'a self,
        name: &'a str,
        method: F,
    ) -> MethodBuilder<'a, C, F> {
        MethodBuilder::new(self, name, method)
    }

    /// Returns a `PropertyBuilder` which can be used to add a property to the class being
    /// registered.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use gdnative::prelude::*;
    ///
    /// #[derive(NativeClass)]
    /// #[inherit(Node)]
    /// #[register_with(Self::my_register)]
    /// #[no_constructor]
    /// struct MyType {
    ///     foo: i32,
    /// }
    ///
    /// // Note: no #[methods] required
    /// impl MyType {
    ///     pub fn get_foo(&self, _owner: TRef<Node>) -> i32 { self.foo }
    ///     pub fn set_foo(&mut self, _owner: TRef<Node>, val: i32) { self.foo = val; }
    ///
    ///     fn my_register(builder: &ClassBuilder<MyType>) {
    ///         builder
    ///             .add_property("foo")
    ///             .with_default(5)
    ///             .with_hint((-10..=30).into())
    ///             .with_getter(MyType::get_foo)
    ///             .with_setter(MyType::set_foo)
    ///             .done();
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn add_property<'a, T>(&'a self, name: &'a str) -> PropertyBuilder<'a, C, T>
    where
        T: Export,
    {
        PropertyBuilder::new(self, name)
    }

    #[inline]
    pub fn add_signal(&self, signal: Signal) {
        unsafe {
            let name = GodotString::from_str(signal.name);
            let owned = signal
                .args
                .iter()
                .map(|arg| {
                    let arg_name = GodotString::from_str(arg.name);
                    let hint_string = arg.export_info.hint_string.new_ref();
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
