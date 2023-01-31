//! Property registration.

// For the `PropertyUsage` bitflags declaration. The attribute doesn't work above the macro
// invocation.
#![allow(clippy::unnecessary_cast)]

use std::marker::PhantomData;

use accessor::{Getter, RawGetter, RawSetter, Setter};
use invalid_accessor::{InvalidGetter, InvalidSetter};

use crate::core_types::*;
use crate::export::{ClassBuilder, NativeClass};
use crate::object::ownership::Shared;
use crate::object::{GodotObject, Instance, Ref};
use crate::private::get_api;

use super::RpcMode;

mod accessor;
mod invalid_accessor;

pub mod hint;

/// Trait for exportable types.
///
/// ## Rust collections
///
/// `Export` is intentionally unimplemented for standard Rust collections, such as [`Vec`] or
/// [`HashMap`][std::collections::HashMap]. The reason is that such types exhibit surprising
/// behavior when used from GDScript, due to how [`ToVariant`]/[`FromVariant`] conversions work
/// for these types.
///
/// Godot has no concept of Rust collections, and cannot operate on them. Whenever a standard
/// collection is converted to [`Variant`] via [`ToVariant`], what actually happens is that:
///
/// - First, a new Godot collection of the corresponding "kind" is allocated.
/// - Then, the Rust collection is iterated over, and each element is converted and inserted into
///   the new collection, possibly triggering many more allocations in the process.
///
/// With properties, this whole process happens anew *with each access to the property*, which
/// means that:
///
/// - Modifying such properties from the remote debugger, or calling methods on the property
///   directly from GDScript (e.g. `thing.exported_vec.append("foo")`) do not produce the desired
///   behavior by the user.
/// - Seemingly innocuous expressions such as
///   `thing.exported_vec[0] + thing.exported_vec[1] + thing.exported_vec[2]` can be much more
///   expensive computationally than what the user would expect.
///
/// As such, we do not allow these types to be exported as properties directly as a precaution.
/// If you wish to export collections to GDScript, consider the following options:
///
/// - Exporting a [`Variant`] collection such as [`VariantArray`] or [`Dictionary`] explicitly,
///   embracing their respective semantics.
/// - Exporting not a property, but methods that have to be explicitly called, to set clear
///   expectations that the return value might be expensive to produce.
pub trait Export: crate::core_types::ToVariant {
    /// A type-specific hint type that is valid for the type being exported.
    ///
    /// If this type shows up as `NoHint`, a private, uninhabitable type indicating
    /// that there are no hints available for the time being, users *must* use `None`
    /// for properties of this type. This ensures that it will not be a breaking change
    /// to add a hint for the type later, since it supports no operations and cannot
    /// be named directly in user code.
    type Hint;

    /// Returns `ExportInfo` given an optional typed hint.
    fn export_info(hint: Option<Self::Hint>) -> ExportInfo;
}

/// Metadata about the exported property.
#[derive(Debug)]
pub struct ExportInfo {
    pub(super) variant_type: VariantType,
    pub(super) hint_kind: sys::godot_property_hint,
    pub(super) hint_string: GodotString,
}

impl ExportInfo {
    /// Create an `ExportInfo` with the given Variant type, but without a hint.
    #[inline]
    pub fn new(variant_type: VariantType) -> Self {
        ExportInfo {
            variant_type,
            hint_kind: sys::godot_property_hint_GODOT_PROPERTY_HINT_NONE,
            hint_string: GodotString::new(),
        }
    }

    /// Create an `ExportInfo` with a hint for a specific Godot resource type.
    #[inline]
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
#[must_use = "PropertyBuilder left unbuilt -- did you forget to call done()?"]
pub struct PropertyBuilder<'a, C, T: Export, S = InvalidSetter<'a>, G = InvalidGetter<'a>> {
    name: &'a str,
    setter: S,
    getter: G,
    default: Option<T>,
    hint: Option<T::Hint>,
    usage: PropertyUsage,
    rpc_mode: RpcMode,
    class_builder: &'a ClassBuilder<C>,
}

impl<'a, C, T> PropertyBuilder<'a, C, T, InvalidSetter<'a>, InvalidGetter<'a>>
where
    C: NativeClass,
    T: Export,
{
    /// Creates a new `PropertyBuilder` with the given property name.
    #[inline]
    pub(super) fn new(class_builder: &'a ClassBuilder<C>, name: &'a str) -> Self {
        PropertyBuilder {
            name,
            setter: InvalidSetter::new(name),
            getter: InvalidGetter::new(name),
            default: None,
            hint: None,
            usage: PropertyUsage::DEFAULT,
            rpc_mode: RpcMode::Disabled,
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
    #[inline]
    pub fn done(self) {
        let ExportInfo {
            variant_type,
            hint_kind,
            hint_string,
        } = T::export_info(self.hint);
        let default = self.default.to_variant();

        let mut attr = sys::godot_property_attributes {
            rset_type: self.rpc_mode.sys(),
            type_: variant_type as sys::godot_int,
            hint: hint_kind,
            hint_string: hint_string.to_sys(),
            usage: self.usage.to_sys(),
            default_value: default.to_sys(),
        };

        let path = ::std::ffi::CString::new(self.name).unwrap();

        let set = unsafe { self.setter.into_godot_function() };
        let get = unsafe { self.getter.into_godot_function() };

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
    #[inline]
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
            rpc_mode: self.rpc_mode,
            class_builder: self.class_builder,
        }
    }

    /// Provides a setter function with the signature `fn(&C, owner: C::Base, value: T)`
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    ///
    /// "shr" stands for "shared reference", as opposed to the more common `&mut self`.
    #[inline]
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
            rpc_mode: self.rpc_mode,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&C, owner: C::Base) -> T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    #[inline]
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
            rpc_mode: self.rpc_mode,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&C, owner: C::Base) -> &T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    #[inline]
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
            rpc_mode: self.rpc_mode,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&mut C, owner: C::Base) -> T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    #[inline]
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
            rpc_mode: self.rpc_mode,
            class_builder: self.class_builder,
        }
    }

    /// Provides a getter function with the signature `fn(&mut C, owner: C::Base) -> &T`,
    /// where `C` is the `NativeClass` type being registered and `T` is the type of the property.
    #[inline]
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
            rpc_mode: self.rpc_mode,
            class_builder: self.class_builder,
        }
    }

    /// Sets a default value for the property as a hint to the editor. The setter may or may not
    /// be actually called with this value.
    #[inline]
    pub fn with_default(mut self, default: T) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets an editor hint.
    #[inline]
    pub fn with_hint(mut self, hint: T::Hint) -> Self {
        self.hint = Some(hint);
        self
    }

    /// Sets a property usage.
    #[inline]
    pub fn with_usage(mut self, usage: PropertyUsage) -> Self {
        self.usage = usage;
        self
    }

    /// Sets a RPC mode.
    #[inline]
    pub fn with_rpc_mode(mut self, rpc_mode: RpcMode) -> Self {
        self.rpc_mode = rpc_mode;
        self
    }
}

bitflags::bitflags! {
    pub struct PropertyUsage: u32 {
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

impl PropertyUsage {
    #[inline]
    pub fn to_sys(self) -> sys::godot_property_usage_flags {
        self.bits() as sys::godot_property_usage_flags
    }
}

/// Placeholder type for exported properties with no backing field.
///
/// This is the go-to type whenever you want to expose a getter/setter to GDScript, which
/// does not directly map to a field in your struct. Instead of adding a useless field
/// of the corresponding type (which needs initialization, extra space, etc.), you can use
/// an instance of this type as a placeholder.
///
/// `Property` is a zero-sized type (ZST) which has exactly one value: `Property::default()`.
/// It implements most of the basic traits, which allows its enclosing struct to remain
/// composable and derive those traits itself.
///
/// ## When to use `Property<T>` instead of `T`
///
/// The following table shows which combinations of `#[property]` attributes and field types are allowed.
/// In this context, `get` and `set` behave symmetrically, so only one of the combinations is listed.
/// Furthermore, `get_ref` can be used in place of `get`, when it appears with a path.
///
/// Field type ➡ <br> Attributes ⬇           | bare `T`                      | `Property<T>`
/// ------------------------------------------|-------------------------------|-----------------------------
/// `#[property]`                             | ✔️ default get + set       | ❌️
/// `#[property(get, set)]` _(same as above)_ | ✔️ default get + set       | ❌️
/// `#[property(get)]`                        | ✔️ default get (no set)    | ❌️
/// `#[property(get="path")]`                 | ⚠️ custom get (no set)     | ✔️ custom get (no set)
/// `#[property(get="path", set)]`            | ✔️ custom get, default set | ❌️
/// `#[property(get="path", set="path")]`     | ⚠️ custom get + set        | ✔️ custom get + set
///
/// "⚠️" means that this attribute combination is allowed for bare `T`, but you should consider
/// using `Property<T>`.
///
/// Since there is no default `get` or `set` in these cases, godot-rust will never access the field
/// directly. In other words, you are not really exporting _that field_, but linking its name and type
/// (but not its value) to the specified get/set methods.
///
/// To decide when to use which:
/// * If you access your field as-is on the Rust side, use bare `T`.<br>
///   With a `Property<T>` field on the other hand, you would need to _additionally_ add a `T` backing field.
/// * If you don't need a backing field, use `Property<T>`.<br>
///   This is the case whenever you compute a result dynamically, or map values between Rust and GDScript
///   representations.
///
/// ## Examples
///
/// Read/write accessible:
/// ```no_run
/// # use gdnative::prelude::*;
/// #[derive(NativeClass)]
/// # #[no_constructor]
/// struct MyObject {
///     #[property]
///     color: Color,
/// }
/// ```
///
/// Read-only:
/// ```no_run
/// # use gdnative::prelude::*;
/// #[derive(NativeClass)]
/// # #[no_constructor]
/// struct MyObject {
///     #[property(get)]
///     hitpoints: f32,
/// }
/// ```
///
/// Read-write, with validating setter:
/// ```no_run
/// # use gdnative::prelude::*;
/// # fn validate(s: &String) -> bool { true }
/// #[derive(NativeClass)]
/// # #[no_constructor]
/// struct MyObject {
///     #[property(get, set = "Self::set_name")]
///     player_name: String,
/// }
///
/// #[methods]
/// impl MyObject {
///     fn set_name(&mut self, _owner: TRef<Reference>, name: String) {
///         if validate(&name) {
///             self.player_name = name;
///         }
///     }
/// }
/// ```
///
/// Write-only, no backing field, custom setter:
/// ```no_run
/// # use gdnative::prelude::*;
/// #[derive(NativeClass)]
/// # #[no_constructor]
/// struct MyObject {
///     #[property(set = "Self::set_password")]
///     password: Property<String>,
/// }
///
/// #[methods]
/// impl MyObject {
///     fn set_password(&mut self, _owner: TRef<Reference>, password: String) {
///         // securely hash and store password
///     }
/// }
/// ```

// Note: traits are mostly implemented to enable deriving the same traits on the enclosing struct.
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Property<T> {
    _marker: PhantomData<T>,
}

mod impl_export {
    use super::*;

    /// Hint type indicating that there are no hints available for the time being.
    ///
    /// This is an inhabitable type that cannot be constructed. As a result, users
    /// *must* use `None` for the property hint when exporting types using this as
    /// the hint.
    pub enum NoHint {}

    macro_rules! impl_export_for_int {
        ($ty:ident) => {
            impl Export for $ty {
                type Hint = hint::IntHint<$ty>;
                #[inline]
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
                #[inline]
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
                #[inline]
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
                type Hint = NoHint;
                #[inline]
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
    impl_export_for_core_type_without_hint!(PoolArray<u8>: ByteArray);
    impl_export_for_core_type_without_hint!(PoolArray<i32>: Int32Array);
    impl_export_for_core_type_without_hint!(PoolArray<f32>: Float32Array);
    impl_export_for_core_type_without_hint!(PoolArray<GodotString>: StringArray);
    impl_export_for_core_type_without_hint!(PoolArray<Vector2>: Vector2Array);
    impl_export_for_core_type_without_hint!(PoolArray<Vector3>: Vector3Array);
    impl_export_for_core_type_without_hint!(PoolArray<Color>: ColorArray);

    impl Export for Color {
        type Hint = hint::ColorHint;
        #[inline]
        fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
            hint.map_or_else(
                || ExportInfo::new(VariantType::Color),
                Self::Hint::export_info,
            )
        }
    }

    impl<T> Export for Ref<T, Shared>
    where
        T: GodotObject,
    {
        type Hint = NoHint;
        #[inline]
        fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
            ExportInfo::resource_type::<T>()
        }
    }

    impl<T> Export for Instance<T, Shared>
    where
        T: NativeClass,
        Instance<T, Shared>: ToVariant,
    {
        type Hint = NoHint;
        #[inline]
        fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
            ExportInfo::resource_type::<T::Base>()
        }
    }

    impl<T> Export for Option<T>
    where
        T: Export,
    {
        type Hint = T::Hint;
        #[inline]
        fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
            T::export_info(hint)
        }
    }

    impl Export for VariantArray<Shared> {
        type Hint = hint::ArrayHint;

        #[inline]
        fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
            hint.unwrap_or_default().export_info()
        }
    }
}
