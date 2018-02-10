use libc;
use sys;
use std::ops::Deref;

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
                        let $pname = if let Some(val) = <$pty as $crate::GodotType>::from_variant(&mut *(*args).offset(offset)) {
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
                    <$retty as $crate::GodotType>::as_variant(&rust_ret)
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
    constructor($godot_info:ident) $construct:block

    $($tt:tt)*
}
    ) => (
        pub struct $name {
            godot_info: $crate::GodotClassInfo,
            $(
                $(#[$fattr])*
                pub $fname: $fty,
            )*
        }

        impl $name {
            godot_class_build_methods!($($tt)*);

            pub fn godot_parent(&self) -> $crate::GodotRef<$parent> {
                unsafe {
                    $crate::GodotRef::from_raw(self.godot_info.this)
                }
            }
        }

        unsafe impl $crate::GodotClass for $name {
            type ClassData = ();
            type Reference = ::std::cell::RefCell<$name>;
            fn godot_name() -> &'static str {
                stringify!($name)
            }
            fn godot_info(&self) -> &$crate::GodotClassInfo {
                &self.godot_info
            }
            unsafe fn from_object(_obj: *mut $crate::sys::godot_object) -> Self::ClassData {
                ()
            }
            unsafe fn reference(this: *mut $crate::sys::godot_object, _data: &Self::ClassData) -> &Self::Reference {
                let api = $crate::get_api();
                let ud = (api.godot_nativescript_get_userdata)(this);
                &*(ud as *const _ as *const ::std::cell::RefCell<$name>)
            }

            unsafe fn register_class(desc: *mut $crate::libc::c_void) {
                use $crate::sys;
                use std::ffi::CString;
                use std::ptr;
                fn constructor($godot_info : $crate::GodotClassInfo) -> $name {
                    $construct
                }

                extern "C" fn godot_create(this: *mut sys::godot_object, _data: *mut $crate::libc::c_void) -> *mut $crate::libc::c_void {
                    use std::cell::RefCell;
                    let val = constructor($crate::GodotClassInfo {
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
                    <$parent as $crate::GodotClass>::godot_name()
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

                let mut $builder: $crate::PropertiesBuilder<$name>  = $crate::PropertiesBuilder {
                    desc: desc,
                    class_name: cname.as_ptr() as *const _,
                    class: ::std::marker::PhantomData,
                };
                $pbody
            }
        }
    )
}

pub unsafe trait GodotClass {
    type ClassData;
    type Reference;

    fn godot_name() -> &'static str;
    fn godot_info(&self) -> &GodotClassInfo;
    unsafe fn from_object(obj: *mut sys::godot_object) -> Self::ClassData;
    unsafe fn register_class(desc: *mut libc::c_void);
    unsafe fn reference(this: *mut sys::godot_object, data: &Self::ClassData) -> &Self::Reference;
}

pub struct GodotClassInfo {
    #[doc(hidden)]
    pub this: *mut sys::godot_object,
}

pub struct GodotRef<T: GodotClass> {
    pub(crate) this: *mut sys::godot_object,
    pub(crate) data: T::ClassData,
    pub(crate) reference: bool,
}

macro_rules! call_bool {
    ($obj:expr, $class:ident, $method:ident) => ({
        use std::ptr;
        use std::ffi;
        use std::sync::{Once, ONCE_INIT};
        #[allow(unused_unsafe)]
        unsafe {
            let api = ::get_api();
            static mut METHOD_BIND: *mut sys::godot_method_bind = 0 as _;
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| {
                let class = ffi::CString::new(stringify!($class)).unwrap();
                let method = ffi::CString::new(stringify!($method)).unwrap();
                METHOD_BIND = (api.godot_method_bind_get_method)(
                    class.as_ptr() as *const _,
                    method.as_ptr() as *const _
                );
            });

            let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];

            let mut ret = false;
            let ret_ptr = &mut ret as *mut _;
            (api.godot_method_bind_ptrcall)(METHOD_BIND, $obj, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);
            ret

        }
    })
}


impl <T> GodotRef<T>
    where T: GodotClass
{
    pub unsafe fn from_object(obj: *mut sys::godot_object) -> GodotRef<T> {
        let reference = Self::is_class(obj, "Reference");
        if reference && !call_bool!(obj, Reference, init_ref) {
            godot_error!("Failed to init reference");
        }
        GodotRef {
            this: obj,
            data: T::from_object(obj),
            reference: reference,
        }
    }

    pub unsafe fn from_raw(obj: *mut sys::godot_object) -> GodotRef<T> {
        GodotRef {
            this: obj,
            data: T::from_object(obj),
            reference: false,
        }
    }

    pub fn cast<O>(&self) -> Option<GodotRef<O>>
        where O: GodotClass
    {
        if Self::is_class(self.this, O::godot_name()) {
            Some(if self.reference {
                call_bool!(self.this, Reference, reference);
                GodotRef {
                    this: self.this,
                    data: unsafe { O::from_object(self.this) },
                    reference: true,
                }
            } else {
                GodotRef {
                    this: self.this,
                    data: unsafe { O::from_object(self.this) },
                    reference: false,
                }
            })
        } else {
            None
        }
    }

    pub fn cast_native<O>(&self) -> Option<GodotRef<O>>
        where O: GodotClass
    {
        let obj: GodotRef<::Object> = GodotRef {
            this: self.this,
            data: unsafe { ::Object::from_object(self.this) },
            reference: false,
        };
        if let Some(script) = obj.get_script().and_then(|v| v.cast::<::NativeScript>()) {
            let class = script.get_class_name();
            if class == O::godot_name() {
                Some(if self.reference {
                    call_bool!(self.this, Reference, reference);
                    GodotRef {
                        this: self.this,
                        data: unsafe { O::from_object(self.this) },
                        reference: true,
                    }
                } else {
                    GodotRef {
                        this: self.this,
                        data: unsafe { O::from_object(self.this) },
                        reference: false,
                    }
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn is_class(obj: *mut sys::godot_object, name: &str) -> bool {
        use std::ptr;
        use std::ffi;
        use std::sync::{Once, ONCE_INIT};
        unsafe {
            let api = ::get_api();
            static mut METHOD_BIND: *mut sys::godot_method_bind = 0 as _;
            static INIT: Once = ONCE_INIT;
            INIT.call_once(|| {
                let class = ffi::CString::new("Object").unwrap();
                let method = ffi::CString::new("is_class").unwrap();
                METHOD_BIND = (api.godot_method_bind_get_method)(
                    class.as_ptr() as *const _,
                    method.as_ptr() as *const _
                );
            });

            let mut argument_buffer = [ptr::null() as *const libc::c_void; 1];

            let mut godot_name = (api.godot_string_chars_to_utf8_with_len)(name.as_ptr() as *const _, name.len() as _);
            argument_buffer[0] = (&godot_name) as *const _ as *const _;

            let mut ret = false;
            let ret_ptr = &mut ret as *mut _;
            (api.godot_method_bind_ptrcall)(METHOD_BIND, obj, argument_buffer.as_mut_ptr() as *mut _, ret_ptr as *mut _);
            (api.godot_string_destroy)(&mut godot_name);
            ret

        }
    }
}


impl <T> Deref for GodotRef<T>
    where T: GodotClass
{
    type Target = T::Reference;

    fn deref(&self) -> &Self::Target {
        unsafe {
            T::reference(self.this, &self.data)
        }
    }
}

impl <T: GodotClass> Drop for GodotRef<T> {
    fn drop(&mut self) {
        if self.reference && call_bool!(self.this, Reference, unreference) {
            unsafe {
                (::get_api().godot_object_destroy)(self.this);
            }
        }
    }
}