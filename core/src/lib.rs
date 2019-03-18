//! # Rust bindings for the Godot game engine
//!
//! This crate contains high-level wrappers around the Godot game engine's gdnaive API.
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
pub extern crate libc;
#[doc(hidden)]
pub extern crate gdnative_sys as sys;
#[macro_use]
extern crate bitflags;

pub extern crate gdnative_geom as geom;

mod macros;
#[macro_use]
mod class;
mod free_on_drop;
mod internal;
mod color;
mod variant;
mod variant_array;
mod dictionary;
mod rid;
mod generated;
mod node_path;
mod string;
mod byte_array;
mod int32_array;
mod float32_array;
mod string_array;
mod vector2;
mod vector2_array;
mod vector3;
mod vector3_array;
mod color_array;
pub mod init;
#[doc(hidden)]
pub mod object;

pub use crate::internal::*;
pub use crate::class::*;
pub use crate::free_on_drop::*;
pub use crate::variant::*;
pub use crate::variant_array::*;
pub use crate::dictionary::*;
pub use crate::geom::*;
pub use crate::color::*;
pub use crate::rid::*;
pub use crate::node_path::*;
pub use crate::generated::*;
pub use crate::string::*;
pub use crate::byte_array::*;
pub use crate::int32_array::*;
pub use crate::float32_array::*;
pub use crate::string_array::*;
pub use crate::vector2::*;
pub use crate::vector2_array::*;
pub use crate::vector3::*;
pub use crate::vector3_array::*;
pub use crate::color_array::*;
pub use crate::object::GodotObject;

use std::mem;

#[doc(hidden)]
pub static mut GODOT_API: Option<GodotApi> = None;
#[inline]
#[doc(hidden)]
pub fn get_api() -> &'static GodotApi {
    unsafe { GODOT_API.as_ref().expect("API not bound") }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GodotError {
    Failed = sys::godot_error_GODOT_FAILED,
    Unavailable = sys::godot_error_GODOT_ERR_UNAVAILABLE,
    Unconfigured = sys::godot_error_GODOT_ERR_UNCONFIGURED,
    Unothorized = sys::godot_error_GODOT_ERR_UNAUTHORIZED,
    PrameterRange = sys::godot_error_GODOT_ERR_PARAMETER_RANGE_ERROR,
    OutOfMemory = sys::godot_error_GODOT_ERR_OUT_OF_MEMORY,
    FileNotFound = sys::godot_error_GODOT_ERR_FILE_NOT_FOUND,
    FileBadDrive = sys::godot_error_GODOT_ERR_FILE_BAD_DRIVE,
    FileBadPath = sys::godot_error_GODOT_ERR_FILE_BAD_PATH,
    FileNoPermission = sys::godot_error_GODOT_ERR_FILE_NO_PERMISSION,
    FileAlreadyInUse = sys::godot_error_GODOT_ERR_FILE_ALREADY_IN_USE,
    FileCantOpen = sys::godot_error_GODOT_ERR_FILE_CANT_OPEN,
    FileCantWrite = sys::godot_error_GODOT_ERR_FILE_CANT_WRITE,
    FileCantRead = sys::godot_error_GODOT_ERR_FILE_CANT_READ,
    FileUnrecognized = sys::godot_error_GODOT_ERR_FILE_UNRECOGNIZED,
    FileCorrupt = sys::godot_error_GODOT_ERR_FILE_CORRUPT,
    FileMissingDependency = sys::godot_error_GODOT_ERR_FILE_MISSING_DEPENDENCIES,
    FileEof = sys::godot_error_GODOT_ERR_FILE_EOF,
    CantOpen = sys::godot_error_GODOT_ERR_CANT_OPEN,
    CantCreate = sys::godot_error_GODOT_ERR_CANT_CREATE,
    QueryFailed = sys::godot_error_GODOT_ERR_QUERY_FAILED,
    AlreadyInUse = sys::godot_error_GODOT_ERR_ALREADY_IN_USE,
    Locked = sys::godot_error_GODOT_ERR_LOCKED,
    TimeOut = sys::godot_error_GODOT_ERR_TIMEOUT,
    CantConnect = sys::godot_error_GODOT_ERR_CANT_CONNECT,
    CantResolve = sys::godot_error_GODOT_ERR_CANT_RESOLVE,
    ConnectionError = sys::godot_error_GODOT_ERR_CONNECTION_ERROR,
    CantAcquireResource = sys::godot_error_GODOT_ERR_CANT_ACQUIRE_RESOURCE,
    CantFork = sys::godot_error_GODOT_ERR_CANT_FORK,
    InvalidData = sys::godot_error_GODOT_ERR_INVALID_DATA,
    InvalidParameter = sys::godot_error_GODOT_ERR_INVALID_PARAMETER,
    AlreadyExists = sys::godot_error_GODOT_ERR_ALREADY_EXISTS,
    DoesNotExist = sys::godot_error_GODOT_ERR_DOES_NOT_EXIST,
    DatabaseCantRead = sys::godot_error_GODOT_ERR_DATABASE_CANT_READ,
    DatabaseCantWrite = sys::godot_error_GODOT_ERR_DATABASE_CANT_WRITE,
    CompilationFailed = sys::godot_error_GODOT_ERR_COMPILATION_FAILED,
    MethodNotFound = sys::godot_error_GODOT_ERR_METHOD_NOT_FOUND,
    LinkFailed = sys::godot_error_GODOT_ERR_LINK_FAILED,
    ScriptFailed = sys::godot_error_GODOT_ERR_SCRIPT_FAILED,
    CyclicLink = sys::godot_error_GODOT_ERR_CYCLIC_LINK,
    InvalidDeclaration = sys::godot_error_GODOT_ERR_INVALID_DECLARATION,
    DuplicateSymbol = sys::godot_error_GODOT_ERR_DUPLICATE_SYMBOL,
    ParseError = sys::godot_error_GODOT_ERR_PARSE_ERROR,
    Busy = sys::godot_error_GODOT_ERR_BUSY,
    Skip = sys::godot_error_GODOT_ERR_SKIP,
    Help = sys::godot_error_GODOT_ERR_HELP,
    Bug = sys::godot_error_GODOT_ERR_BUG,
    PrinterOnFire = sys::godot_error_GODOT_ERR_PRINTER_ON_FIRE,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Vector3Axis {
    X = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_X,
    Y = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Y,
    Z = sys::godot_vector3_axis_GODOT_VECTOR3_AXIS_Z,
}

pub type GodotResult = Result<(), GodotError>;

pub fn result_from_sys(err: sys::godot_error) -> GodotResult {
    if err == sys::godot_error_GODOT_OK {
        return Ok(());
    }

    Err(unsafe { mem::transmute(err) })
}
