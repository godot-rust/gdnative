use crate::api::*;

use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_ctor(class: &GodotClass) -> TokenStream {
    let method_table = format_ident!("{}MethodTable", class.name);

    let documentation = if class.is_refcounted() {
        r#"Creates a new instance of this object.

This is a reference-counted type. The returned object is automatically managed
by `Ref`."#
    } else {
        r#"Creates a new instance of this object.

Because this type is not reference counted, the lifetime of the returned object
is *not* automatically managed.

Immediately after creation, the object is owned by the caller, and can be
passed to the engine (in which case the engine will be responsible for
destroying the object) or destroyed manually using `Ptr::free`, or preferably
`Ptr::queue_free` if it is a `Node`."#
    };

    quote! {
        #[doc=#documentation]
        #[inline]
        pub fn new() -> Ref<Self, thread_access::Unique> {
            unsafe {
                let gd_api = get_api();
                let ctor = #method_table::get(gd_api).class_constructor.unwrap();
                let obj = ptr::NonNull::new(ctor()).expect("constructor should not return null");

                Ref::init_from_sys(obj)
            }
        }
    }
}

pub fn generate_godot_object_impl(class: &GodotClass) -> TokenStream {
    let name = &class.name;
    let class_name = format_ident!("{}", class.name);

    let ref_kind = if class.is_refcounted() {
        quote! { ref_kind::RefCounted }
    } else {
        quote! { ref_kind::ManuallyManaged }
    };

    quote! {
        impl gdnative_core::private::godot_object::Sealed for #class_name {}

        unsafe impl GodotObject for #class_name {
            type RefKind = #ref_kind;

            #[inline]
            fn class_name() -> &'static str {
                #name
            }
        }
    }
}

pub fn generate_instantiable_impl(class: &GodotClass) -> TokenStream {
    assert!(class.instantiable, "class should be instantiable");

    let class_name = format_ident!("{}", class.name);
    quote! {
        impl Instanciable for #class_name {
            #[inline]
            fn construct() -> Ref<Self, thread_access::Unique> {
                #class_name::new()
            }
        }
    }
}

pub fn generate_queue_free_impl(api: &Api, class: &GodotClass) -> TokenStream {
    let class_name = format_ident!("{}", class.name);

    let queue_free_output = if class.name == "Node" || api.class_inherits(&class, "Node") {
        quote! {
            impl QueueFree for #class_name {
                #[inline]
                unsafe fn godot_queue_free(obj: *mut sys::godot_object) {
                    crate::generated::node::Node_queue_free(obj)
                }
            }
        }
    } else {
        Default::default()
    };

    quote! {
        #queue_free_output
    }
}

pub fn generate_singleton_getter(class: &GodotClass) -> TokenStream {
    assert!(class.singleton, "class should be a singleton");

    let s_name = if class.name.starts_with('_') {
        &class.name[1..]
    } else {
        class.name.as_ref()
    };

    let singleton_name = format!("{}\0", s_name);

    assert!(
        singleton_name.ends_with('\0'),
        "singleton_name should be null terminated"
    );
    quote! {
        #[inline]
        pub fn godot_singleton() -> &'static Self {
            unsafe {
                let this = (get_api().godot_global_get_singleton)(#singleton_name.as_ptr() as *mut _);
                let this = ptr::NonNull::new(this).expect("singleton should not be null");
                let this = RawObject::from_sys_ref_unchecked::<'static>(this);
                Self::cast_ref(this)
            }
        }
    }
}

pub fn generate_upcast(api: &Api, base_class_name: &str, is_pointer_safe: bool) -> TokenStream {
    if let Some(parent) = api.find_class(&base_class_name) {
        let snake_name = class_name_to_snake_case(&base_class_name);
        let parent_class = format_ident!("{}", parent.name);
        let parent_class_module = format_ident!("{}", parent.name.to_snake_case());
        let to_snake_name = format_ident!("to_{}", snake_name);

        let upcast = generate_upcast(api, &parent.base_class, is_pointer_safe);
        let qualified_parent_class = quote! {
            crate::generated::#parent_class_module::#parent_class
        };
        quote! {
            /// Up-cast.
            #[inline]
            pub fn #to_snake_name(&self) -> &#qualified_parent_class {
                unsafe { #qualified_parent_class::cast_ref(self.this.cast_unchecked()) }
            }

            #upcast
        }
    } else {
        Default::default()
    }
}

pub fn generate_deref_impl(class: &GodotClass) -> TokenStream {
    assert!(
        !class.base_class.is_empty(),
        "should not be called on a class with no base_class"
    );

    let class_name = format_ident!("{}", class.name);
    let base_class_module = format_ident!("{}", class.base_class.to_snake_case(),);
    let base_class = format_ident!("{}", class.base_class);

    let qualified_base_class = quote! {
        crate::generated::#base_class_module::#base_class
    };

    quote! {
        impl std::ops::Deref for #class_name {
            type Target = #qualified_base_class;

            #[inline]
            fn deref(&self) -> &#qualified_base_class {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }

        impl std::ops::DerefMut for #class_name {
            #[inline]
            fn deref_mut(&mut self) -> &mut #qualified_base_class {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }
    }
}

pub fn generate_gdnative_library_singleton_getter(class: &GodotClass) -> TokenStream {
    assert_eq!(
        class.name, "GDNativeLibrary",
        "Can only generate library singletons for GDNativeLibrary"
    );

    quote! {
        /// Returns the GDNativeLibrary object of this library. Can be used to construct NativeScript objects.
        ///
        /// See also `Instance::new` for a typed API.
        #[inline]
        pub fn current_library() -> &'static Self {
            unsafe {
                let this = gdnative_core::private::get_gdnative_library_sys();
                let this = ptr::NonNull::new(this).expect("singleton should not be null");
                let this = RawObject::from_sys_ref_unchecked(this);
                Self::cast_ref(this)
            }
        }
    }
}

pub fn class_name_to_snake_case(name: &str) -> String {
    // TODO: this is a quick-n-dirty band-aid, it'd be better to
    // programmatically do the right conversion, but to_snake_case
    // currently translates "Node2D" into "node2_d".
    match name {
        "SpriteBase3D" => "sprite_base_3d".to_string(),
        "Node2D" => "node_2d".to_string(),
        "CollisionObject2D" => "collision_object_2d".to_string(),
        "PhysicsBody2D" => "physics_body_2d".to_string(),
        "VisibilityNotifier2D" => "visibility_notifier_2d".to_string(),
        "Joint2D" => "joint_2d".to_string(),
        "Shape2D" => "shape_2d".to_string(),
        "Physics2DServer" => "physics_2d_server".to_string(),
        "Physics2DDirectBodyState" => "physics_2d_direct_body_state".to_string(),
        _ => name.to_snake_case(),
    }
}
