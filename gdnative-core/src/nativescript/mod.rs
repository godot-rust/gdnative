//! Types and functions related to the NativeScript extension of GDNative.

mod macros;

pub mod class;
pub mod init;
pub mod type_tag;
pub mod user_data;
pub mod utils;

pub use class::*;
pub use init::*;
pub use user_data::{Map, MapMut, UserData};
