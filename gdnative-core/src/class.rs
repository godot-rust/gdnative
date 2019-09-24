use crate::get_api;
use crate::object;
use crate::sys;
use crate::FromVariant;
use crate::GodotObject;
use crate::GodotString;
use crate::Instanciable;
use crate::Map;
use crate::MapMut;
use crate::ToVariant;
use crate::UserData;
use crate::Variant;

/// Trait used for describing and initializing a Godot script class.
///
/// This trait is used to provide data and functionality to the
/// "data-part" of the class, such as name, initialization and information
/// about exported properties.
///
/// For exported methods, see the [`NativeClassMethods`] trait.
///
/// [`NativeClassMethods`]: ./trait.NativeClassMethods.html
pub trait NativeClass: Sized + 'static {
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

    /// User-data wrapper type of the class.
    ///
    /// See module-level documentation on `user_data` for more info.
    type UserData: UserData<Target = Self>;

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

/// A reference to a GodotObject with a rust NativeClass attached.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Instance<T: NativeClass> {
    owner: T::Base,
    script: T::UserData,
}

impl<T: NativeClass> Instance<T> {
    /// Creates a `T::Base` with the script `T` attached. Both `T::Base` and `T` must have zero
    /// argument constructors.
    ///
    /// Must be called after the library is initialized.
    pub fn new() -> Self
    where
        T::Base: Instanciable,
    {
        unsafe {
            let gd_api = get_api();

            // The API functions take NUL-terminated C strings. &CStr is not used for its runtime cost.
            let class_name = b"NativeScript\0".as_ptr() as *const libc::c_char;
            let ctor = (gd_api.godot_get_class_constructor)(class_name).unwrap();
            let set_class_name = (gd_api.godot_method_bind_get_method)(
                class_name,
                b"set_class_name\0".as_ptr() as *const libc::c_char,
            );
            let set_library = (gd_api.godot_method_bind_get_method)(
                class_name,
                b"set_library\0".as_ptr() as *const libc::c_char,
            );
            let object_set_script = crate::ObjectMethodTable::get(gd_api).set_script;

            let native_script = ctor();
            object::init_ref_count(native_script);

            let script_class_name = GodotString::from(T::class_name());
            let mut args: [*const libc::c_void; 1] = [script_class_name.sys() as *const _];
            (gd_api.godot_method_bind_ptrcall)(
                set_class_name,
                native_script,
                args.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            let mut args: [*const libc::c_void; 1] = [crate::get_gdnative_library_sys()];
            (gd_api.godot_method_bind_ptrcall)(
                set_library,
                native_script,
                args.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            let owner = T::Base::construct();

            assert_ne!(
                std::ptr::null_mut(),
                owner.to_sys(),
                "base object should not be null"
            );

            let mut args: [*const libc::c_void; 1] = [native_script as *const _];
            (gd_api.godot_method_bind_ptrcall)(
                object_set_script,
                owner.to_sys(),
                args.as_mut_ptr(),
                std::ptr::null_mut(),
            );

            let script_ptr =
                (gd_api.godot_nativescript_get_userdata)(owner.to_sys()) as *const libc::c_void;

            assert_ne!(
                std::ptr::null(),
                script_ptr,
                "script instance should not be null"
            );

            let script = T::UserData::clone_from_user_data_unchecked(script_ptr);

            object::unref(native_script);

            Instance { owner, script }
        }
    }

    pub fn into_base(self) -> T::Base {
        self.owner
    }

    pub fn into_script(self) -> T::UserData {
        self.script
    }

    pub fn decouple(self) -> (T::Base, T::UserData) {
        (self.owner, self.script)
    }

    pub fn base(&self) -> &T::Base {
        &self.owner
    }

    pub fn script(&self) -> &T::UserData {
        &self.script
    }

    /// Try to downcast `T::Base` to `Instance<T>`. This safe version can only be used with
    /// reference counted base classes.
    pub fn try_from_base(owner: T::Base) -> Option<Self>
    where
        T::Base: Clone,
    {
        unsafe { Self::try_from_unsafe_base(owner) }
    }

    /// Try to downcast `T::Base` to `Instance<T>`.
    ///
    /// # Safety
    ///
    /// It's up to the caller to ensure that `owner` points to a valid Godot object, and
    /// that it will not be freed until this function returns. Otherwise, it is undefined
    /// behavior to call this function and/or use its return value.
    pub unsafe fn try_from_unsafe_base(owner: T::Base) -> Option<Self> {
        let type_tag = (get_api().godot_nativescript_get_type_tag)(owner.to_sys());
        if type_tag.is_null() {
            return None;
        }

        if !crate::type_tag::check::<T>(type_tag) {
            return None;
        }

        Some(Self::from_sys_unchecked(owner.to_sys()))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    pub fn map<F, U>(&self, op: F) -> Result<U, <T::UserData as Map>::Err>
    where
        T::Base: Clone,
        T::UserData: Map,
        F: FnOnce(&T, T::Base) -> U,
    {
        self.script.map(|script| op(script, self.owner.clone()))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used on reference counted types for multiple times.
    pub fn map_mut<F, U>(&self, op: F) -> Result<U, <T::UserData as MapMut>::Err>
    where
        T::Base: Clone,
        T::UserData: MapMut,
        F: FnOnce(&mut T, T::Base) -> U,
    {
        self.script.map_mut(|script| op(script, self.owner.clone()))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used for multiple times via aliasing:
    ///
    /// ```ignore
    /// unsafe {
    ///     instance.map_aliased(/* ... */);
    ///     // instance.owner may be invalid now, but you can still:
    ///     instance.map_aliased(/* ... */);
    ///     instance.map_aliased(/* ... */); // ...for multiple times
    /// }
    /// ```
    ///
    /// For reference-counted types behaves like the safe `map`, which should be preferred.
    pub unsafe fn map_aliased<F, U>(&self, op: F) -> Result<U, <T::UserData as Map>::Err>
    where
        T::UserData: Map,
        F: FnOnce(&T, T::Base) -> U,
    {
        self.script
            .map(|script| op(script, T::Base::from_sys(self.owner.to_sys())))
    }

    /// Calls a function with a NativeClass instance and its owner, and returns its return
    /// value. Can be used for multiple times via aliasing:
    ///
    /// ```ignore
    /// unsafe {
    ///     instance.map_mut_aliased(/* ... */);
    ///     // instance.owner may be invalid now, but you can still:
    ///     instance.map_mut_aliased(/* ... */);
    ///     instance.map_mut_aliased(/* ... */); // ...for multiple times
    /// }
    /// ```
    ///
    /// For reference-counted types behaves like the safe `map_mut`, which should be preferred.
    pub unsafe fn map_mut_aliased<F, U>(&self, op: F) -> Result<U, <T::UserData as MapMut>::Err>
    where
        T::UserData: MapMut,
        F: FnOnce(&mut T, T::Base) -> U,
    {
        self.script
            .map_mut(|script| op(script, T::Base::from_sys(self.owner.to_sys())))
    }

    #[doc(hidden)]
    pub unsafe fn from_sys_unchecked(ptr: *mut sys::godot_object) -> Self {
        let api = get_api();
        let user_data = (api.godot_nativescript_get_userdata)(ptr);
        Self::from_raw(ptr, user_data)
    }

    #[doc(hidden)]
    pub unsafe fn from_raw(ptr: *mut sys::godot_object, user_data: *mut libc::c_void) -> Self {
        let owner = T::Base::from_sys(ptr);
        let script_ptr = user_data as *const libc::c_void;
        let script = T::UserData::clone_from_user_data_unchecked(script_ptr);
        Instance { owner, script }
    }
}

impl<T> Clone for Instance<T>
where
    T: NativeClass,
    T::Base: Clone,
{
    fn clone(&self) -> Self {
        Instance {
            owner: self.owner.clone(),
            script: self.script.clone(),
        }
    }
}

impl<T> ToVariant for Instance<T>
where
    T: NativeClass,
    T::Base: ToVariant,
{
    fn to_variant(&self) -> Variant {
        self.owner.to_variant()
    }
}

impl<T> FromVariant for Instance<T>
where
    T: NativeClass,
    T::Base: FromVariant + Clone,
{
    fn from_variant(variant: &Variant) -> Option<Self> {
        let owner = T::Base::from_variant(variant)?;
        Self::try_from_base(owner)
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
class $name:ident ($user_data:ty) : $owner:ty {
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
            type UserData = $user_data;

            fn class_name() -> &'static str { stringify!($name) }

            fn init(owner: $owner) -> Self {
                $name::_constructor(owner)
            }

            fn register_properties($builder: &$crate::init::ClassBuilder<Self>) {
                $pbody
            }
        }
    );
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
        godot_class! {
            class $name ($crate::user_data::DefaultUserData<$name>) : $owner {
                fields {
                    $(
                        $(#[$fattr])*
                        $fname : $fty,
                    )*
                }

                setup($builder) $pbody
                constructor($owner_name : $owner_ty) $construct

                $($tt)*
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

#[cfg(test)]
godot_class! {
    class TestReturnInstanceClass: super::Object {

        fields {
        }

        setup(_builder) {}

        constructor(_owner: super::Object) {
            TestReturnInstanceClass {
            }
        }

        export fn answer(&mut self, _owner: super::Object) -> Instance<TestClass> {
            Instance::new()
        }
    }
}
