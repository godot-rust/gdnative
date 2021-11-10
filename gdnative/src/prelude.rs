#[cfg(feature = "bindings")]
pub use gdnative_bindings::utils::*;
#[cfg(feature = "bindings")]
pub use gdnative_bindings::{
    Button, CanvasItem, CanvasLayer, ColorRect, Control, Image, Input, InputEvent, InputEventKey,
    KinematicBody, KinematicBody2D, Label, Node, Node2D, Object, PackedScene, Reference,
    ResourceLoader, SceneTree, Shader, Spatial, Sprite, Texture, Timer, Tween, Viewport,
};
pub use gdnative_core::core_types::{
    Aabb, Basis, ByteArray, Color, ColorArray, Dictionary, Float32Array, GodotError, GodotString,
    Int32Array, NodePath, Plane, Quat, Rect2, Rid, StringArray, StringName, Transform, Transform2D,
    TypedArray, Variant, VariantArray, VariantDispatch, VariantOperator, VariantType, Vector2,
    Vector2Array, Vector3, Vector3Array,
};
pub use gdnative_core::core_types::{
    FromVariant, FromVariantError, OwnedToVariant, ToVariant, ToVariantEq,
};
pub use gdnative_core::export::{
    ClassBuilder, ExportInfo, Method, MethodBuilder, NativeClass, NativeClassMethods,
    PropertyUsage, Signal, SignalArgument,
};
pub use gdnative_core::init::InitHandle;
pub use gdnative_core::object::{
    memory::{ManuallyManaged, RefCounted},
    ownership::{Shared, ThreadLocal, Unique},
    AsArg, GodotObject, Instance, Instanciable, NewRef, Null, QueueFree, Ref, SubClass, TInstance,
    TRef,
};
pub use gdnative_core::{godot_dbg, godot_error, godot_init, godot_print, godot_warn};
pub use gdnative_derive::*;

/// User-data attributes from [`export::user_data`][crate::export::user_data] module.
pub mod user_data {
    // Re-export selected user_data types, but keep qualified due to rather generic names
    pub use gdnative_core::export::user_data::{
        Aether, ArcData, LocalCellData, MutexData, RwLockData,
    };
}

// Deprecated symbols. Keep them only in prelude, as all the other paths have changed anyway.
// This way, old symbol names are still discoverable and users who used prelude won't have (as many) breaking changes.
// Important: the referred-to type (right-hand-side) should point into the full path, not the prelude re-export.

#[deprecated(since = "0.10.0", note = "Confusing name; use TInstance instead.")]
pub type RefInstance<'a, T, Access> = crate::object::TInstance<'a, T, Access>;
