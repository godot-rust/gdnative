use std;
use godot_core_api::*;

impl From<()> for godot_variant {
  fn from(i: ()) -> godot_variant {
    unsafe {
      let mut ret: godot_variant = std::mem::uninitialized();
      godot_variant_new_nil(&mut ret as *mut _);
      ret
    }
  }
}
impl Into<()> for godot_variant {
  fn into(self) -> () {
    unsafe {
      if (godot_variant_get_type(&self as *const _) != godot_variant_type::GODOT_VARIANT_TYPE_NIL) {
        panic!("Unexpected variant type!");
      }
      ()
    }
  }
}

impl From<bool> for godot_variant {
  fn from(i: bool) -> godot_variant {
    unsafe{
      let mut ret: godot_variant = std::mem::uninitialized();
      godot_variant_new_bool(&mut ret as *mut _, i); 
      ret
    }
  }
}
impl Into<bool> for godot_variant {
  fn into(self) -> bool {
    unsafe {
      if (godot_variant_get_type(&self as *const _) != godot_variant_type::GODOT_VARIANT_TYPE_BOOL) {
        panic!("Unexpected variant type!");
      }
      godot_variant_as_bool(&self as *const _)
    }
  }
}

impl From<u64> for godot_variant {
  fn from(i: u64) -> godot_variant {
    unsafe{
      let mut ret: godot_variant = std::mem::uninitialized();
      godot_variant_new_uint(&mut ret as *mut _, i);
      ret
    }
  }
}

impl From<i64> for godot_variant {
  fn from(i: i64) -> godot_variant {
    unsafe{
      let mut ret: godot_variant = std::mem::uninitialized();
      godot_variant_new_int(&mut ret as *mut _, i); 
      ret
    }
  }
}
impl Into<i64> for godot_variant {
  fn into(self) -> i64 {
    unsafe {
      if (godot_variant_get_type(&self as *const _) != godot_variant_type::GODOT_VARIANT_TYPE_INT) {
        panic!("Unexpected variant type!");
      }
      godot_variant_as_int(&self as *const _)
    }
  }
}

impl From<String> for godot_variant {
  fn from(i: String) -> godot_variant {
    unsafe{
      let mut ret: godot_variant = std::mem::uninitialized();
      godot_variant_new_nil(&mut ret as *mut _); 
      ret
    } //TODO: actually create an actual variant object
  }
}
impl<'a> From<&'a str> for godot_variant {
  fn from(i: &'a str) -> godot_variant {
    unsafe{
      let mut ret: godot_variant = std::mem::uninitialized();
      godot_variant_new_nil(&mut ret as *mut _); 
      ret
    } //TODO: actually create an actual variant object
  }
}

impl<T> From<Option<T>> for godot_variant where godot_variant: From<T> {
  fn from(i: Option<T>) -> godot_variant {
    match i {
      None => ().into(),
      Some(i) => i.into()
    }
  }
}
/*impl<T> Into<Option<T>> for godot_variant where godot_variant: Into<T> {
  fn into(self) -> Option<T> {
    unsafe {
      match godot_variant_get_type(&self as *const _) {
        godot_variant_type::GODOT_VARIANT_TYPE_NIL => None,
        _ => Some(self.into::<T>())
      }
    }
  }
}*/
