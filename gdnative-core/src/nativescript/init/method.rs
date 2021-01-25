//! Method registration

// Temporary for unsafe method registration
#![allow(deprecated)]

use std::fmt;
use std::marker::PhantomData;

use crate::core_types::{FromVariant, FromVariantError, Variant};
use crate::nativescript::class::{NativeClass, RefInstance};
use crate::object::{Ref, TRef};
use crate::thread_access::Shared;

use super::ClassBuilder;

/// Builder type used to register a method on a `NativeClass`.
pub struct MethodBuilder<'a, C, F> {
    class_builder: &'a super::ClassBuilder<C>,
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

        self.class_builder.add_method_advanced(script_method);
    }
}

impl<'a, C, F> MethodBuilder<'a, C, F>
where
    C: NativeClass,
    F: Method<C> + Copy + Default,
{
    /// Register the method as a stateless method. Stateless methods do not have data
    /// pointers and destructors and is thus slightly lighter. This is intended for ZSTs,
    /// but can be used with any `Method` type with `Copy + Default`.
    #[inline]
    pub fn done_stateless(self) {
        let script_method = ScriptMethod {
            name: self.name,
            method_ptr: Some(method_wrapper::<C, Stateless<F>>),
            attributes: ScriptMethodAttributes {
                rpc_mode: self.rpc_mode,
            },
            method_data: 1 as *mut libc::c_void,
            free_func: None,
        };

        self.class_builder.add_method_advanced(script_method);
    }
}

#[deprecated(
    note = "Unsafe registration is deprecated. Use the safe, higher-level `MethodBuilder` API instead."
)]
pub type ScriptMethodFn = unsafe extern "C" fn(
    *mut sys::godot_object,
    *mut libc::c_void,
    *mut libc::c_void,
    libc::c_int,
    *mut *mut sys::godot_variant,
) -> sys::godot_variant;

pub enum RpcMode {
    Disabled,
    Remote,
    RemoteSync,
    Master,
    Puppet,
    MasterSync,
    PuppetSync,
}

impl Default for RpcMode {
    #[inline]
    fn default() -> Self {
        RpcMode::Disabled
    }
}

#[deprecated(
    note = "Unsafe registration is deprecated. Use the safe, higher-level `MethodBuilder` API instead."
)]
pub struct ScriptMethodAttributes {
    pub rpc_mode: RpcMode,
}

#[deprecated(
    note = "Unsafe registration is deprecated. Use the safe, higher-level `MethodBuilder` API instead."
)]
pub struct ScriptMethod<'l> {
    pub name: &'l str,
    pub method_ptr: Option<ScriptMethodFn>,
    pub attributes: ScriptMethodAttributes,

    pub method_data: *mut libc::c_void,
    pub free_func: Option<unsafe extern "C" fn(*mut libc::c_void) -> ()>,
}

/// Safe low-level trait for stateful, variadic methods that can be called on a native script type.
pub trait Method<C: NativeClass>: Send + Sync + 'static {
    fn call(&self, this: RefInstance<'_, C, Shared>, args: Varargs<'_>) -> Variant;
}

/// Wrapper for stateless methods that produces values with `Copy` and `Default`.
struct Stateless<F> {
    _marker: PhantomData<F>,
}

impl<C: NativeClass, F: Method<C> + Copy + Default> Method<C> for Stateless<F> {
    fn call(&self, this: RefInstance<'_, C, Shared>, args: Varargs<'_>) -> Variant {
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
    fn call(&self, this: RefInstance<'_, C, Shared>, args: Self::Args) -> Variant;
}

impl<C: NativeClass, F: StaticArgsMethod<C>> Method<C> for StaticArgs<F> {
    #[inline]
    fn call(&self, this: RefInstance<'_, C, Shared>, mut args: Varargs<'_>) -> Variant {
        match args.read_many::<F::Args>() {
            Ok(parsed) => {
                if let Err(err) = args.done() {
                    godot_error!("{}", err);
                    return Variant::new();
                }
                F::call(&self.f, this, parsed)
            }
            Err(errors) => {
                for err in errors {
                    godot_error!("{}", err);
                }
                Variant::new()
            }
        }
    }
}

/// Safe interface to a list of borrowed method arguments with a convenient API
/// for common operations with them. Can also be used as an iterator.
pub struct Varargs<'a> {
    idx: usize,
    iter: std::slice::Iter<'a, &'a Variant>,
}

impl<'a> Varargs<'a> {
    /// Returns the amount of arguments left.
    #[inline]
    pub fn len(&self) -> usize {
        self.iter.len()
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
        self.iter.as_slice()
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
            Err(ArgumentError::ExcessArguments {
                rest: self.as_slice(),
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
            iter: args.iter(),
        }
    }
}

impl<'a> Iterator for Varargs<'a> {
    type Item = &'a Variant;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().copied()
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
    name: Option<&'a str>,
    ty: Option<&'a str>,
    _marker: PhantomData<T>,
}

impl<'r, 'a, T> ArgBuilder<'r, 'a, T> {
    /// Provides a name for this argument. If an old name is already set, it is
    /// silently replaced.
    #[inline]
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Provides a more readable type name for this argument. If an old name is
    /// already set, it is silently replaced. If no type name is given, a value
    /// from `std::any::type_name` is used.
    #[inline]
    pub fn with_type_name(mut self, ty: &'a str) -> Self {
        self.ty = Some(ty);
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
    pub fn get(self) -> Result<T, ArgumentError<'a>> {
        let name = self.name;
        let idx = self.args.idx;

        self.get_optional()
            .and_then(|arg| arg.ok_or(ArgumentError::Missing { idx, name }))
    }

    /// Get the argument as optional.
    ///
    /// # Errors
    ///
    /// If the argument is present, but cannot be converted to the desired type.
    #[inline]
    pub fn get_optional(self) -> Result<Option<T>, ArgumentError<'a>> {
        let Self { args, name, ty, .. } = self;
        let idx = args.idx;

        if let Some(arg) = args.iter.next() {
            args.idx += 1;
            T::from_variant(arg)
                .map(Some)
                .map_err(|err| ArgumentError::CannotConvert {
                    idx,
                    name,
                    value: arg,
                    ty: ty.unwrap_or_else(|| std::any::type_name::<T>()),
                    err,
                })
        } else {
            Ok(None)
        }
    }
}

/// Error during argument parsing.
#[derive(Debug)]
pub enum ArgumentError<'a> {
    Missing {
        idx: usize,
        name: Option<&'a str>,
    },
    CannotConvert {
        idx: usize,
        name: Option<&'a str>,
        ty: &'a str,
        value: &'a Variant,
        err: FromVariantError,
    },
    ExcessArguments {
        rest: &'a [&'a Variant],
    },
}

impl<'a> std::error::Error for ArgumentError<'a> {}

impl<'a> fmt::Display for ArgumentError<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ArgumentError as E;

        match self {
            E::Missing {
                idx,
                name: Some(name),
            } => {
                write!(f, "missing non-optional parameter `{}` (#{})", name, idx)
            }
            E::Missing { idx, name: None } => {
                write!(f, "missing non-optional parameter #{}", idx)
            }
            E::CannotConvert {
                idx,
                name: Some(name),
                value,
                ty,
                err,
            } => {
                write!(f,
                    "cannot convert argument `{}` (#{}, {:?}) to {}: {} (non-primitive types may impose structural checks)",
                    name, idx, value, ty, err
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
                    "cannot convert argument #{} ({:?}) to {}: {} (non-primitive types may impose structural checks)",
                    idx, value, ty, err
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
        godot_error!(
            "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
            C::class_name(),
        );
        return Variant::new().forget();
    }

    let this = match std::ptr::NonNull::new(this) {
        Some(this) => this,
        None => {
            godot_error!(
                "gdnative-core: base object pointer for {} is null (probably a bug in Godot)",
                C::class_name(),
            );
            return Variant::new().forget();
        }
    };

    let result = std::panic::catch_unwind(move || {
        let method = &*(method_data as *const F);

        let this: Ref<C::Base, Shared> = Ref::from_sys(this);
        let this: TRef<'_, C::Base, _> = this.assume_safe_unchecked();
        let this: RefInstance<'_, C, _> = RefInstance::from_raw_unchecked(this, user_data);

        let args = Varargs::from_sys(num_args, args);

        F::call(method, this, args)
    });

    result
        .unwrap_or_else(|_| {
            godot_error!("gdnative-core: method panicked (check stderr for output)");
            Variant::new()
        })
        .forget()
}

unsafe extern "C" fn free_func<F>(method_data: *mut libc::c_void) {
    drop(Box::from_raw(method_data as *mut F))
}
