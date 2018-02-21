use super::*;
use godot_type::GodotType;
use std::marker::PhantomData;

pub struct PropertiesBuilder<C> {
    #[doc(hidden)]
    pub desc: *mut libc::c_void,
    #[doc(hidden)]
    pub class_name: *const libc::c_char,
    #[doc(hidden)]
    pub class: PhantomData<C>,
}

pub enum PropertyHint {
    None,
    Range {
        min: f64,
        max: f64,
        step: f64,
        slider: bool,
    },
    // ExpRange,
    Enum {
        values: Vec<String>,
    },
    // ExpEasing,
    // Length,
    // SpriteFrame,
    // KeyAccel,
    Flags {
        values: Vec<String>,
    },
    // Layers2DRender,
    // Layers2DPhysics,
    // Layers3DRender,
    // Layers3DPhysics,
    // File,
    // Dir,
    // GlobalFile,
    // GlobalDir,
    // ResourceType,
    // MultilineText,
    // ColorNoAlpha,
    // ImageCompressLossy,
    // IMageCompressLossless,
    // ObjectID,
    // TypeString,
    NodePathToEditedNode,
    // MethodOfVariantType,
    // MethodOfBaseType,
    // MethodOfInstance,
    // MethodOfScript,
    // PropertyOfVariantType,
    // PropertyOfBaseType,
    // PropertyOfInstance,
    // PropertyOfScript,
}

bitflags! {
    pub struct PropertyUsage: u32 {
        const STORAGE = 1;
        const EDITOR = 2;
        const NETWORK = 4;
        const EDITOR_HELPER = 8;
        const CHECKABLE = 32;
        const CHECKED = 32;
        const INTERNATIONALIZED = 64;
        const GROUP = 128;
        const CATEGORY = 256;
        const STORE_IF_NONZERO = 512;
        const STORE_IF_NONONE = 1024;
        const NO_INSTANCE_STATE = 2048;
        const RESTART_IF_CHANGED = 0x1000;
        const SCRIPT_VARIABLE  = 0x2000;
        const STORE_IF_NULL = 0x4000;
        const ANIMATE_AS_TRIGGER = 0x8000;
        const UPDATE_ALL_IF_MODIFIED = 0x1_0000;

        const DEFAULT = Self::STORAGE.bits | Self::EDITOR.bits | Self::NETWORK.bits;
        const DEFAULT_INTL = Self::DEFAULT.bits | Self::INTERNATIONALIZED.bits;
        const NOEDITOR = Self::STORAGE.bits | Self::NETWORK.bits;
    }
}

impl <C> PropertiesBuilder<C>
    where C: GodotClass,
{
    pub fn property<T: GodotType>(&mut self, name: &str, default: T) -> PropertyBuilder<C, (), (), T>
    {
        let def = default.to_variant().forget();
        let api = get_api();
        PropertyBuilder {
            parent: self,
            name: name.into(),
            setter: (),
            getter: (),

            ty: unsafe { (api.godot_variant_get_type)(&def) as i32 },
            hint: PropertyHint::None,
            usage: PropertyUsage::DEFAULT,
            default: def,

            _t: PhantomData,
        }
    }
    pub fn signal(&mut self, name: &str) -> SignalBuilder<C>
    {
        SignalBuilder {
            parent: self,
            name: name.into(),
        }
    }
}

pub struct SignalBuilder<'a, C: 'a> {
    parent: &'a PropertiesBuilder<C>,
    name: String,
}

impl <'a, C> SignalBuilder<'a, C> {

    pub fn register(self) {
        use std::ptr;
        unsafe {
            let api = get_api();

            let name = (api.godot_string_chars_to_utf8_with_len)(self.name.as_ptr() as *const _, self.name.len() as _);
            let signal = sys::godot_signal {
                name: name,
                num_args: 0,
                args: ptr::null_mut(),
                num_default_args: 0,
                default_args: ptr::null_mut(),
            };

            (api.godot_nativescript_register_signal)(self.parent.desc, self.parent.class_name, &signal);
        }
    }
}

pub struct PropertyBuilder<'a, C: 'a, S, G, T> {
    parent: &'a PropertiesBuilder<C>,
    name: String,
    setter: S,
    getter: G,

    ty: i32,
    hint: PropertyHint,
    usage: PropertyUsage,
    default: sys::godot_variant,

    _t: PhantomData<(C, T)>,
}

impl <'a, C, S, G, T> PropertyBuilder<'a, C, S, G, T>
    where T: GodotType,
          C: GodotClass,
          S: GodotSetFunction<C, T>,
          G: GodotGetFunction<C, T>,
{
    pub fn hint(mut self, hint: PropertyHint) -> PropertyBuilder<'a, C, S, G, T> {
        self.hint = hint;
        self
    }
    pub fn usage(mut self, usage: PropertyUsage) -> PropertyBuilder<'a, C, S, G, T> {
        self.usage = usage;
        self
    }

    pub fn register(self) {
        use std::mem;
        use sys::godot_property_hint::*;
        unsafe {
            let api = get_api();
            let mut hint_text = None;
            let hint = match self.hint {
                PropertyHint::None => GODOT_PROPERTY_HINT_NONE,
                PropertyHint::Range{min, max, step, slider} => {
                    if slider {
                        hint_text = Some(format!("{},{},{},slider", min, max, step));
                    } else {
                        hint_text = Some(format!("{},{},{}", min, max, step));
                    }
                    GODOT_PROPERTY_HINT_RANGE
                },
                PropertyHint::Enum{values} => {
                    hint_text = Some(values.join(","));
                    GODOT_PROPERTY_HINT_ENUM
                },
                PropertyHint::Flags{values} => {
                    hint_text = Some(values.join(","));
                    GODOT_PROPERTY_HINT_FLAGS
                }
                PropertyHint::NodePathToEditedNode => GODOT_PROPERTY_HINT_NODE_PATH_TO_EDITED_NODE,
            };
            let hint_string = if let Some(text) = hint_text {
                (api.godot_string_chars_to_utf8_with_len)(text.as_ptr() as *const _, text.len() as _)
            } else {
                sys::godot_string::default()
            };
            let mut attr = sys::godot_property_attributes {
                rset_type: sys::godot_method_rpc_mode::GODOT_METHOD_RPC_MODE_DISABLED, // TODO:

                type_: self.ty,

                hint: hint,
                hint_string: hint_string,

                usage: mem::transmute(self.usage),
                default_value: self.default,
            };
            let path = ::std::ffi::CString::new(self.name).unwrap();

            let set = self.setter.as_godot_function();
            let get = self.getter.as_godot_function();

            (api.godot_nativescript_register_property)(self.parent.desc, self.parent.class_name, path.as_ptr() as *const _, &mut attr, set, get);
        }
    }
}

impl <'a, C, G, T> PropertyBuilder<'a, C, (), G, T>
    where T: GodotType,
          C: GodotClass,
{
    pub fn setter<S>(self, s: S) -> PropertyBuilder<'a, C, S, G, T>
        where S: GodotSetFunction<C, T>
    {
        PropertyBuilder {
            parent: self.parent,
            name: self.name,
            setter: s,
            getter: self.getter,
            ty: self.ty,
            hint: self.hint,
            usage: self.usage,
            default: self.default,
            _t: PhantomData,
        }
    }
}

impl <'a, C, S, T> PropertyBuilder<'a, C, S, (), T>
    where T: GodotType,
          C: GodotClass,
{
    pub fn getter<G>(self, g: G) -> PropertyBuilder<'a, C, S, G, T>
        where G: GodotGetFunction<C, T>
    {
        PropertyBuilder {
            parent: self.parent,
            name: self.name,
            setter: self.setter,
            getter: g,
            ty: self.ty,
            hint: self.hint,
            usage: self.usage,
            default: self.default,
            _t: PhantomData,
        }
    }
}

pub unsafe trait GodotSetFunction<C: GodotClass, T: GodotType> {
    unsafe fn as_godot_function(self) -> sys::godot_property_set_func;
}

pub unsafe trait GodotGetFunction<C: GodotClass, T: GodotType> {
    unsafe fn as_godot_function(self) -> sys::godot_property_get_func;
}

unsafe impl <C: GodotClass, T: GodotType> GodotSetFunction<C, T> for () {
    unsafe fn as_godot_function(self) -> sys::godot_property_set_func {
        sys::godot_property_set_func::default()
    }
}

unsafe impl <C: GodotClass, T: GodotType> GodotGetFunction<C, T> for () {
    unsafe fn as_godot_function(self) -> sys::godot_property_get_func {
        sys::godot_property_get_func::default()
    }
}

unsafe impl <F, C, T> GodotSetFunction<C, T> for F
    where C: GodotClass,
          T: GodotType,
          F: Fn(&mut C, T),
{
    unsafe fn as_godot_function(self) -> sys::godot_property_set_func {
        use std::cell::RefCell;
        let mut set = sys::godot_property_set_func::default();
        let data = Box::new(self);
        set.method_data = Box::into_raw(data) as *mut _;

        extern "C" fn invoke<C, F, T>(_this: *mut sys::godot_object, method: *mut libc::c_void, class: *mut libc::c_void, val: *mut sys::godot_variant)
            where C: GodotClass,
                T: GodotType,
                F: Fn(&mut C, T),

        {
            unsafe {
                let rust_ty = &*(class as *mut RefCell<C>);
                let mut rust_ty = rust_ty.borrow_mut();
                let func = &mut *(method as *mut F);

                if let Some(val) = T::from_variant(Variant::cast_ref(val)) {
                    func(&mut *rust_ty, val);
                } else {
                    godot_error!("Incorrect type passed to property");
                }
            }
        }
        set.set_func = Some(invoke::<C, F, T>);

        extern "C" fn free_func<F>(data: *mut libc::c_void) {
            unsafe {
                drop(Box::from_raw(data as *mut F));
            }
        }
        set.free_func = Some(free_func::<F>);

        set
    }
}


unsafe impl <F, C, T> GodotGetFunction<C, T> for F
    where C: GodotClass,
          T: GodotType,
          F: Fn(&mut C) -> T,
{
    unsafe fn as_godot_function(self) -> sys::godot_property_get_func {
        use std::cell::RefCell;
        let mut get = sys::godot_property_get_func::default();
        let data = Box::new(self);
        get.method_data = Box::into_raw(data) as *mut _;

        extern "C" fn invoke<C, F, T>(_this: *mut sys::godot_object, method: *mut libc::c_void, class: *mut libc::c_void) -> sys::godot_variant
            where C: GodotClass,
                T: GodotType,
                F: Fn(&mut C) -> T,

        {
            unsafe {
                let rust_ty = &*(class as *mut RefCell<C>);
                let mut rust_ty = rust_ty.borrow_mut();
                let func = &mut *(method as *mut F);
                let ret = func(&mut *rust_ty);
                ret.to_variant().forget()
            }
        }
        get.get_func = Some(invoke::<C, F, T>);

        extern "C" fn free_func<F>(data: *mut libc::c_void) {
            unsafe {
                drop(Box::from_raw(data as *mut F));
            }
        }
        get.free_func = Some(free_func::<F>);

        get
    }
}