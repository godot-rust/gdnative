//! Method registration

use std::borrow::Cow;
use std::marker::PhantomData;
use std::{fmt, ops};

use crate::core_types::{FromVariant, FromVariantError, Variant};
use crate::export::class::NativeClass;
use crate::export::{class_registry, ClassBuilder};
use crate::log::Site;
use crate::object::ownership::Shared;
use crate::object::{Ref, TInstance, TRef};

/// Builder type used to register a method on a `NativeClass`.
#[must_use = "MethodBuilder left unbuilt -- did you forget to call done() or done_stateless()?"]
pub struct MethodBuilder<'a, C, F> {
    class_builder: &'a ClassBuilder<C>,
    name: &'a str,
    method: F,

    rpc_mode: RpcMode,
}

impl<'a, C, F> MethodBuilder<'a, C, F>
where
    C: NativeClass,
    F: Method<C>,
{
    pub(super) fn new(class_builder: &'a ClassBuilder<C>, name: &'a str, method: F) -> Self {
        MethodBuilder {
            class_builder,
            name,
            method,
            rpc_mode: RpcMode::Disabled,
        }
    }

    /// Set a RPC mode for this method.
    #[inline]
    pub fn with_rpc_mode(mut self, rpc_mode: RpcMode) -> Self {
        self.rpc_mode = rpc_mode;
        self
    }

    /// Register the method.
    #[inline]
    pub fn done(self) {
        let method_data = Box::into_raw(Box::new(self.method));

        let script_method = ScriptMethod {
            name: self.name,
            method_ptr: Some(method_wrapper::<C, F>),
            attributes: ScriptMethodAttributes {
                rpc_mode: self.rpc_mode,
            },
            method_data: method_data as *mut libc::c_void,
            free_func: Some(free_func::<F>),
        };

        self.class_builder.add_method(script_method);
    }
}

impl<'a, C, F> MethodBuilder<'a, C, F>
where
    C: NativeClass,
    F: Method<C> + Copy + Default,
{
    /// Register the method as a stateless method. Stateless methods do not have data
    /// pointers and destructors and are thus slightly lighter. This is intended for ZSTs,
    /// but can be used with any `Method` type with `Copy + Default`.
    #[inline]
    pub fn done_stateless(self) {
        let script_method = ScriptMethod {
            name: self.name,
            method_ptr: Some(method_wrapper::<C, Stateless<F>>),
            attributes: ScriptMethodAttributes {
                rpc_mode: self.rpc_mode,
            },

            // Stateless<F> is a ZST for any type F, so we can use any non-zero value as
            // a valid pointer for it.
            method_data: 1 as *mut libc::c_void,
            free_func: None,
        };

        self.class_builder.add_method(script_method);
    }
}

type ScriptMethodFn = unsafe extern "C" fn(
    *mut sys::godot_object,
    *mut libc::c_void,
    *mut libc::c_void,
    libc::c_int,
    *mut *mut sys::godot_variant,
) -> sys::godot_variant;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[non_exhaustive]
pub enum RpcMode {
    #[default]
    Disabled,
    Remote,
    RemoteSync,
    Master,
    Puppet,
    MasterSync,
    PuppetSync,
}

impl RpcMode {
    pub(crate) fn sys(self) -> sys::godot_method_rpc_mode {
        match self {
            RpcMode::Master => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_MASTER,
            RpcMode::Remote => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_REMOTE,
            RpcMode::Puppet => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_PUPPET,
            RpcMode::RemoteSync => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_REMOTESYNC,
            RpcMode::Disabled => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_DISABLED,
            RpcMode::MasterSync => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_MASTERSYNC,
            RpcMode::PuppetSync => sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_PUPPETSYNC,
        }
    }
}

pub(crate) struct ScriptMethodAttributes {
    pub rpc_mode: RpcMode,
}

pub(crate) struct ScriptMethod<'l> {
    pub name: &'l str,
    pub method_ptr: Option<ScriptMethodFn>,
    pub attributes: ScriptMethodAttributes,

    pub method_data: *mut libc::c_void,
    pub free_func: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
}

/// Safe low-level trait for stateful, variadic methods that can be called on a native script type.
pub trait Method<C: NativeClass>: Send + Sync + 'static {
    /// Calls the method on `this` with `args`.
    fn call(&self, this: TInstance<'_, C>, args: Varargs<'_>) -> Variant;

    /// Returns an optional site where this method is defined. Used for logging errors in FFI wrappers.
    ///
    /// Default implementation returns `None`.
    #[inline]
    fn site() -> Option<Site<'static>> {
        None
    }
}

/// Wrapper for stateless methods that produces values with `Copy` and `Default`.
struct Stateless<F> {
    _marker: PhantomData<F>,
}

impl<C: NativeClass, F: Method<C> + Copy + Default> Method<C> for Stateless<F> {
    fn call(&self, this: TInstance<'_, C>, args: Varargs<'_>) -> Variant {
        let f = F::default();
        f.call(this, args)
    }
}

/// Adapter for methods whose arguments are statically determined. If the arguments would fail to
/// type check, the method will print the errors to Godot's debug console and return `null`.
#[derive(Clone, Copy, Default, Debug)]
pub struct StaticArgs<F> {
    f: F,
}

impl<F> StaticArgs<F> {
    /// Wrap `f` in an adapter that implements `Method`.
    #[inline]
    pub fn new(f: F) -> Self {
        StaticArgs { f }
    }
}

/// Trait for methods whose argument lists are known at compile time. Not to be confused with a
/// "static method".
pub trait StaticArgsMethod<C: NativeClass>: Send + Sync + 'static {
    type Args: FromVarargs;
    fn call(&self, this: TInstance<'_, C>, args: Self::Args) -> Variant;

    /// Returns an optional site where this method is defined. Used for logging errors in FFI wrappers.
    ///
    /// Default implementation returns `None`.
    #[inline]
    fn site() -> Option<Site<'static>> {
        None
    }
}

impl<C: NativeClass, F: StaticArgsMethod<C>> Method<C> for StaticArgs<F> {
    #[inline]
    fn call(&self, this: TInstance<'_, C>, mut args: Varargs<'_>) -> Variant {
        match args.read_many::<F::Args>() {
            Ok(parsed) => {
                if let Err(err) = args.done() {
                    err.with_site(F::site().unwrap_or_default()).log_error();
                    return Variant::nil();
                }
                F::call(&self.f, this, parsed)
            }
            Err(errors) => {
                for err in errors {
                    err.with_site(F::site().unwrap_or_default()).log_error();
                }
                Variant::nil()
            }
        }
    }

    #[inline]
    fn site() -> Option<Site<'static>> {
        F::site()
    }
}

/// Safe interface to a list of borrowed method arguments with a convenient API
/// for common operations with them.
///
/// **Note:** the `impl Iterator` is deprecated, `Varargs` itself should not be treated as a consumable iterator.
/// Instead, use [`Self::as_slice()`].
///
/// This type can be destructured into tuples:
/// ```no_run
/// use gdnative::prelude::*;
/// use gdnative::export::Varargs;
///
/// #[derive(NativeClass)]
/// #[no_constructor]
/// struct MyClass {}
///
/// struct CalcMethod;
/// impl Method<MyClass> for CalcMethod {
///     fn call(
///         &self,
///         _this: TInstance<'_, MyClass>,
///         args: Varargs<'_>,
///     ) -> Variant {
///         // Destructure via try_into():
///         let (a, b): (i64, i64) = args.try_into().expect("signature mismatch");
///         let ret = a * b;
///         ret.to_variant()
///     }
/// }
/// ```
pub struct Varargs<'a> {
    idx: usize,
    args: &'a [&'a Variant],
    offset_index: usize,
}

impl<'a> Varargs<'a> {
    /// Returns the amount of arguments left.
    #[inline]
    pub fn len(&self) -> usize {
        self.args.len() - self.idx
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a builder for reading the next argument, that can be used to refine
    /// the error message on failure.
    #[inline]
    pub fn read<T: FromVariant>(&mut self) -> ArgBuilder<'_, 'a, T> {
        ArgBuilder {
            args: self,
            name: None,
            ty: None,
            site: None,
            _marker: PhantomData,
        }
    }

    /// Parses a structure that implements `FromVarargs` incrementally from the
    /// remaining arguments.
    #[inline]
    pub fn read_many<T: FromVarargs>(&mut self) -> Result<T, Vec<ArgumentError<'a>>> {
        T::read(self)
    }

    /// Returns the remaining arguments as a slice of `Variant`s.
    #[inline]
    pub fn as_slice(&self) -> &'a [&'a Variant] {
        &self.args[self.idx..]
    }

    /// Discard the rest of the arguments, and return an error if there is any.
    ///
    /// # Errors
    ///
    /// If there are any excess arguments left.
    #[inline]
    pub fn done(self) -> Result<(), ArgumentError<'a>> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(ArgumentError {
                site: None,
                kind: ArgumentErrorKind::ExcessArguments {
                    rest: self.as_slice(),
                },
            })
        }
    }

    /// Create a typed interface from raw pointers. This is an internal interface.
    ///
    /// # Safety
    ///
    /// `args` must point to an array of valid `godot_variant` pointers of at least `num_args` long.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_sys(num_args: libc::c_int, args: *mut *mut sys::godot_variant) -> Self {
        let args = std::slice::from_raw_parts(args, num_args as usize);
        let args = std::mem::transmute::<&[*mut sys::godot_variant], &[&Variant]>(args);
        Self {
            idx: 0,
            args,
            offset_index: 0,
        }
    }

    /// Check the length of arguments.
    ///
    /// See [`Self::get()`] or [`Self::get_opt()`] for examples.
    ///
    /// # Errors
    /// Returns an [`VarargsError::InvalidLength`] if the length of arguments is outside the specified range.
    #[inline]
    pub fn check_length(&self, expected: impl Into<IndexBounds>) -> Result<(), VarargsError> {
        let passed = self.args.len();
        let expected = expected.into();
        if expected.contains(passed) {
            Ok(())
        } else {
            // Note: cannot use Box<dyn RangeBounds<usize>> because trait is not object-safe due to _unrelated_ method contains()
            Err(VarargsError::InvalidLength {
                length: passed,
                expected,
            })
        }
    }

    /// Returns the type-converted value at the specified argument position.
    ///
    /// # Errors
    /// Returns a [`VarargsError::InvalidArgumentType`] if the conversion fails or the argument is not set.
    ///
    /// # Examples
    /// ```
    /// # fn call(args: gdnative::export::Varargs) -> Result<(), Box<dyn std::error::Error>> {
    ///     args.check_length(2)?;
    ///     let a: usize = args.get(0)?;
    ///     let rest: i64 = args.get(1)?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get<T: FromVariant>(&self, index: usize) -> Result<T, VarargsError> {
        // Note: disregards iterator offset, since that representation is deprecated

        match self.args.get(index) {
            Some(v) => match T::from_variant(v) {
                Ok(ok) => Ok(ok),
                Err(error) => Err(VarargsError::InvalidArgumentType { index, error }),
            },
            None => {
                let error = FromVariantError::Custom("Argument is not set".to_owned());
                Err(VarargsError::InvalidArgumentType { index, error })
            }
        }
    }

    /// Returns the type-converted value at the specified argument position.
    /// Returns `None` if the argument is not set.
    ///
    /// # Errors
    /// Returns a [`VarargsError::InvalidArgumentType`] if the conversion fails.
    ///
    /// # Examples
    /// ```
    /// # fn call(args: gdnative::export::Varargs) -> Result<(), Box<dyn std::error::Error>> {
    ///     args.check_length(1..=2)?;
    ///     let a: usize = args.get(0)?;
    ///     let rest: i64 = args.get_opt(1)?.unwrap_or(72);
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn get_opt<T: FromVariant>(&self, index: usize) -> Result<Option<T>, VarargsError> {
        // Note: disregards iterator offset, since that representation is deprecated

        match self.args.get(index) {
            Some(v) => match T::from_variant(v) {
                Ok(ok) => Ok(Some(ok)),
                Err(error) => Err(VarargsError::InvalidArgumentType { index, error }),
            },
            None => Ok(None),
        }
    }
}

impl<'a> Iterator for Varargs<'a> {
    type Item = &'a Variant;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.args.get(self.idx);
        ret.map(|&v| {
            self.idx += 1;
            v
        })
    }
}

// Return a second token.
macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

// Count parameters.
macro_rules! count_tts {
    ($($tts:tt)*) => {
        0usize $(+ replace_expr!($tts 1usize))*
    };
}

// Convert from Varargs to tuples, implement traits.
macro_rules! varargs_into_tuple {
    ($($params:ident),*) => {
        impl<'a, $($params: FromVariant),*> std::convert::TryFrom<Varargs<'a>> for ($($params,)*) {
            type Error = VarargsError;

            #[inline]
            fn try_from(args: Varargs<'a>) -> Result<Self, Self::Error> {
                const EXPECTED: usize = count_tts!($($params)*);
                args.check_length(EXPECTED)?;
                let mut i: usize = 0;
                #[allow(unused_variables, unused_mut)]
                let mut inc = || {
                    let ret = i;
                    i += 1;
                    ret
                };
                Ok((
                    $(args.get::<$params>(inc())?,)*
                ))
            }
        }
    };
}

// Define up to the length supported by standard library.
varargs_into_tuple!();
varargs_into_tuple!(A);
varargs_into_tuple!(A, B);
varargs_into_tuple!(A, B, C);
varargs_into_tuple!(A, B, C, D);
varargs_into_tuple!(A, B, C, D, E);
varargs_into_tuple!(A, B, C, D, E, F);
varargs_into_tuple!(A, B, C, D, E, F, G);
varargs_into_tuple!(A, B, C, D, E, F, G, H);
varargs_into_tuple!(A, B, C, D, E, F, G, H, I);
varargs_into_tuple!(A, B, C, D, E, F, G, H, I, J);
varargs_into_tuple!(A, B, C, D, E, F, G, H, I, J, K);
varargs_into_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

/// All possible errors that can occur when converting from Varargs.
///
/// For methods that return this error, see [`Varargs::check_length()`], [`Varargs::get()`] or [`Varargs::get_opt()`].
/// Another context where this type is used is when destructuring `Varargs` into tuples.
#[derive(Debug)]
#[non_exhaustive]
pub enum VarargsError {
    /// At least one argument type mismatches.
    InvalidArgumentType {
        index: usize,
        error: FromVariantError,
    },
    /// Number of arguments doesn't match expectations.
    InvalidLength {
        /// The number of arguments actually passed.
        length: usize,
        expected: IndexBounds,
    },
}

impl std::error::Error for VarargsError {}

impl fmt::Display for VarargsError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarargsError::InvalidArgumentType { index, error } => {
                write!(f, "type error for argument #{index}: {error}")?
            }
            VarargsError::InvalidLength { expected, length } => write!(
                f,
                "length mismatch: expected range {expected}, actual {length}"
            )?,
        }

        Ok(())
    }
}

/// Defines which number of arguments is valid.
///
/// Convertible from an exact value `a` as well as inclusive range expressions `a..`, `..=b`, `a..=b`.
pub struct IndexBounds {
    /// The lower (inclusive) bound of the expected number of arguments, or `None` if unbounded.
    pub start: Option<usize>,

    /// The upper (inclusive) bound of the expected number of arguments, or `None` if unbounded.
    pub end: Option<usize>,
}

impl IndexBounds {
    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        match (self.start, self.end) {
            (Some(s), Some(e)) => value >= s && value <= e,
            (Some(s), None) => value >= s,
            (None, Some(e)) => value <= e,
            (None, None) => false, // unreachable in this context, but can be constructed by user
        }
    }
}

/// `a`
impl From<usize> for IndexBounds {
    #[inline]
    fn from(exact_value: usize) -> Self {
        Self {
            start: Some(exact_value),
            end: Some(exact_value),
        }
    }
}

/// `a..=b`
impl From<ops::RangeInclusive<usize>> for IndexBounds {
    #[inline]
    fn from(range: ops::RangeInclusive<usize>) -> Self {
        Self {
            start: Some(*range.start()),
            end: Some(*range.end()),
        }
    }
}

/// `a..`
impl From<ops::RangeFrom<usize>> for IndexBounds {
    #[inline]
    fn from(range: ops::RangeFrom<usize>) -> Self {
        Self {
            start: Some(range.start),
            end: None,
        }
    }
}

/// `..=b`
impl From<ops::RangeToInclusive<usize>> for IndexBounds {
    #[inline]
    fn from(range: ops::RangeToInclusive<usize>) -> Self {
        Self {
            start: None,
            end: Some(range.end),
        }
    }
}

impl fmt::Debug for IndexBounds {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IndexBounds({self})")
    }
}

impl fmt::Display for IndexBounds {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(start) = self.start {
            write!(f, "{start}")?
        }

        write!(f, "..=")?;

        if let Some(end) = self.end {
            write!(f, "{end}")?
        }

        Ok(())
    }
}

/// Trait for structures that can be parsed from `Varargs`.
///
/// This trait can be derived for structure types where each type implements `FromVariant`.
/// The order of fields matter for this purpose:
///
/// ```ignore
/// #[derive(FromVarargs)]
/// struct MyArgs {
///     foo: i32,
///     bar: String,
///     #[opt] baz: Option<Ref<Node>>,
/// }
/// ```
pub trait FromVarargs: Sized {
    fn read<'a>(args: &mut Varargs<'a>) -> Result<Self, Vec<ArgumentError<'a>>>;
}

/// Builder for providing additional argument information for error reporting.
pub struct ArgBuilder<'r, 'a, T> {
    args: &'r mut Varargs<'a>,
    name: Option<Cow<'a, str>>,
    ty: Option<Cow<'a, str>>,
    site: Option<Site<'a>>,
    _marker: PhantomData<T>,
}

impl<'r, 'a, T> ArgBuilder<'r, 'a, T> {
    /// Provides a name for this argument. If an old name is already set, it is
    /// silently replaced. The name can either be borrowed from the environment
    /// or owned.
    #[inline]
    pub fn with_name<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Provides a more readable type name for this argument. If an old name is
    /// already set, it is silently replaced. If no type name is given, a value
    /// from `std::any::type_name` is used. The name can either be borrowed from
    /// the environment or owned.
    #[inline]
    pub fn with_type_name<S: Into<Cow<'a, str>>>(mut self, ty: S) -> Self {
        self.ty = Some(ty.into());
        self
    }

    /// Provides a call site for this argument. If an old call site is already set,
    /// it is silently replaced. If given, the site will be used in case of error.
    #[inline]
    pub fn with_site(mut self, site: Site<'a>) -> Self {
        self.site = Some(site);
        self
    }
}

impl<'r, 'a, T: FromVariant> ArgBuilder<'r, 'a, T> {
    /// Get the converted argument value.
    ///
    /// # Errors
    ///
    /// If the argument is missing, or cannot be converted to the desired type.
    #[inline]
    pub fn get(mut self) -> Result<T, ArgumentError<'a>> {
        self.get_optional_internal().and_then(|arg| {
            let actual_index = self.args.idx + self.args.offset_index;
            arg.ok_or(ArgumentError {
                site: self.site,
                kind: ArgumentErrorKind::Missing {
                    idx: actual_index,
                    name: self.name,
                },
            })
        })
    }

    /// Get the argument as optional.
    ///
    /// # Errors
    ///
    /// If the argument is present, but cannot be converted to the desired type.
    #[inline]
    pub fn get_optional(mut self) -> Result<Option<T>, ArgumentError<'a>> {
        self.get_optional_internal()
    }

    fn get_optional_internal(&mut self) -> Result<Option<T>, ArgumentError<'a>> {
        let Self {
            site,
            args,
            name,
            ty,
            ..
        } = self;
        let actual_index = args.idx + args.offset_index;

        if let Some(arg) = args.next() {
            T::from_variant(arg).map(Some).map_err(|err| ArgumentError {
                site: *site,
                kind: ArgumentErrorKind::CannotConvert {
                    idx: actual_index,
                    name: name.take(),
                    value: arg,
                    ty: ty
                        .take()
                        .unwrap_or_else(|| Cow::Borrowed(std::any::type_name::<T>())),
                    err,
                },
            })
        } else {
            Ok(None)
        }
    }
}

/// Error during argument parsing.
#[derive(Debug)]
pub struct ArgumentError<'a> {
    site: Option<Site<'a>>,
    kind: ArgumentErrorKind<'a>,
}

impl<'a> std::error::Error for ArgumentError<'a> {}
impl<'a> fmt::Display for ArgumentError<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(site) = &self.site {
            write!(f, "at {site}: ")?;
        }
        write!(f, "{}", self.kind)
    }
}

impl<'a> ArgumentError<'a> {
    /// Assign a call site for this error. If an old one is already set, it is silently
    /// replaced.
    #[inline]
    pub fn with_site(mut self, site: Site<'a>) -> Self {
        self.site = Some(site);
        self
    }

    /// Print this error in the Godot debug console as a warning.
    ///
    /// # Panics
    ///
    /// If the API isn't initialized.
    #[inline]
    pub fn log_warn(&self) {
        crate::log::warn(self.site.unwrap_or_default(), &self.kind);
    }

    /// Print this error in the Godot debug console as an error.
    ///
    /// # Panics
    ///
    /// If the API isn't initialized.
    #[inline]
    pub fn log_error(&self) {
        crate::log::error(self.site.unwrap_or_default(), &self.kind);
    }
}

/// Error during argument parsing.
#[derive(Debug)]
enum ArgumentErrorKind<'a> {
    Missing {
        idx: usize,
        name: Option<Cow<'a, str>>,
    },
    CannotConvert {
        idx: usize,
        name: Option<Cow<'a, str>>,
        ty: Cow<'a, str>,
        value: &'a Variant,
        err: FromVariantError,
    },
    ExcessArguments {
        rest: &'a [&'a Variant],
    },
}

impl<'a> std::error::Error for ArgumentErrorKind<'a> {}

impl<'a> fmt::Display for ArgumentErrorKind<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ArgumentErrorKind as E;

        match self {
            E::Missing {
                idx,
                name: Some(name),
            } => {
                write!(f, "missing non-optional parameter `{name}` (#{idx})")
            }
            E::Missing { idx, name: None } => {
                write!(f, "missing non-optional parameter #{idx}")
            }
            E::CannotConvert {
                idx,
                name: Some(name),
                value,
                ty,
                err,
            } => {
                write!(f,
                    "cannot convert argument `{name}` (#{idx}, {value:?}) to {ty}: {err} (non-primitive types may impose structural checks)"
                )
            }
            E::CannotConvert {
                idx,
                name: None,
                value,
                ty,
                err,
            } => {
                write!(f,
                    "cannot convert argument #{idx} ({value:?}) to {ty}: {err} (non-primitive types may impose structural checks)"
                )
            }
            E::ExcessArguments { rest } => {
                if rest.len() > 1 {
                    write!(
                        f,
                        "{} excessive arguments are given: {:?}",
                        rest.len(),
                        rest
                    )
                } else {
                    write!(f, "an excessive argument is given: {:?}", rest[0])
                }
            }
        }
    }
}

unsafe extern "C" fn method_wrapper<C: NativeClass, F: Method<C>>(
    this: *mut sys::godot_object,
    method_data: *mut libc::c_void,
    user_data: *mut libc::c_void,
    num_args: libc::c_int,
    args: *mut *mut sys::godot_variant,
) -> sys::godot_variant {
    if user_data.is_null() {
        crate::log::error(
            F::site().unwrap_or_default(),
            format_args!(
                "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
                class_registry::class_name_or_default::<C>(),
            ),
        );
        return Variant::nil().leak();
    }

    let this = match std::ptr::NonNull::new(this) {
        Some(this) => this,
        None => {
            crate::log::error(
                F::site().unwrap_or_default(),
                format_args!(
                    "gdnative-core: base object pointer for {} is null (probably a bug in Godot)",
                    class_registry::class_name_or_default::<C>(),
                ),
            );
            return Variant::nil().leak();
        }
    };

    let result = std::panic::catch_unwind(move || {
        let method = &*(method_data as *const F);

        let this: Ref<C::Base, Shared> = Ref::from_sys(this);
        let this: TRef<'_, C::Base, _> = this.assume_safe_unchecked();
        let this: TInstance<'_, C, _> = TInstance::from_raw_unchecked(this, user_data);

        let args = Varargs::from_sys(num_args, args);

        F::call(method, this, args)
    });

    result
        .unwrap_or_else(|e| {
            crate::log::error(
                F::site().unwrap_or_default(),
                "gdnative-core: method panicked (check stderr for output)",
            );
            crate::private::print_panic_error(e);
            Variant::nil()
        })
        .leak()
}

unsafe extern "C" fn free_func<F>(method_data: *mut libc::c_void) {
    drop(Box::from_raw(method_data as *mut F))
}
