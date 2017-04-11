#![feature(proc_macro)]

extern crate gdnative_macros;
pub extern crate gdnative_sys;

pub use gdnative_sys::core_types::*;

use std::ffi::{
  CString,
};
use std::panic::catch_unwind;
use std::boxed::Box;

pub use gdnative_macros::godot_export;

#[allow(non_camel_case_types)]
type void = std::os::raw::c_void;

pub trait GodotObject where Self: std::marker::Sized + std::panic::RefUnwindSafe, Self: GodotObjectRegisterMethods {
  unsafe extern "C" fn _new(p_instance: *mut gdnative_sys::godot_object, _: *mut void) -> *mut void {
    if let Ok(r) = catch_unwind(|| {
      Box::into_raw(Box::new(Self::new())) as *mut void
    }) {
      r
    } else {
      println!("new panicked! ending the process");
      std::process::exit(1);
    }
  }
  fn new() -> Self;

  unsafe extern "C" fn _destroy(p_instance: *mut gdnative_sys::godot_object, _: *mut void, p_data: *mut void) {
    let ref mut this = *(p_data as *mut Self);
    if let Err(_) = catch_unwind(|| {
      this.destroy();
    }) {
      println!("destroy panicked! ending the process");
      std::process::exit(1);
    }
  }
  fn destroy(&self) {}
}

pub trait GodotObjectRegisterMethods {
  fn register_methods();
}

pub unsafe fn register_godot_class<T: GodotObject>(class_name: &str, derives: &str) -> Result<(),()> {
  gdnative_sys::godot_script_register_class(CString::new(class_name).unwrap().as_ptr(), CString::new(derives).unwrap().as_ptr(), 
                                            gdnative_sys::godot_instance_create_func{
                                              create_func: Some(T::_new),
                                              method_data: std::ptr::null_mut(),
                                              free_func: None
                                            },
                                            gdnative_sys::godot_instance_destroy_func{
                                              destroy_func: Some(T::_destroy),
                                              method_data: std::ptr::null_mut(),
                                              free_func: None
                                            });
  Ok(())
}

#[macro_export]
macro_rules! generate_gdnative_init {
  ( $($t:ident),* ) => {
    #[no_mangle]
    pub unsafe extern "C" fn godot_native_init(options: *mut gdnative_sys::godot_native_init_options) {
      use godot::GodotObjectRegisterMethods;
      $(
        godot::register_godot_class::<$t>(stringify!($t), "Node").unwrap(); //TODO: derives from type
        $t::register_methods();
      )*
    }
  }
}
