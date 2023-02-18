use std::fmt::{Debug, Display};
use std::sync::Arc;

use crate::export::user_data::{ArcData, Map, MapMut, MapOwned};
use crate::export::NativeClass;
use crate::object::ownership::Shared;

use super::{Instance, TInstance};

/// Trait for types that can be used as the `self` or `#[self]` argument (receiver) for methods
/// exported through the `#[methods]` attribute macro. This trait has no public interface, and
/// is not intended to be implemented by users.
///
/// Notably, this is implemented for [`Instance`] and [`TInstance`] along with the usual `self`
/// reference types. For types using [`ArcData`] as the wrapper specifically, [`Arc<C>`] is also
/// allowed.
///
/// The trait is unsealed for technical coherence issues, but is not intended to be implemented
/// by users. Changes to the definition of this trait are not considered breaking changes under
/// semver.
pub trait Receiver<C: NativeClass>: Sized {
    #[doc(hidden)]
    type This<'a>;
    #[doc(hidden)]
    type Err: std::error::Error;

    #[doc(hidden)]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R;
}

/// Error type indicating that an operation can't fail.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[allow(clippy::exhaustive_enums)] // explicitly uninhabited
pub enum Infallible {}

impl std::error::Error for Infallible {}
impl Display for Infallible {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!("uninhabited enum")
    }
}

impl<'r, C: NativeClass> Receiver<C> for TInstance<'r, C, Shared> {
    type This<'a> = TInstance<'a, C, Shared>;
    type Err = Infallible;

    #[inline]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R,
    {
        Ok(f(instance))
    }
}

impl<C: NativeClass> Receiver<C> for Instance<C, Shared> {
    type This<'a> = Instance<C, Shared>;
    type Err = Infallible;

    #[inline]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R,
    {
        Ok(f(instance.claim()))
    }
}

impl<'r, C: NativeClass> Receiver<C> for &'r C
where
    C::UserData: Map,
{
    type This<'a> = &'a C;
    type Err = <C::UserData as Map>::Err;

    #[inline]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R,
    {
        instance.map(|this, _| f(this))
    }
}

impl<'r, C: NativeClass> Receiver<C> for &'r mut C
where
    C::UserData: MapMut,
{
    type This<'a> = &'a mut C;
    type Err = <C::UserData as MapMut>::Err;

    #[inline]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R,
    {
        instance.map_mut(|this, _| f(this))
    }
}

impl<C: NativeClass> Receiver<C> for C
where
    C::UserData: MapOwned,
{
    type This<'a> = C;
    type Err = <C::UserData as MapOwned>::Err;

    #[inline]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R,
    {
        instance.map_owned(|this, _| f(this))
    }
}

impl<C> Receiver<C> for Arc<C>
where
    C: NativeClass<UserData = ArcData<C>>,
{
    type This<'a> = Arc<C>;
    type Err = Infallible;

    #[inline]
    fn with_instance<F, R>(instance: TInstance<'_, C, Shared>, f: F) -> Result<R, Self::Err>
    where
        F: for<'a> FnOnce(Self::This<'a>) -> R,
    {
        let (_, script) = instance.claim().decouple();
        Ok(f(script.into_inner()))
    }
}
