use crate::sys;

/// Error codes used in various Godot APIs.
#[allow(clippy::unnecessary_cast)] // False positives: casts necessary for cross-platform
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum GodotError {
    Failed = sys::godot_error_GODOT_FAILED as u32,
    Unavailable = sys::godot_error_GODOT_ERR_UNAVAILABLE as u32,
    Unconfigured = sys::godot_error_GODOT_ERR_UNCONFIGURED as u32,
    Unauthorized = sys::godot_error_GODOT_ERR_UNAUTHORIZED as u32,
    ParameterRange = sys::godot_error_GODOT_ERR_PARAMETER_RANGE_ERROR as u32,
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

impl GodotError {
    /// Creates a `Result<(), GodotError>` from a raw error code.
    ///
    /// This is intended to be an internal API.
    ///
    /// # Safety
    ///
    /// `err` should be a valid value for `GodotError`.
    #[inline]
    #[doc(hidden)]
    #[allow(clippy::unnecessary_cast)] // False positives: casts necessary for cross-platform
    pub unsafe fn result_from_sys(err: sys::godot_error) -> Result<(), Self> {
        if err == sys::godot_error_GODOT_OK {
            return Ok(());
        }

        Err(std::mem::transmute(err as u32))
    }
}

impl std::fmt::Display for GodotError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Godot API error: {self:?}")
    }
}

impl std::error::Error for GodotError {}

/// Result type with [GodotError]
pub type GodotResult = Result<(), GodotError>;
