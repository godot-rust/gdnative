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

use crate::core_types::{GodotString, Variant};
use crate::export::*;
use crate::object::NewRef;
use crate::private::get_api;

#[derive(Debug)]
pub struct ClassBuilder<C> {
    pub(super) init_handle: *mut libc::c_void,
    pub(super) class_name: CString,
    _marker: PhantomData<C>,
}

impl<C: NativeClass> ClassBuilder<C> {
    pub(crate) fn new(init_handle: *mut libc::c_void, class_name: CString) -> Self {
        Self {
            init_handle,
            class_name,
            _marker: PhantomData,
        }
    }

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
    /// use gdnative::export::{RpcMode, Varargs};
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
    ///     fn call(&self, this: TInstance<'_, MyType, Shared>, _args: Varargs<'_>) -> Variant {
    ///         this.map(|obj: &MyType, _| {
    ///             let result = obj.my_method();
    ///             Variant::new(result)
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
