//! # Rust bindings for the Godot game engine
//!
//! This crate contains high-level wrappers around the Godot game engine's gdnative API.
//! Some of the types were automatically generated from the engine's JSON API description,
//! and some other types are hand made wrappers around the core C types.
//!
//! ## Memory management
//!
//! ### Reference counting
//!
//! A lot of the types provided by the engine are internally reference counted and
//! allow mutable aliasing.
//! In rust parlance this means that a type such as `gdnative::ConcavePolygonShape2D`
//! is functionally equivalent to a `Rc<Cell<Something>>` rather than `Rc<Something>`.
//!
//! Since it is easy to expect containers and other types to allocate a copy of their
//! content when using the `Clone` trait, most of these types do not implement `Clone`
//! and instead provide a `new_ref(&self) -> Self` method to create references to the
//! same collection or object.
//!
//! ### Manually managed objects
//!
//! Some types are manually managed. This means that ownership can be passed to the
//! engine or the object must be carefully deallocated using the object's `free`  method.
//!

#[doc(hidden)]
pub extern crate gdnative_sys as sys;
#[doc(hidden)]
pub extern crate libc;
#[macro_use]
extern crate bitflags;
extern crate parking_lot;

pub mod geom;

mod macros;
#[macro_use]
mod class;
pub mod access;
mod byte_array;
mod color;
mod color_array;
mod dictionary;
mod float32_array;
mod free_on_drop;
mod generated;
pub mod init;
mod int32_array;
mod node_path;
#[doc(hidden)]
pub mod object;
mod point2;
mod rid;
mod string;
mod string_array;
mod type_tag;
pub mod user_data;
mod variant;
mod variant_array;
mod vector2;
mod vector2_array;
mod vector3;
mod vector3_array;

pub use crate::byte_array::*;
pub use crate::class::*;
pub use crate::color::*;
pub use crate::color_array::*;
pub use crate::dictionary::*;
pub use crate::float32_array::*;
pub use crate::free_on_drop::*;
pub use crate::generated::*;
pub use crate::geom::*;
pub use crate::int32_array::*;
pub use crate::node_path::*;
pub use crate::object::GodotObject;
pub use crate::object::Instanciable;
pub use crate::point2::*;
pub use crate::rid::*;
pub use crate::string::*;
pub use crate::string_array::*;
pub use crate::user_data::Map;
pub use crate::user_data::MapMut;
pub use crate::user_data::UserData;
pub use crate::variant::*;
pub use crate::variant_array::*;
pub use crate::vector2::*;
pub use crate::vector2_array::*;
pub use crate::vector3::*;
pub use crate::vector3_array::*;

pub use sys::GodotApi;

use std::mem;

#[doc(hidden)]
pub static mut GODOT_API: Option<GodotApi> = None;
#[doc(hidden)]
pub static mut GDNATIVE_LIBRARY_SYS: Option<*mut sys::godot_object> = None;
#[inline]
#[doc(hidden)]
pub fn get_api() -> &'static GodotApi {
    unsafe { GODOT_API.as_ref().expect("API not bound") }
}
#[inline]
#[doc(hidden)]
pub fn get_gdnative_library_sys() -> *mut sys::godot_object {
    unsafe { GDNATIVE_LIBRARY_SYS.expect("GDNativeLibrary not bound") }
}
#[inline]
#[doc(hidden)]
pub unsafe fn cleanup_internal_state() {
    type_tag::cleanup();
    GODOT_API = None;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GodotError {
    Failed = sys::godot_error_GODOT_FAILED as u32,
    Unavailable = sys::godot_error_GODOT_ERR_UNAVAILABLE as u32,
    Unconfigured = sys::godot_error_GODOT_ERR_UNCONFIGURED as u32,
    Unothorized = sys::godot_error_GODOT_ERR_UNAUTHORIZED as u32,
    PrameterRange = sys::godot_error_GODOT_ERR_PARAMETER_RANGE_ERROR as u32,
    OutOfMemory = sys::godot_error_GODOT_ERR_OUT_OF_MEMORY as u32,
    FileNotFound = sys::godot_error_GODOT_ERR_FILE_NOT_FOUND as u32,
    FileBadDrive = sys::godot_error_GODOT_ERR_FILE_BAD_DRIVE as u32,
    FileBadPath = sys::godot_error_GODOT_ERR_FILE_BAD_PATH as u32,
    FileNoPermission = sys::godot_error_GODOT_ERR_FILE_NO_PERMISSION as u32,
    FileAlreadyInUse = sys::godot_error_GODOT_ERR_FILE_ALREADY_IN_USE as u32,
    FileCantOpen = sys::godot_error_GODOT_ERR_FILE_CANT_OPEN as u32,
    FileCantWrite = sys::godot_error_GODOT_ERR_FILE_CANT_WRITE as u32,
    FileCantRead = sys::godot_error_GODOT_ERR_FILE_CANT_READ as u32,
    FileUnrecognized = sys::godot_error_GODOT_ERR_FILE_UNRECOGNIZED as u32,
    FileCorrupt = sys::godot_error_GODOT_ERR_FILE_CORRUPT as u32,
    FileMissingDependency = sys::godot_error_GODOT_ERR_FILE_MISSING_DEPENDENCIES as u32,
    FileEof = sys::godot_error_GODOT_ERR_FILE_EOF as u32,
    CantOpen = sys::godot_error_GODOT_ERR_CANT_OPEN as u32,
    CantCreate = sys::godot_error_GODOT_ERR_CANT_CREATE as u32,
    QueryFailed = sys::godot_error_GODOT_ERR_QUERY_FAILED as u32,
    AlreadyInUse = sys::godot_error_GODOT_ERR_ALREADY_IN_USE as u32,
    Locked = sys::godot_error_GODOT_ERR_LOCKED as u32,
    TimeOut = sys::godot_error_GODOT_ERR_TIMEOUT as u32,
    CantConnect = sys::godot_error_GODOT_ERR_CANT_CONNECT as u32,
    CantResolve = sys::godot_error_GODOT_ERR_CANT_RESOLVE as u32,
    ConnectionError = sys::godot_error_GODOT_ERR_CONNECTION_ERROR as u32,
    CantAcquireResource = sys::godot_error_GODOT_ERR_CANT_ACQUIRE_RESOURCE as u32,
    CantFork = sys::godot_error_GODOT_ERR_CANT_FORK as u32,
    InvalidData = sys::godot_error_GODOT_ERR_INVALID_DATA as u32,
    InvalidParameter = sys::godot_error_GODOT_ERR_INVALID_PARAMETER as u32,
    AlreadyExists = sys::godot_error_GODOT_ERR_ALREADY_EXISTS as u32,
    DoesNotExist = sys::godot_error_GODOT_ERR_DOES_NOT_EXIST as u32,
    DatabaseCantRead = sys::godot_error_GODOT_ERR_DATABASE_CANT_READ as u32,
    DatabaseCantWrite = sys::godot_error_GODOT_ERR_DATABASE_CANT_WRITE as u32,
    CompilationFailed = sys::godot_error_GODOT_ERR_COMPILATION_FAILED as u32,
    MethodNotFound = sys::godot_error_GODOT_ERR_METHOD_NOT_FOUND as u32,
    LinkFailed = sys::godot_error_GODOT_ERR_LINK_FAILED as u32,
    ScriptFailed = sys::godot_error_GODOT_ERR_SCRIPT_FAILED as u32,
    CyclicLink = sys::godot_error_GODOT_ERR_CYCLIC_LINK as u32,
    InvalidDeclaration = sys::godot_error_GODOT_ERR_INVALID_DECLARATION as u32,
    DuplicateSymbol = sys::godot_error_GODOT_ERR_DUPLICATE_SYMBOL as u32,
    ParseError = sys::godot_error_GODOT_ERR_PARSE_ERROR as u32,
    Busy = sys::godot_error_GODOT_ERR_BUSY as u32,
    Skip = sys::godot_error_GODOT_ERR_SKIP as u32,
    Help = sys::godot_error_GODOT_ERR_HELP as u32,
    Bug = sys::godot_error_GODOT_ERR_BUG as u32,
    PrinterOnFire = sys::godot_error_GODOT_ERR_PRINTER_ON_FIRE as u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Vector3Axis {
    X = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_X as u32,
    Y = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Y as u32,
    Z = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Z as u32,
}

pub type GodotResult = Result<(), GodotError>;

pub fn result_from_sys(err: sys::godot_error) -> GodotResult {
    if err == sys::godot_error_GODOT_OK {
        return Ok(());
    }

    Err(unsafe { mem::transmute(err as u32) })
}

pub unsafe fn report_init_error(
    options: *const sys::godot_gdnative_init_options,
    error: sys::InitError,
) {
    use std::ffi::CString;
    match error {
        sys::InitError::VersionMismatch {
            api_type,
            want,
            got,
        } => {
            if let Some(f) = (*options).report_version_mismatch {
                f(
                    (*options).gd_native_library,
                    CString::new(format!("{}", api_type)).unwrap().as_ptr(),
                    want,
                    got,
                );
            }
        }
        sys::InitError::Generic { message } => {
            if let Some(f) = (*options).report_loading_error {
                f(
                    (*options).gd_native_library,
                    CString::new(message).unwrap().as_ptr(),
                );
            }
        }
    }
}
