use std::ops::Deref;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::mem;
use sys;
use GodotObject;
use Object;
use object;
use get_api;

/// Trait used for describing and initializing a Godot script class.
///
/// This trait is used to provide data and functionality to the
/// "data-part" of the class, such as name, initialization and information
/// about exported properties.
///
/// For exported methods, see the [`NativeClassMethods`] trait.
///
/// [`NativeClassMethods`]: ./trait.NativeClassMethods.html
pub trait NativeClass: Sized {

    /// Base type of the class.
    ///
    /// In Godot, scripting languages can define "script instances" which can be
    /// attached to objects. Because of the dynamic nature, the intended "inheritance"
    /// is not easily implementable properly.
    ///
    /// Instead, delegation is used and most calls to a Godot object query the script instance
    /// first. This way, some methods can be "overwritten" and new ones can be exposed.
    ///
    /// This only works when using so called "variant calls", since the querying of the script
    /// instance is performed there.
    /// When not using variant calls, any direct(*) calls have to be made to the Godot object
    /// directly.
    ///
    /// The base type describes the "most general" type of object this script class can be
    /// attached to.
    ///
    /// *(\*)*: GDNative enables use of "ptrcall"s, which are wrappers for function pointers.
    /// Those do not do explicit checks for script implementations **unless the method
    /// implementation does**.
    type Base: GodotObject;

    /// The name of the class.
    ///
    /// In GDNative+NativeScript many classes can be defined in one dynamic library.
    /// To identify which class has to be used, a library-unique name has to be given.
    fn class_name() -> &'static str;

    /// Function that creates a value of `Self`, used for the script-instance.
    ///
    /// This function has a reference to the owner object as a parameter, which can be used to
    /// set state on the owner upon creation or to query values
    fn init(owner: Self::Base) -> Self;

    /// Register any exported properties to Godot.
    fn register_properties(_builder: &crate::init::ClassBuilder<Self>) {}
}

/// Trait used to provide information of Godot-exposed methods of a script class.
pub trait NativeClassMethods: NativeClass {

    /// Function that registers all exposed methods to Godot.
    fn register(builder: &crate::init::ClassBuilder<Self>);
}

/// A reference to a rust native script.
pub struct NativeRef<T: NativeClass> {
    this: *mut sys::godot_object,
    _marker: PhantomData<T>,
}

impl<T: NativeClass> NativeRef<T> {

    /// Try to cast into a godot object reference.
    pub fn cast<O>(&self) -> Option<O> where O: GodotObject {
        object::godot_cast::<O>(self.this)
    }

    /// Creates a new reference to the same object.
    pub fn new_ref(&self) -> Self {
        unsafe {
            object::add_ref(self.this);

            Self {
                this: self.this,
                _marker: PhantomData,
            }
        }
    }

    fn get_impl(&self) -> &RefCell<T> {
        unsafe {
            let api = get_api();
            let ud = (api.godot_nativescript_get_userdata)(self.this);
            &*(ud as *const _ as *const RefCell<T>)
        }
    }

    #[doc(hidden)]
    pub unsafe fn sys(&self) -> *mut sys::godot_object {
        self.this
    }

    #[doc(hidden)]
    pub unsafe fn from_sys(ptr: *mut sys::godot_object) -> Self {
        object::add_ref(ptr);

        NativeRef {
            this: ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: NativeClass> Deref for NativeRef<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        self.get_impl()
    }
}

impl <T: NativeClass> Drop for NativeRef<T> {
    fn drop(&mut self) {
        unsafe {
            if object::unref(self.this) {
                (get_api().godot_object_destroy)(self.this);
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! godot_class_build_export_methods {
    ($classty:ty, $builder:ident,) => ();

    ($classty:ty, $builder:ident,
        export fn $name:ident(
            &mut self,
            $owner_name:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
        ) $body:block
        $($tt:tt)*
    ) => (
        godot_class_build_export_methods!($classty, $builder,
            export fn $name(&mut self, $owner_name: $owner_ty $(,$pname : $pty)*) -> () $body
            $($tt)*
        );
    );

    ($classty:ty, $builder:ident,
        export fn $name:ident(
            &mut self,
            $owner_name:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty $body:block
        $($tt:tt)*
    ) => (
        $builder.add_method(
            stringify!($name),
            godot_wrap_method!(
                $classty,
                fn $name(&mut self, $owner_name: $owner_ty $(,$pname : $pty)* ) -> $retty
            ),
        );
        godot_class_build_export_methods!($classty, $builder, $($tt)*);
    );
}

#[macro_export]
#[doc(hidden)]
macro_rules! godot_class_build_methods {
    () => ();
    (
        export fn $name:ident(
            &mut $self:ident
            $(,$pname:ident : $pty:ty)*
        ) $body:block
        $($tt:tt)*
    ) => (
        godot_class_build_methods!(
            export fn $name(&mut $self$(,$pname : $pty)*) -> () $body
            $($tt)*
        );
    );
    (
        export fn $name:ident(
            &mut $self:ident
            $(,$pname:ident : $pty:ty)*
        ) -> $retty:ty $body:block
        $($tt:tt)*
    ) => (
        pub fn $name(&mut $self$(
            ,$pname : $pty
        )*) -> $retty $body
        godot_class_build_methods!($($tt)*);
    )
}

/// Convenience macro to declare a native class.
///
/// ## Example
///
/// ```ignore
/// godot_class! {
///    class HelloWorld: godot::Node {
///
///        fields {
///            x: f32,
///        }
///
///        setup(builder) {
///            builder.add_property(
///                Property {
///                    name: "base/x",
///                    default: 1.0,
///                    hint: PropertyHint::Range {
///                        range: 0.0..1.0,
///                        step: 0.01,
///                        slider: true
///                    },
///                    getter: |this: &mut RustTest| this.x,
///                    setter: |this: &mut RustTest, v| this.x = v,
///                    usage: PropertyUsage::DEFAULT,
///                }
///            );
///        }
///
///        constructor(_owner: godot::Node) {
///            HelloWorld {
///                x: 0.0,
///            }
///        }
///
///        export fn _ready(&mut self, _owner: godot::Node) {
///            godot_print!("hello, world.");
///        }
///    }
/// }
/// ```
#[macro_export]
macro_rules! godot_class {
    (
class $name:ident: $owner:ty {
    fields {
        $(
            $(#[$fattr:meta])*
            $fname:ident : $fty:ty,
        )*
    }
    setup($builder:ident) $pbody:block
    constructor($owner_name:ident : $owner_ty:ty) $construct:block

    $($tt:tt)*
}
    ) => (
        pub struct $name {
            $(
                $(#[$fattr])*
                pub $fname: $fty,
            )*
        }

        impl $name {
            godot_class_build_methods!($($tt)*);

            fn _constructor($owner_name: $owner_ty) -> Self {
                $construct
            }
        }

        impl $crate::NativeClassMethods for $name {
            fn register($builder: &$crate::init::ClassBuilder<Self>) {
                godot_class_build_export_methods!($name, $builder, $($tt)*);
            }
        }

        impl $crate::NativeClass for $name {
            type Base = $owner;

            fn class_name() -> &'static str { stringify!($name) }

            fn init(owner: $owner) -> Self {
                $name::_constructor(owner)
            }

            fn register_properties($builder: &$crate::init::ClassBuilder<Self>) {
                $pbody
            }
        }
    );
}

#[cfg(test)]
godot_class! {
    class TestClass: super::Object {

        fields {
            a: u32,
        }

        setup(_builder) {}

        constructor(_owner: super::Object) {
            TestClass {
                a: 42,
            }
        }

        export fn _ready(&mut self, _owner: super::Object) {
            godot_print!("hello, world.");
        }
    }
}
