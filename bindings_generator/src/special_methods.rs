use crate::api::*;
use crate::GeneratorResult;

use heck::SnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::io::Write;

pub fn generate_reference_ctor(class: &GodotClass) -> TokenStream {
    let class_name = format_ident!("{}", class.name);
    let method_table = format_ident!("{}MethodTable", class.name);
    quote! {
        // Constructor
        #[inline]
        pub fn new() -> Self {
            unsafe {
                let gd_api = get_api();
                let ctor = #method_table::get(gd_api).class_constructor.unwrap();
                let obj = ctor();
                object::init_ref_count(obj);

                #class_name {
                    this: obj
                }
            }
        }
    }
}

pub fn generate_non_reference_ctor(class: &GodotClass) -> TokenStream {
    let class_name = format_ident!("{}", class.name);
    let method_table = format_ident!("{}MethodTable", class.name);

    let documentation = format!(
        r#"/// Constructor.
///
/// Because this type is not reference counted, the lifetime of the returned object
/// is *not* automatically managed.
/// Immediately after creation, the object is owned by the caller, and can be
/// passed to the engine (in which case the engine will be responsible for
/// destroying the object) or destroyed manually using `{}::free`."#,
        class_name
    );

    quote! {
        #[doc=#documentation]
        #[inline]
        pub fn new() -> Self {
            unsafe {
                let gd_api = get_api();
                let ctor = #method_table::get(gd_api).class_constructor.unwrap();
                let this = ctor();

                #class_name {
                    this
                }
            }
        }

        /// Manually deallocate the object.
        #[inline]
        pub unsafe fn free(self) {
            (get_api().godot_object_destroy)(self.this);
        }
    }
}

pub fn generate_godot_object_impl(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    let name = &class.name;
    let class_name = format_ident!("{}", class.name);
    let addref_if_reference = if class.is_refcounted() {
        quote! { object::add_ref(obj); }
    } else {
        quote! {
           // Not reference-counted.
        }
    };

    let code = quote! {
        impl crate::private::godot_object::Sealed for #class_name {}

        unsafe impl GodotObject for #class_name {
            #[inline]
            fn class_name() -> &'static str {
                #name
            }

            #[inline]
            unsafe fn from_sys(obj: *mut sys::godot_object) -> Self {
                #addref_if_reference
                Self { this: obj, }
            }

            #[inline]
            unsafe fn from_return_position_sys(obj: *mut sys::godot_object) -> Self {
                Self { this: obj, }
            }

            #[inline]
            unsafe fn to_sys(&self) -> *mut sys::godot_object {
                self.this
            }
        }

        impl ToVariant for #class_name {
            #[inline]
            fn to_variant(&self) -> Variant { Variant::from_object(self) }
        }

        impl FromVariant for #class_name {
            #[inline]
            fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
                variant.try_to_object_with_error::<Self>()
            }
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_instantiable_impl(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    assert!(class.instantiable, "class should be instantiable");

    let class_name = format_ident!("{}", class.name);
    let code = quote! {
        impl Instanciable for #class_name {
            #[inline]
            fn construct() -> Self {
                #class_name::new()
            }
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_free_impl(
    output: &mut impl Write,
    api: &Api,
    class: &GodotClass,
) -> GeneratorResult {
    let class_name = format_ident!("{}", class.name);
    let free_output = if class.instantiable && !class.is_pointer_safe() {
        quote! {
            impl Free for #class_name {
                #[inline]
                unsafe fn godot_free(self) { self.free() }
            }
        }
    } else {
        Default::default()
    };

    let queue_free_output = if class.name == "Node" || api.class_inherits(&class, "Node") {
        quote! {
            impl QueueFree for #class_name {
                #[inline]
                unsafe fn godot_queue_free(&mut self) { self.queue_free() }
            }
        }
    } else {
        Default::default()
    };

    let code = quote! {
        #free_output
        #queue_free_output
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_singleton_getter(class: &GodotClass) -> TokenStream {
    assert!(class.singleton, "class should be a singleton");

    let s_name = if class.name.starts_with('_') {
        &class.name[1..]
    } else {
        class.name.as_ref()
    };

    let class_name = format_ident!("{}", class.name);
    let singleton_name = format!("{}\0", s_name);

    assert!(
        singleton_name.ends_with('\0'),
        "singleton_name should be null terminated"
    );
    quote! {
        #[inline]
        pub fn godot_singleton() -> Self {
            unsafe {
                let this = (get_api().godot_global_get_singleton)(#singleton_name.as_ptr() as *mut _);

                #class_name {
                    this
                }
            }
        }
    }
}

pub fn generate_dynamic_cast(class: &GodotClass) -> TokenStream {
    let maybe_unsafe = if class.is_pointer_safe() {
        Default::default()
    } else {
        quote! {unsafe}
    };

    quote! {
        /// Generic dynamic cast.
        #[inline]
        pub #maybe_unsafe fn cast<T: GodotObject>(&self) -> Option<T> {
        unsafe {
                object::godot_cast::<T>(self.this)
            }
        }
    }
}

pub fn generate_upcast(api: &Api, base_class_name: &str, is_pointer_safe: bool) -> TokenStream {
    if let Some(parent) = api.find_class(&base_class_name) {
        let snake_name = class_name_to_snake_case(&base_class_name);
        let parent_class = format_ident!("{}", parent.name);
        let to_snake_name = format_ident!("to_{}", snake_name);
        let addref_if_reference = if parent.is_refcounted() {
            quote! {
                unsafe { object::add_ref(self.this); }
            }
        } else {
            quote! {
                // Not reference-counted.
            }
        };
        let maybe_unsafe = if is_pointer_safe {
            Default::default()
        } else {
            quote! { unsafe }
        };

        let upcast = generate_upcast(api, &parent.base_class, is_pointer_safe);
        quote! {
            /// Up-cast.
            #[inline]
            pub #maybe_unsafe fn #to_snake_name(&self) -> #parent_class {
                #addref_if_reference
                unsafe { #parent_class::from_sys(self.this) }
            }

            #upcast
        }
    } else {
        Default::default()
    }
}

pub fn generate_deref_impl(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    assert!(
        !class.base_class.is_empty(),
        "should not be called on a class with no base_class"
    );

    let class_name = format_ident!("{}", class.name);
    let base_class = format_ident!("{}", class.base_class);

    let code = quote! {
        impl std::ops::Deref for #class_name {
            type Target = #base_class;

            #[inline]
            fn deref(&self) -> &#base_class {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }

        impl std::ops::DerefMut for #class_name {
            #[inline]
            fn deref_mut(&mut self) -> &mut #base_class {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_reference_clone(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    assert!(class.is_refcounted(), "Only call with refcounted classes");

    let class_name = format_ident!("{}", class.name);

    let code = quote! {
        impl Clone for #class_name {
            #[inline]
            fn clone(&self) -> Self {
                self.new_ref()
            }
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_impl_ref_counted(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    assert!(class.is_refcounted(), "Only call with refcounted classes");

    let class_name = format_ident!("{}", class.name);
    let code = quote! {
        impl RefCounted for #class_name {
            /// Creates a new reference to the same reference-counted object.
            #[inline]
            fn new_ref(&self) -> Self {
                unsafe {
                    object::add_ref(self.this);

                    Self {
                        this: self.this,
                    }
                }
            }
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
}

pub fn generate_drop(output: &mut impl Write, class: &GodotClass) -> GeneratorResult {
    assert!(class.is_refcounted(), "Only call with refcounted classes");

    let class_name = format_ident!("{}", class.name);
    let code = quote! {
        impl Drop for #class_name {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    if object::unref(self.this) {
                        (get_api().godot_object_destroy)(self.this);
                    }
                }
            }
        }
    };

    generated_at!(output);
    write!(output, "{}", code)?;

    Ok(())
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
        pub fn current_library() -> Self {
            let this = gdnative_core::private::get_gdnative_library_sys();

            Self {
                this
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
