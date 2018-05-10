use libc;
use sys;
use std::ffi::CString;
use std::ptr;
use std::ops::Deref;
use GodotString;
use GodotObject;
use GodotType;
use Variant;
use Object;
use NativeScript;
use object;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use get_api;
use property::*;

/// Godot native class implementation detail that must be stored
/// in each instance.
pub struct NativeInstanceHeader {
    #[doc(hidden)]
    pub this: *mut sys::godot_object,
}

pub trait NativeClass {
    fn class_name() -> &'static str;

    fn get_header(&self) -> &NativeInstanceHeader;

    fn as_object(&self) -> &Object {
        unsafe {
            mem::transmute(self.get_header())
        }
    }
}

/// A reference to a rust native script.
pub struct NativeRef<T: NativeClass> {
    this: *mut sys::godot_object,
    _marker: PhantomData<T>,
}

impl<T: NativeClass> NativeRef<T> {
    /// Try to down-cast from a `NativeScript` reference.
    pub fn from_native_script(script: &NativeScript) -> Option<Self> {
        // TODO: There's gotta be a better way.
        let class = script.get_class_name();
        let gd_name = GodotString::from_str(T::class_name());

        if class != gd_name {
            return None;
        }

        unsafe {
            let this = script.to_sys();
            object::add_ref(this);

            return Some(NativeRef { this, _marker: PhantomData, });
        }
    }

    /// Try to down-cast from an `Object` reference.
    pub fn from_object(&self, obj: &Object) -> Option<Self> {
        if let Some(script) = obj.get_script().and_then(|v| v.cast::<NativeScript>()) {
            return Self::from_native_script(&script)
        }

        None
    }

    /// Up-cast to a `NativeScript` reference.
    pub fn to_native_script(&self) -> NativeScript {
        unsafe {
            NativeScript::from_sys(self.this)
        }
    }

    /// Try to cast into a godot object reference.
    pub fn cast<O>(&self) -> Option<O> where O: GodotObject {
        object::godot_cast::<O>(self.this)
    }

    /// Creates a new reference to the same object.
    pub fn new_ref(&self) -> Self {
        unsafe {
            object::add_ref(self.this);

            Self {
                this: self.this,
                _marker: PhantomData,
            }
        }
    }

    fn get_impl(&self) -> &RefCell<T> {
        unsafe {
            let api = get_api();
            let ud = (api.godot_nativescript_get_userdata)(self.this);
            &*(ud as *const _ as *const RefCell<T>)
        }
    }
}

impl<T: NativeClass> Deref for NativeRef<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        self.get_impl()
    }
}

impl <T: NativeClass> Drop for NativeRef<T> {
    fn drop(&mut self) {
        unsafe {
            if object::unref(self.this) {
                (get_api().godot_object_destroy)(self.this);
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! godot_class_build_export_methods {
    ($classty:ty, $builder:ident,) => ();

    ($classty:ty, $builder:ident,
        export fn $name:ident(
            &mut self
            $(,$pname:ident : $pty:ty)*
        ) $body:block
        $($tt:tt)*
    ) => (
        godot_class_build_export_methods!($classty, $builder,
            export fn $name(&mut self $(,$pname : $pty)*) -> () $body
            $($tt)*
        );
    );

    ($classty:ty, $builder:ident,
        export fn $name:ident(
            &mut self
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty $body:block
        $($tt:tt)*
    ) => (
        $builder.add_method(
            stringify!($name),
            godot_wrap_method!(
                $classty,
                fn $name(&mut self $(,$pname : $pty)* ) -> $retty
            ),
        );
        godot_class_build_export_methods!($classty, $builder, $($tt)*);
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! godot_class_build_methods {
    () => ();
    (
        export fn $name:ident(
            &mut $self:ident
            $(,$pname:ident : $pty:ty)*
        ) $body:block
        $($tt:tt)*
    ) => (
        godot_class_build_methods!(
            export fn $name(&mut $self$(,$pname : $pty)*) -> () $body
            $($tt)*
        );
    );
    (
        export fn $name:ident(
            &mut $self:ident
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty $body:block
        $($tt:tt)*
    ) => (
        pub fn $name(&mut $self$(
            ,$pname : $pty
        )*) -> $retty $body
        godot_class_build_methods!($($tt)*);
    )
}

#[macro_export]
macro_rules! godot_class {
    (
class $name:ident: $parent:ty {
    fields {
        $(
            $(#[$fattr:meta])*
            $fname:ident : $fty:ty,
        )*
    }
    setup($builder:ident) $pbody:block
    constructor($header:ident) $construct:block

    $($tt:tt)*
}
    ) => (
        pub struct $name {
            header: $crate::NativeInstanceHeader,
            $(
                $(#[$fattr])*
                pub $fname: $fty,
            )*
        }

        impl $name {
            godot_class_build_methods!($($tt)*);

            pub fn as_parent(&self) -> $parent {
                unsafe {
                    <$parent as $crate::GodotObject>::from_sys(self.header.this)
                }
            }

            pub unsafe fn register_class(init_handle: $crate::InitHandle) {
                use $crate::sys;

                fn constructor($header : $crate::NativeInstanceHeader) -> $name {
                    $construct
                }

                extern "C" fn godot_create(this: *mut sys::godot_object, _data: *mut $crate::libc::c_void) -> *mut $crate::libc::c_void {
                    use std::cell::RefCell;

                    let val = constructor($crate::NativeInstanceHeader {
                        this: this,
                    });
                    let wrapper = Box::new(RefCell::new(val));
                    Box::into_raw(wrapper) as *mut _
                }
                extern "C" fn godot_free(_this: *mut sys::godot_object, _data: *mut $crate::libc::c_void, ud: *mut $crate::libc::c_void) {
                    use std::cell::RefCell;
                    let wrapper: Box<RefCell<$name>> = unsafe { Box::from_raw(ud as *mut _) };
                    drop(wrapper);
                }

                let $builder = init_handle.add_class::<Self>(
                    $crate::ClassDescriptor {
                        name: stringify!($name),
                        base_class: <$parent as $crate::GodotObject>::class_name(),
                        constructor: Some(godot_create),
                        destructor: Some(godot_free),
                    }
                );

                godot_class_build_export_methods!($name, $builder, $($tt)*);

                $pbody
            }
        }

        impl $crate::NativeClass for $name {
            fn class_name() -> &'static str { stringify!($name) }
            fn get_header(&self) -> &$crate::NativeInstanceHeader { &self.header }
        }
    )
}

#[cfg(test)]
godot_class! {
    class TestClass: super::Node {
        fields {
            a: u32,
        }

        setup(_builder) {}

        constructor(header) {
            TestClass {
                header,
                a: 42,
            }
        }

        export fn _ready(&mut self) {
            godot_print!("hello, world.");
        }
    }
}

// Class builder

pub type GodotScriptMethodFn = unsafe extern "C" fn(
    *mut sys::godot_object,
    *mut libc::c_void,
    *mut libc::c_void,
    libc::c_int,
    *mut *mut sys::godot_variant
) -> sys::godot_variant;

pub type GodotScriptConstructorFn = unsafe extern "C" fn(
    *mut sys::godot_object,
    *mut libc::c_void
) -> *mut libc::c_void;

pub type GodotScriptDestructorFn = unsafe extern "C" fn(
    *mut sys::godot_object,
    *mut libc::c_void,
    *mut libc::c_void
) -> ();

pub enum GodotRpcMode {
    Disabled,
    Remote,
    Sync,
    Mater,
    Slave
}

pub struct GodotScriptMethodAttributes {
    pub rpc_mode: GodotRpcMode
}

pub struct GodotScriptMethod<'l> {
    pub name: &'l str,
    pub method_ptr: Option<GodotScriptMethodFn>,
    pub attributes: GodotScriptMethodAttributes,

    pub method_data: *mut libc::c_void,
    pub free_func: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
}

pub struct ClassDescriptor<'l> {
    pub name: &'l str,
    pub base_class: &'l str,
    pub constructor: Option<GodotScriptConstructorFn>,
    pub destructor: Option<GodotScriptDestructorFn>,
}

pub struct InitHandle {
    #[doc(hidden)]
    pub handle: *mut libc::c_void,
}

impl InitHandle {
    #[doc(hidden)]
    pub unsafe fn new(handle: *mut libc::c_void) -> Self { InitHandle { handle } }

    pub fn add_class<C>(&self, desc: ClassDescriptor) -> ClassBuilder<C>
    where C: NativeClass {
        unsafe {
            let class_name = CString::new(desc.name).unwrap();
            let base_name = CString::new(desc.base_class).unwrap();

            let create = sys::godot_instance_create_func {
                create_func: desc.constructor,
                method_data: ptr::null_mut(),
                free_func: None,
            };

            let destroy = sys::godot_instance_destroy_func {
                destroy_func: desc.destructor,
                method_data: ptr::null_mut(),
                free_func: None,
            };

            (get_api().godot_nativescript_register_class)(
                self.handle as *mut _,
                class_name.as_ptr() as *const _,
                base_name.as_ptr() as *const _,
                create,
                destroy
            );

            ClassBuilder {
                init_handle: self.handle,
                class_name,
                _marker: PhantomData,
            }
        }
    }
}

pub struct ClassBuilder<C: NativeClass> {
    #[doc(hidden)]
    pub init_handle: *mut libc::c_void,
    class_name: CString,
    _marker: PhantomData<C>,
}

impl<C: NativeClass> ClassBuilder<C> {

    pub fn add_method_advanced(&self, method: GodotScriptMethod) {
        let method_name = CString::new(method.name).unwrap();
        let attr = sys::godot_method_attributes {
            rpc_type: sys::godot_method_rpc_mode::GODOT_METHOD_RPC_MODE_DISABLED
        };

        let method_desc = sys::godot_instance_method {
            method: method.method_ptr,
            method_data: method.method_data,
            free_func: method.free_func
        };

        unsafe {
            (get_api().godot_nativescript_register_method)(
                self.init_handle,
                self.class_name.as_ptr() as *const _,
                method_name.as_ptr() as *const _,
                attr,
                method_desc
            );
        }
    }

    pub fn add_method(&self, name: &str, method: GodotScriptMethodFn) {
        self.add_method_advanced(
            GodotScriptMethod {
                name: name,
                method_ptr: Some(method),
                attributes: GodotScriptMethodAttributes {
                    rpc_mode: GodotRpcMode::Disabled
                },
                method_data: ptr::null_mut(),
                free_func: None
            },
        );
    }

    pub fn add_property<T, S, G>(&self, property: Property<T, S, G>)
    where
        T: GodotType,
        S: PropertySetter<C, T>,
        G: PropertyGetter<C, T>,
    {
        unsafe {
            let hint_text = match property.hint {
                PropertyHint::Range { ref range, step, slider } => {

                    if slider {
                        Some(format!("{},{},{},slider", range.start, range.end, step))
                    } else {
                        Some(format!("{},{},{}", range.start, range.end, step))
                    }
                }
                PropertyHint::Enum { values } | PropertyHint::Flags { values } => { Some(values.join(",")) }
                PropertyHint::NodePathToEditedNode | PropertyHint::None => { None }
            };
            let hint_string = if let Some(text) = hint_text {
                GodotString::from_str(text)
            } else {
                GodotString::default()
            };

            let default: Variant = property.default.to_variant();
            let ty = default.get_type();

            let mut attr = sys::godot_property_attributes {
                rset_type: sys::godot_method_rpc_mode::GODOT_METHOD_RPC_MODE_DISABLED, // TODO:
                type_: mem::transmute(ty),
                hint: property.hint.to_sys(),
                hint_string: hint_string.to_sys(),
                usage: property.usage.to_sys(),
                default_value: default.to_sys(),
            };

            let path = ::std::ffi::CString::new(property.name).unwrap();

            let set = property.setter.as_godot_function();
            let get = property.getter.as_godot_function();

            (get_api().godot_nativescript_register_property)(
                self.init_handle,
                self.class_name.as_ptr(),
                path.as_ptr() as *const _,
                &mut attr, set, get
            );
        }
    }

    pub fn add_signal(&self, signal: Signal) {
        use std::ptr;
        unsafe {
            let name = GodotString::from_str(signal.name);
            (get_api().godot_nativescript_register_signal)(
                self.init_handle,
                self.class_name.as_ptr(),
                &sys::godot_signal {
                    name: name.to_sys(),
                    num_args: 0,
                    args: ptr::null_mut(),
                    num_default_args: 0,
                    default_args: ptr::null_mut(),
                }
            );
        }
    }
}
