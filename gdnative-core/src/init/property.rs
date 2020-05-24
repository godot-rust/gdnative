//! Property registration.

// FIXME(#357): Temporary allow for ExportInfo fields.
#![allow(deprecated)]

use std::mem;

use crate::get_api;
use crate::object::GodotObject;
use crate::GodotString;
use crate::NativeClass;
use crate::ToVariant;
use crate::VariantType;

use super::ClassBuilder;

mod accessor;
pub mod hint;

pub use hint::*;

use accessor::{Getter, InvalidGetter, InvalidSetter, RawGetter, RawSetter, Setter};

/// Trait for exportable types.
pub trait Export: ToVariant {
    /// A type-specific hint type that is valid for the type being exported.
    type Hint;

    /// Returns `ExportInfo` given an optional typed hint.
    fn export_info(hint: Option<Self::Hint>) -> ExportInfo;
}

/// Metadata about the exported property.
#[derive(Debug)]
pub struct ExportInfo {
    #[deprecated(
        since = "0.8.1",
        note = "Fields of ExportInfo will become private in 0.9. Use one of the constructors or Export::export_info instead."
    )]
    pub variant_type: VariantType,

    #[deprecated(
        since = "0.8.1",
        note = "Fields of ExportInfo will become private in 0.9. Use one of the constructors or Export::export_info instead."
    )]
    pub hint_kind: sys::godot_property_hint,

    #[deprecated(
        since = "0.8.1",
        note = "Fields of ExportInfo will become private in 0.9. Use one of the constructors or Export::export_info instead."
    )]
    pub hint_string: GodotString,
}

impl ExportInfo {
    /// Create an `ExportInfo` with the given Variant type, but without a hint.
    pub fn new(variant_type: VariantType) -> Self {
        ExportInfo {
            variant_type,
            hint_kind: sys::godot_property_hint_GODOT_PROPERTY_HINT_NONE,
            hint_string: GodotString::new(),
        }
    }

    /// Create an `ExportInfo` with a hint for a specific Godot resource type.
    pub fn resource_type<T>() -> Self
    where
        T: GodotObject,
    {
        ExportInfo {
            variant_type: VariantType::Object,
            hint_kind: sys::godot_property_hint_GODOT_PROPERTY_HINT_RESOURCE_TYPE,
            hint_string: T::class_name().into(),
        }
    }
}

/// Builder type used to register a property on a `NativeClass`.
#[derive(Debug)]
#[must_use]
pub struct PropertyBuilder<'a, C, T: Export, S = InvalidSetter<'a>, G = InvalidGetter<'a>> {
    name: &'a str,
    setter: S,
    getter: G,
    default: Option<T>,
    hint: Option<T::Hint>,
    usage: Usage,
    class_builder: &'a ClassBuilder<C>,
}

impl<'a, C, T> PropertyBuilder<'a, C, T, InvalidSetter<'a>, InvalidGetter<'a>>
where
    C: NativeClass,
    T: Export,
{
    /// Creates a new `PropertyBuilder` with the given property name.
    pub(super) fn new(class_builder: &'a ClassBuilder<C>, name: &'a str) -> Self {
        PropertyBuilder {
            name,
            setter: InvalidSetter::new(name),
            getter: InvalidGetter::new(name),
            default: None,
            hint: None,
            usage: Usage::DEFAULT,
            class_builder,
        }
    }
}

impl<'a, C, T, S, G> PropertyBuilder<'a, C, T, S, G>
where
    C: NativeClass,
    T: Export,
    S: RawSetter<C, T>,
    G: RawGetter<C, T>,
{
    /// Register the property built with this builder.
    pub fn done(self) {
        let ExportInfo {
            variant_type,
            hint_kind,
            hint_string,
        } = T::export_info(self.hint);
        let default = self.default.to_variant();

        let mut attr = sys::godot_property_attributes {
            rset_type: sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_DISABLED, // TODO:
            type_: variant_type as sys::godot_int,
            hint: hint_kind,
            hint_string: hint_string.to_sys(),
            usage: self.usage.to_sys(),
            default_value: default.to_sys(),
        };

        let path = ::std::ffi::CString::new(self.name).unwrap();

        let set = unsafe { self.setter.as_godot_function() };
        let get = unsafe { self.getter.as_godot_function() };

        unsafe {
            (get_api().godot_nativescript_register_property)(
                self.class_builder.init_handle,
                self.class_builder.class_name.as_ptr(),
                path.as_ptr() as *const _,
                &mut attr,
                set,
                get,
            );
        }
    }

    /// Provides a setter function with the signature `fn(&mut C, owner: C::Base, value: T)`
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    pub fn with_setter<NS>(
        self,
        setter: NS,
    ) -> PropertyBuilder<'a, C, T, Setter<accessor::Mut, NS>, G>
    where
        Setter<accessor::Mut, NS>: RawSetter<C, T>,
    {
        PropertyBuilder {
            name: self.name,
            setter: Setter::new(setter),
            getter: self.getter,
            default: self.default,
            hint: self.hint,
            usage: self.usage,
            class_builder: self.class_builder,
        }
    }

    /// Provides a setter function with the signature `fn(&C, owner: C::Base, value: T)`
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    pub fn with_shr_setter<NS>(
        self,
        setter: NS,
    ) -> PropertyBuilder<'a, C, T, Setter<accessor::Shr, NS>, G>
    where
        Setter<accessor::Shr, NS>: RawSetter<C, T>,
    {
        PropertyBuilder {
            name: self.name,
            setter: Setter::new(setter),
            getter: self.getter,
            default: self.default,
            hint: self.hint,
            usage: self.usage,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&C, owner: C::Base) -> T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    pub fn with_getter<NG>(
        self,
        getter: NG,
    ) -> PropertyBuilder<'a, C, T, S, Getter<accessor::Shr, accessor::Owned, NG>>
    where
        Getter<accessor::Shr, accessor::Owned, NG>: RawGetter<C, T>,
    {
        PropertyBuilder {
            name: self.name,
            setter: self.setter,
            getter: Getter::new(getter),
            default: self.default,
            hint: self.hint,
            usage: self.usage,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&C, owner: C::Base) -> &T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    pub fn with_ref_getter<NG>(
        self,
        getter: NG,
    ) -> PropertyBuilder<'a, C, T, S, Getter<accessor::Shr, accessor::Ref, NG>>
    where
        Getter<accessor::Shr, accessor::Ref, NG>: RawGetter<C, T>,
    {
        PropertyBuilder {
            name: self.name,
            setter: self.setter,
            getter: Getter::new(getter),
            default: self.default,
            hint: self.hint,
            usage: self.usage,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&mut C, owner: C::Base) -> T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    pub fn with_mut_getter<NG>(
        self,
        getter: NG,
    ) -> PropertyBuilder<'a, C, T, S, Getter<accessor::Mut, accessor::Owned, NG>>
    where
        Getter<accessor::Mut, accessor::Owned, NG>: RawGetter<C, T>,
    {
        PropertyBuilder {
            name: self.name,
            setter: self.setter,
            getter: Getter::new(getter),
            default: self.default,
            hint: self.hint,
            usage: self.usage,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&mut C, owner: C::Base) -> &T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    pub fn with_mut_ref_getter<NG>(
        self,
        getter: NG,
    ) -> PropertyBuilder<'a, C, T, S, Getter<accessor::Mut, accessor::Ref, NG>>
    where
        Getter<accessor::Mut, accessor::Ref, NG>: RawGetter<C, T>,
    {
        PropertyBuilder {
            name: self.name,
            setter: self.setter,
            getter: Getter::new(getter),
            default: self.default,
            hint: self.hint,
            usage: self.usage,
            class_builder: self.class_builder,
        }
    }

    /// Sets a default value for the property as a hint to the editor. The setter may or may not
    /// be actually called with this value.
    pub fn with_default(mut self, default: T) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets an editor hint.
    pub fn with_hint(mut self, hint: T::Hint) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Sets a property usage.
    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = usage;
        self
    }
}

bitflags! {
    pub struct Usage: u32 {
        const STORAGE = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_STORAGE as u32;
        const EDITOR = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_EDITOR as u32;
        const NETWORK = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_NETWORK as u32;
        const EDITOR_HELPER = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_EDITOR_HELPER as u32;
        const CHECKABLE = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_CHECKABLE as u32;
        const CHECKED = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_CHECKED as u32;
        const INTERNATIONALIZED = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_INTERNATIONALIZED as u32;
        const GROUP = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_GROUP as u32;
        const CATEGORY = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_CATEGORY as u32;
        const STORE_IF_NONZERO = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_STORE_IF_NONZERO as u32;
        const STORE_IF_NONONE = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_STORE_IF_NONONE as u32;
        const NO_INSTANCE_STATE = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_NO_INSTANCE_STATE as u32;
        const RESTART_IF_CHANGED = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_RESTART_IF_CHANGED as u32;
        const SCRIPT_VARIABLE  = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_SCRIPT_VARIABLE as u32;
        const STORE_IF_NULL = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_STORE_IF_NULL as u32;
        const ANIMATE_AS_TRIGGER = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_ANIMATE_AS_TRIGGER as u32;
        const UPDATE_ALL_IF_MODIFIED = sys::godot_property_usage_flags_GODOT_PROPERTY_USAGE_UPDATE_ALL_IF_MODIFIED as u32;

        const DEFAULT = Self::STORAGE.bits | Self::EDITOR.bits | Self::NETWORK.bits as u32;
        const DEFAULT_INTL = Self::DEFAULT.bits | Self::INTERNATIONALIZED.bits as u32;
        const NOEDITOR = Self::STORAGE.bits | Self::NETWORK.bits as u32;
    }
}

impl Usage {
    pub fn to_sys(&self) -> sys::godot_property_usage_flags {
        unsafe { mem::transmute(*self) }
    }
}

mod impl_export {
    use super::*;
    use crate::*;

    macro_rules! impl_export_for_int {
        ($ty:ident) => {
            impl Export for $ty {
                type Hint = hint::IntHint<$ty>;
                fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
                    hint.map_or_else(
                        || ExportInfo::new(VariantType::I64),
                        Self::Hint::export_info,
                    )
                }
            }
        };
    }

    impl_export_for_int!(i8);
    impl_export_for_int!(i16);
    impl_export_for_int!(i32);
    impl_export_for_int!(i64);
    impl_export_for_int!(u8);
    impl_export_for_int!(u16);
    impl_export_for_int!(u32);
    impl_export_for_int!(u64);

    macro_rules! impl_export_for_float {
        ($ty:ident) => {
            impl Export for $ty {
                type Hint = hint::FloatHint<$ty>;
                fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
                    hint.map_or_else(
                        || ExportInfo::new(VariantType::F64),
                        Self::Hint::export_info,
                    )
                }
            }
        };
    }

    impl_export_for_float!(f32);
    impl_export_for_float!(f64);

    macro_rules! impl_export_for_string {
        ($ty:ty) => {
            impl Export for $ty {
                type Hint = hint::StringHint;
                fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
                    hint.map_or_else(
                        || ExportInfo::new(VariantType::GodotString),
                        Self::Hint::export_info,
                    )
                }
            }
        };
    }

    impl_export_for_string!(GodotString);
    impl_export_for_string!(String);

    macro_rules! impl_export_for_core_type_without_hint {
        ($ty:ty: $variant_ty:ident) => {
            impl Export for $ty {
                type Hint = ();
                fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
                    ExportInfo::new(VariantType::$variant_ty)
                }
            }
        };
        ($ty:ident) => {
            impl_export_for_core_type_without_hint!($ty: $ty);
        };
    }

    impl_export_for_core_type_without_hint!(bool: Bool);
    impl_export_for_core_type_without_hint!(Vector2);
    impl_export_for_core_type_without_hint!(Rect2);
    impl_export_for_core_type_without_hint!(Vector3);
    impl_export_for_core_type_without_hint!(Transform2D);
    impl_export_for_core_type_without_hint!(Plane);
    impl_export_for_core_type_without_hint!(Quat);
    impl_export_for_core_type_without_hint!(Aabb);
    impl_export_for_core_type_without_hint!(Basis);
    impl_export_for_core_type_without_hint!(Transform);
    impl_export_for_core_type_without_hint!(NodePath);
    impl_export_for_core_type_without_hint!(Rid);
    impl_export_for_core_type_without_hint!(Dictionary);
    impl_export_for_core_type_without_hint!(VariantArray);
    impl_export_for_core_type_without_hint!(ByteArray);
    impl_export_for_core_type_without_hint!(Int32Array);
    impl_export_for_core_type_without_hint!(Float32Array);
    impl_export_for_core_type_without_hint!(StringArray);
    impl_export_for_core_type_without_hint!(Vector2Array);
    impl_export_for_core_type_without_hint!(Vector3Array);
    impl_export_for_core_type_without_hint!(ColorArray);

    impl Export for Color {
        type Hint = hint::ColorHint;
        fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
            hint.map_or_else(
                || ExportInfo::new(VariantType::Color),
                Self::Hint::export_info,
            )
        }
    }

    impl<T> Export for T
    where
        T: GodotObject + ToVariant,
    {
        type Hint = ();
        fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
            ExportInfo::resource_type::<T>()
        }
    }

    impl<T> Export for Instance<T>
    where
        T: NativeClass,
        T::Base: ToVariant,
    {
        type Hint = ();
        fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
            ExportInfo::resource_type::<T::Base>()
        }
    }

    impl<T> Export for Option<T>
    where
        T: Export,
    {
        type Hint = T::Hint;
        fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
            T::export_info(hint)
        }
    }
}
