use crate::api::*;

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
destroying the object) or destroyed manually using `Ref::free`, or preferably
`Ref::queue_free` if it is a `Node`."#
    };

    quote! {
        #[doc=#documentation]
        #[inline]
        pub fn new() -> Ref<Self, ownership::Unique> {
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

    let memory = if class.is_refcounted() {
        quote! { memory::RefCounted }
    } else {
        quote! { memory::ManuallyManaged }
    };

    quote! {
        impl gdnative_core::private::godot_object::Sealed for #class_name {}

        unsafe impl GodotObject for #class_name {
            type Memory = #memory;

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
            fn construct() -> Ref<Self, ownership::Unique> {
                #class_name::new()
            }
        }
    }
}

pub fn generate_queue_free_impl(api: &Api, class: &GodotClass) -> TokenStream {
    let class_name = format_ident!("{}", class.name);

    let queue_free_output = if class.name == "Node" || api.class_inherits(class, "Node") {
        #[cfg(feature = "ptrcall")]
        let icall_ident = proc_macro2::Ident::new("icallptr_void", proc_macro2::Span::call_site());

        #[cfg(not(feature = "ptrcall"))]
        let icall_ident = proc_macro2::Ident::new("icallvar_", proc_macro2::Span::call_site());

        quote! {
            impl QueueFree for #class_name {
                #[inline]
                unsafe fn godot_queue_free(obj: *mut sys::godot_object) {
                    let method_bind: *mut sys::godot_method_bind = crate::generated::node::NodeMethodTable::get(get_api()).queue_free;
                    crate::icalls::#icall_ident(method_bind, obj);
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

    let singleton_name = format!("{s_name}\0");

    assert!(
        singleton_name.ends_with('\0'),
        "singleton_name should be null terminated"
    );

    let (maybe_unsafe, doc) = if class.is_singleton_thread_safe() {
        (
            TokenStream::new(),
            "Returns a reference to the singleton instance.",
        )
    } else {
        let maybe_unsafe = quote! { unsafe };
        let doc = r#"Returns a reference to the singleton instance.

# Safety

This singleton server is only safe to access from outside the main thread if thread-safe
operations are enabled in the project settings. See the official
[thread-safety guidelines][thread-safety] for more information.

[thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html"#;
        (maybe_unsafe, doc)
    };

    quote! {
        #[doc=#doc]
        #[inline]
        pub #maybe_unsafe fn godot_singleton() -> &'static Self {
            unsafe {
                let this = (get_api().godot_global_get_singleton)(#singleton_name.as_ptr() as *mut _);
                let this = ptr::NonNull::new(this).expect("singleton should not be null");
                let this = RawObject::from_sys_ref_unchecked::<'static>(this);
                Self::cast_ref(this)
            }
        }
    }
}

pub fn generate_deref_impl(class: &GodotClass) -> TokenStream {
    assert!(
        !class.base_class.is_empty(),
        "should not be called on a class with no base_class"
    );

    let class_name = format_ident!("{}", class.name);
    let base_class = format_ident!("{}", class.base_class);

    let qualified_base_class = quote! {
        crate::generated::#base_class
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

pub fn generate_sub_class_impls<'a>(api: &'a Api, mut class: &'a GodotClass) -> TokenStream {
    let class_name = format_ident!("{}", class.name);

    let mut tokens = TokenStream::new();

    while let Some(base_class) = class.base_class(api) {
        let base_class_ident = format_ident!("{}", base_class.name);

        tokens.extend(quote! {
            unsafe impl SubClass<crate::generated::#base_class_ident> for #class_name {}
        });

        class = base_class;
    }

    tokens
}

pub fn generate_send_sync_impls(class: &GodotClass) -> TokenStream {
    assert!(class.is_singleton_thread_safe());
    let class_name = format_ident!("{}", class.name);

    quote! {
        unsafe impl Send for #class_name {}
        unsafe impl Sync for #class_name {}
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
