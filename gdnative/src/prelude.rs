pub use gdnative_core::core_types::{
    Aabb, Basis, ByteArray, Color, ColorArray, Dictionary, Float32Array, GodotError, GodotString,
    Int32Array, NodePath, Plane, Quat, Rect2, Rid, StringArray, StringName, Transform, Transform2D,
    TypedArray, Variant, VariantArray, VariantDispatch, VariantOperator, VariantType, Vector2,
    Vector2Array, Vector3, Vector3Array,
};

pub use gdnative_core::core_types::{
    FromVariant, FromVariantError, OwnedToVariant, ToVariant, ToVariantEq,
};

pub use gdnative_core::object::{
    memory::{ManuallyManaged, RefCounted},
    ownership::{Shared, ThreadLocal, Unique},
    AsArg, GodotObject, Instanciable, NewRef, Null, QueueFree, Ref, SubClass, TRef,
};

pub use gdnative_core::export::{
    ClassBuilder, ExportInfo, InitHandle, Instance, Method, MethodBuilder, NativeClass,
    NativeClassMethods, PropertyUsage, RefInstance, Signal, SignalArgument,
};

// Re-export selected user_data types, but keep qualified due to rather generic names
pub mod user_data {
    pub use gdnative_core::export::user_data::{
        Aether, ArcData, LocalCellData, MutexData, RwLockData,
    };
}

pub use gdnative_core::{
    godot_dbg, godot_error, godot_gdnative_init, godot_gdnative_terminate, godot_init,
    godot_nativescript_init, godot_print, godot_site, godot_warn,
};

pub use gdnative_derive::*;

#[cfg(feature = "bindings")]
pub use gdnative_bindings::{
    Button, CanvasItem, CanvasLayer, ColorRect, Control, Image, Input, InputEvent, InputEventKey,
    KinematicBody, KinematicBody2D, Label, Node, Node2D, Object, PackedScene, Reference,
    ResourceLoader, SceneTree, Shader, Spatial, Sprite, Texture, Timer, Tween, Viewport,
};

#[cfg(feature = "bindings")]
pub use gdnative_bindings::utils::*;
