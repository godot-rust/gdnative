use libc;
use sys;
use std::ops::Deref;
use GodotString;
use GodotObject;
use Object;
use NativeScript;
use object;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use get_api;

/// Godot native class implementation detail that must be stored
/// instances.
pub struct NativeInstanceHeader {
    #[doc(hidden)]
    pub this: *mut sys::godot_object,
}

pub unsafe trait NativeClass {
    fn class_name() -> &'static str;

    fn get_header(&self) -> &NativeInstanceHeader;

    fn as_object(&self) -> &Object {
        unsafe {
            mem::transmute(self.get_header())
        }
    }

    unsafe fn register_class(desc: *mut libc::c_void);
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
macro_rules! godot_class_count_params {
    () => (0);
    ($name:ident, $($other:ident,)*) => (
        1 + godot_class_count_params!($($other,)*)
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! godot_class_build_export_methods {
    ($classty:ty, $class:ident, $desc:ident,) => ();
    ($classty:ty, $class:ident, $desc:ident,
        export fn $name:ident(
            &mut self
            $(,$pname:ident : $pty:ty)*
        ) $body:block
        $($tt:tt)*
    ) => (
        godot_class_build_export_methods!($classty, $class, $desc,
            export fn $name(&mut self $(,$pname : $pty)*) -> () $body
            $($tt)*
        );
    );
    ($classty:ty, $class:ident, $desc:ident,
        export fn $name:ident(
            &mut self
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty $body:block
        $($tt:tt)*
    ) => (
        {
            #[allow(unused_assignments, unused_unsafe, dead_code, unused_variables, unused_mut)]
            extern "C" fn godot_invoke(
                _obj: *mut $crate::sys::godot_object,
                _md: *mut $crate::libc::c_void,
                ud: *mut $crate::libc::c_void,
                num_args: $crate::libc::c_int,
                args: *mut *mut $crate::sys::godot_variant
            ) -> $crate::sys::godot_variant {
                use std::cell::RefCell;
                use std::panic::{self, AssertUnwindSafe};
                unsafe {
                    let api = $crate::get_api();

                    let num_params = godot_class_count_params!($($pname,)*);
                    if num_args < num_params {
                        godot_error!("Incorrect number of parameters: got {} and wanted {}", num_args, num_params);
                        let mut ret = $crate::sys::godot_variant::default();
                        (api.godot_variant_new_nil)(&mut ret);
                        return ret;
                    }
                    let mut offset = 0;
                    $(
                        let $pname = if let Some(val) = <$pty as $crate::GodotType>::from_sys_variant(&mut *(*args).offset(offset)) {
                            val
                        } else {
                            godot_error!("Incorrect parameter type for parameter {}", offset);
                            let mut ret = $crate::sys::godot_variant::default();
                            (api.godot_variant_new_nil)(&mut ret);
                            return ret;
                        };
                        offset += 1;
                    )*
                    let __rust_ty = &*(ud as *mut RefCell<$classty>);
                    let mut __rust_ty = __rust_ty.borrow_mut();
                    let rust_ret = match panic::catch_unwind(AssertUnwindSafe(|| {
                        __rust_ty.$name($(
                            $pname
                        ),*);
                    })) {
                        Ok(val) => val,
                        Err(err) => {
                            let err = if let Some(err) = err.downcast_ref::<&str>() {
                                (*err).to_owned()
                            } else if let Some(err) = err.downcast_ref::<String>() {
                                (*err).clone()
                            } else {
                                "Unknown".to_owned()
                            };
                            godot_error!("Method call failed, everything may be in an invalid state: {:?}", err);
                            let mut ret = $crate::sys::godot_variant::default();
                            (api.godot_variant_new_nil)(&mut ret);
                            return ret;
                        }
                    };
                    <$retty as $crate::GodotType>::to_variant(&rust_ret).forget()
                }
            }
            let method = $crate::sys::godot_instance_method {
                method: Some(godot_invoke),
                method_data: ::std::ptr::null_mut(),
                free_func: None,
            };
            let attr = $crate::sys::godot_method_attributes {
                rpc_type: $crate::sys::godot_method_rpc_mode::GODOT_METHOD_RPC_MODE_DISABLED,
            };
            let name = CString::new(stringify!($name)).unwrap();
            ($crate::get_api().godot_nativescript_register_method)(
                $desc as *mut _,
                $class.as_ptr() as *const _,
                name.as_ptr() as *const _,
                attr,
                method
            );
        }
        godot_class_build_export_methods!($classty, $class, $desc, $($tt)*);
    )
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
        }

        unsafe impl $crate::NativeClass for $name {

            unsafe fn register_class(desc: *mut $crate::libc::c_void) {
                use $crate::sys;
                use std::ffi::CString;
                use std::ptr;
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

                let cname = CString::new(stringify!($name)).unwrap();
                let pname = CString::new(
                    <$parent as $crate::GodotObject>::class_name()
                ).unwrap();

                let create = sys::godot_instance_create_func {
                    create_func: Some(godot_create),
                    method_data: ptr::null_mut(),
                    free_func: None,
                };

                let destroy = sys::godot_instance_destroy_func {
                    destroy_func: Some(godot_free),
                    method_data: ptr::null_mut(),
                    free_func: None,
                };

                ($crate::get_api().godot_nativescript_register_class)(
                    desc as *mut _,
                    cname.as_ptr() as *const _,
                    pname.as_ptr() as *const _,
                    create,
                    destroy
                );

                godot_class_build_export_methods!($name, cname, desc, $($tt)*);

                let $builder: $crate::PropertyBuilder<$name>  = $crate::PropertyBuilder {
                    desc: desc,
                    class_name: cname.as_ptr() as *const _,
                    _marker: ::std::marker::PhantomData,
                };
                $pbody
            }

            fn class_name() -> &'static str {
                stringify!($name)
            }

            fn get_header(&self) -> &$crate::NativeInstanceHeader {
                &self.header
            }
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
