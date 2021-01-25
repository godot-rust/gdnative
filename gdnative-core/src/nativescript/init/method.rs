use std::fmt;
use std::marker::PhantomData;

use crate::core_types::{FromVariant, FromVariantError, Variant};

/// Interface to a list of borrowed method arguments with a convenient interface
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

/// Builder for providing additional argument information for error reporting.
pub struct ArgBuilder<'r, 'a, T> {
    args: &'r mut Varargs<'a>,
    name: Option<&'r str>,
    ty: Option<&'r str>,
    _marker: PhantomData<T>,
}

impl<'r, 'a, T> ArgBuilder<'r, 'a, T> {
    /// Provides a name for this argument. If an old name is already set, it is
    /// silently replaced.
    #[inline]
    pub fn with_name(mut self, name: &'r str) -> Self {
        self.name = Some(name);
        self
    }

    /// Provides a more readable type name for this argument. If an old name is
    /// already set, it is silently replaced. If no type name is given, a value
    /// from `std::any::type_name` is used.
    #[inline]
    pub fn with_type_name(mut self, ty: &'r str) -> Self {
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
    pub fn get(self) -> Result<T, ArgumentError<'r>> {
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
    pub fn get_optional(self) -> Result<Option<T>, ArgumentError<'r>> {
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
