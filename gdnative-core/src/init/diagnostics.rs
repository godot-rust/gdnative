//! Run-time tracing functions to help debug the init process.
//!
//! The provided functions are designed to convey any issues found through human-readable
//! error output, while programmatically providing only an overall indication of whether
//! any problems were found. This is so that they can be freely improved without compatibility
//! concerns.

mod missing_manual_registration;
mod missing_suggested_diagnostics;

#[doc(inline)]
pub use missing_manual_registration::missing_manual_registration;

#[doc(inline)]
pub use missing_suggested_diagnostics::missing_suggested_diagnostics;
