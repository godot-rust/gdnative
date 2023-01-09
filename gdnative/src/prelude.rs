pub use gdnative_bindings::utils::*;
pub use gdnative_bindings::{
    Button, CanvasItem, CanvasLayer, ColorRect, Control, Image, Input, InputEvent, InputEventKey,
    KinematicBody, KinematicBody2D, Label, Node, Node2D, Object, PackedScene, Reference,
    ResourceLoader, SceneTree, Shader, Spatial, Sprite, Texture, Timer, Tween, Viewport,
};
pub use gdnative_core::core_types::{
    Aabb, Basis, Color, Dictionary, GodotError, GodotString, NodePath, Plane, PoolArray, Quat,
    Rect2, Rid, StringName, Transform, Transform2D, Variant, VariantArray, VariantDispatch,
    VariantOperator, VariantType, Vector2, Vector3,
};
#[allow(deprecated)]
pub use gdnative_core::core_types::{
    ByteArray, ColorArray, Float32Array, Int32Array, StringArray, Vector2Array, Vector3Array,
};
pub use gdnative_core::core_types::{
    FromVariant, FromVariantError, OwnedToVariant, ToVariant, ToVariantEq,
};
pub use gdnative_core::export::{
    ClassBuilder, ExportInfo, Method, MethodBuilder, NativeClass, NativeClassMethods, Property,
    PropertyUsage, SignalBuilder, SignalParam,
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
#[doc(inline)]
pub use crate::globalscope::load;
