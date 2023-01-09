use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr;

use crate::core_types::{GodotString, VariantType};
use crate::export::*;
use crate::object::NewRef;
use crate::private::get_api;

// TODO(#996): unify string parameters across all buiders
// Potential candidates:
// * &str
// * impl Into<GodotString>
// * impl Into<Cow<'a, str>>

/// Allows registration of exported properties, methods and signals.
///
/// See member functions of this class for usage examples.
#[derive(Debug)]
pub struct ClassBuilder<C> {
    pub(super) init_handle: *mut libc::c_void,
    pub(super) class_name: CString,
    mixins: RefCell<HashSet<TypeId, ahash::RandomState>>,
    _marker: PhantomData<C>,
}

impl<C: NativeClass> ClassBuilder<C> {
    pub(crate) fn new(init_handle: *mut libc::c_void, class_name: CString) -> Self {
        Self {
            init_handle,
            class_name,
            mixins: RefCell::default(),
            _marker: PhantomData,
        }
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
    ///             .method("my_method", MyMethod)
    ///             .with_rpc_mode(RpcMode::RemoteSync)
    ///             .done();
    ///     }
    /// }
    ///
    /// // Now, wrap the method (this can do anything and does not need to actually call a method)
    /// struct MyMethod;
    /// impl Method<MyType> for MyMethod {
    ///     fn call(&self, this: TInstance<'_, MyType>, _args: Varargs<'_>) -> Variant {
    ///         this.map(|obj: &MyType, _| {
    ///             let result = obj.my_method();
    ///             Variant::new(result)
    ///         }).expect("method call succeeds")
    ///     }
    /// }
    /// ```
    ///
    #[inline]
    pub fn method<'a, F: Method<C>>(&'a self, name: &'a str, method: F) -> MethodBuilder<'a, C, F> {
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
    ///             .property("foo")
    ///             .with_default(5)
    ///             .with_hint((-10..=30).into())
    ///             .with_getter(MyType::get_foo)
    ///             .with_setter(MyType::set_foo)
    ///             .done();
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn property<'a, T>(&'a self, name: &'a str) -> PropertyBuilder<'a, C, T>
    where
        T: Export,
    {
        PropertyBuilder::new(self, name)
    }

    /// Returns a `SignalBuilder` which can be used to add a signal to the class being
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
    /// struct MyType {}
    ///
    /// // Note: no #[methods] required
    /// impl MyType {
    ///     fn my_register(builder: &ClassBuilder<MyType>) {
    ///         // Add signal without parameters
    ///         builder
    ///             .signal("jumped")
    ///             .done();
    ///
    ///         // Add another signal with 1 parameter (untyped)
    ///         builder
    ///             .signal("fired")
    ///             .with_param_untyped("weapon_type")
    ///             .done();
    ///
    ///         // Add third signal with int + String parameters, the latter with a default value "Kerosene"
    ///         builder
    ///             .signal("used_jetpack")
    ///             .with_param("fuel_spent", VariantType::I64)
    ///             .with_param_default("fuel_type", Variant::new("Kerosene"))
    ///             .done();
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn signal(&self, name: &str) -> SignalBuilder<C> {
        SignalBuilder::new(self, GodotString::from(name))
    }

    #[inline]
    pub(crate) fn add_signal(&self, signal: Signal) {
        unsafe {
            let args_and_hints = signal
                .args
                .iter()
                .map(|arg| {
                    let hint_string = arg.export_info.hint_string.new_ref();
                    (arg, hint_string)
                })
                .collect::<Vec<_>>();

            let mut sys_args = args_and_hints
                .iter()
                .map(|(param, hint_string)| sys::godot_signal_argument {
                    name: param.name.to_sys(),
                    type_: Self::get_param_type(param) as i32,
                    hint: param.export_info.hint_kind,
                    hint_string: hint_string.to_sys(),
                    usage: param.usage.to_sys(),
                    default_value: param.default.to_sys(),
                })
                .collect::<Vec<_>>();

            (get_api().godot_nativescript_register_signal)(
                self.init_handle,
                self.class_name.as_ptr(),
                &sys::godot_signal {
                    name: signal.name.to_sys(),
                    num_args: sys_args.len() as i32,
                    args: sys_args.as_mut_ptr(),
                    num_default_args: 0,
                    default_args: ptr::null_mut(),
                },
            );
        }
    }

    /// Returns the declared parameter type, or the default value's type, or Nil (in that order)
    fn get_param_type(arg: &SignalParam) -> VariantType {
        let export_type = arg.export_info.variant_type;
        if export_type != VariantType::Nil {
            export_type
        } else {
            arg.default.get_type()
        }
    }

    pub(crate) fn add_method(&self, method: ScriptMethod) {
        let method_name = CString::new(method.name).unwrap();

        let attr = sys::godot_method_attributes {
            rpc_type: method.attributes.rpc_mode.sys(),
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

    /// Add a mixin to the class being registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use gdnative::prelude::*;
    ///
    /// #[derive(NativeClass)]
    /// #[inherit(Node)]
    /// #[register_with(my_register)]
    /// #[no_constructor]
    /// struct MyType {}
    ///
    /// // This creates a opaque type `MyMixin` in the current scope that implements
    /// // the `Mixin` trait. Mixin types have no public interface or stable layout.
    /// #[methods(mixin = "MyMixin")]
    /// impl MyType {
    ///     #[method]
    ///     fn my_method(&self) -> i64 { 42 }
    /// }
    ///
    /// fn my_register(builder: &ClassBuilder<MyType>) {
    ///     builder.mixin::<MyMixin>();
    /// }
    /// ```
    #[inline]
    pub fn mixin<M: Mixin<C>>(&self) {
        if self.mixins.borrow_mut().insert(TypeId::of::<M>()) {
            M::register(self);
        }
    }
}

/// Trait for mixins, manually registered `#[methods]` blocks that may be applied to multiple types.
///
/// This trait is implemented on generated types by the `#[methods]` proc-macro only, and has no public interface.
/// Use [`ClassBuilder::mixin`] to register mixins to [`NativeClass`] types.    
pub trait Mixin<C>: crate::private::mixin::Sealed + 'static
where
    C: NativeClass,
{
    #[doc(hidden)]
    fn register(builder: &ClassBuilder<C>);
}
