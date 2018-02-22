use super::*;
use godot_type::GodotType;
use std::marker::PhantomData;
use sys::godot_property_usage_flags::*;
use sys::godot_property_hint::*;
use std::mem;

pub struct PropertiesBuilder<C> {
    #[doc(hidden)]
    pub desc: *mut libc::c_void,
    #[doc(hidden)]
    pub class_name: *const libc::c_char,
    #[doc(hidden)]
    pub class: PhantomData<C>,
}

// TODO: missing property hints.
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

impl PropertyHint {
    fn to_sys(&self) -> sys::godot_property_hint {
        match *self {
            PropertyHint::None => GODOT_PROPERTY_HINT_NONE,
            PropertyHint::Range { .. } => GODOT_PROPERTY_HINT_RANGE,
            PropertyHint::Enum { .. } => GODOT_PROPERTY_HINT_ENUM,
            PropertyHint::Flags { .. } => GODOT_PROPERTY_HINT_FLAGS,
            PropertyHint::NodePathToEditedNode => GODOT_PROPERTY_HINT_NODE_PATH_TO_EDITED_NODE,
        }
    }
}

bitflags! {
    pub struct PropertyUsage: u32 {
        const STORAGE = GODOT_PROPERTY_USAGE_STORAGE as u32;
        const EDITOR = GODOT_PROPERTY_USAGE_EDITOR as u32;
        const NETWORK = GODOT_PROPERTY_USAGE_NETWORK as u32;
        const EDITOR_HELPER = GODOT_PROPERTY_USAGE_EDITOR_HELPER as u32;
        const CHECKABLE = GODOT_PROPERTY_USAGE_CHECKABLE as u32;
        const CHECKED = GODOT_PROPERTY_USAGE_CHECKED as u32;
        const INTERNATIONALIZED = GODOT_PROPERTY_USAGE_INTERNATIONALIZED as u32;
        const GROUP = GODOT_PROPERTY_USAGE_GROUP as u32;
        const CATEGORY = GODOT_PROPERTY_USAGE_CATEGORY as u32;
        const STORE_IF_NONZERO = GODOT_PROPERTY_USAGE_STORE_IF_NONZERO as u32;
        const STORE_IF_NONONE = GODOT_PROPERTY_USAGE_STORE_IF_NONONE as u32;
        const NO_INSTANCE_STATE = GODOT_PROPERTY_USAGE_NO_INSTANCE_STATE as u32;
        const RESTART_IF_CHANGED = GODOT_PROPERTY_USAGE_RESTART_IF_CHANGED as u32;
        const SCRIPT_VARIABLE  = GODOT_PROPERTY_USAGE_SCRIPT_VARIABLE as u32;
        const STORE_IF_NULL = GODOT_PROPERTY_USAGE_STORE_IF_NULL as u32;
        const ANIMATE_AS_TRIGGER = GODOT_PROPERTY_USAGE_ANIMATE_AS_TRIGGER as u32;
        const UPDATE_ALL_IF_MODIFIED = GODOT_PROPERTY_USAGE_UPDATE_ALL_IF_MODIFIED as u32;

        const DEFAULT = Self::STORAGE.bits | Self::EDITOR.bits | Self::NETWORK.bits;
        const DEFAULT_INTL = Self::DEFAULT.bits | Self::INTERNATIONALIZED.bits;
        const NOEDITOR = Self::STORAGE.bits | Self::NETWORK.bits;
    }
}

impl PropertyUsage {
    fn to_sys(&self) -> sys::godot_property_usage_flags {
        unsafe { mem::transmute(*self) }
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
        unsafe {
            let api = get_api();
            let hint_text = match self.hint {
                PropertyHint::Range{min, max, step, slider} => {
                    if slider {
                        Some(format!("{},{},{},slider", min, max, step))
                    } else {
                        Some(format!("{},{},{}", min, max, step))
                    }
                }
                PropertyHint::Enum { values } => { Some(values.join(",")) }
                PropertyHint::Flags { values } => { Some(values.join(",")) }
                PropertyHint::NodePathToEditedNode => { None }
                PropertyHint::None => { None }
            };
            let hint_string = if let Some(text) = hint_text {
                GodotString::from_str(text)
            } else {
                GodotString::default()
            };
            let mut attr = sys::godot_property_attributes {
                rset_type: sys::godot_method_rpc_mode::GODOT_METHOD_RPC_MODE_DISABLED, // TODO:

                type_: self.ty,

                hint: self.hint.to_sys(),
                hint_string: hint_string.forget(),

                usage: self.usage.to_sys(),
                default_value: self.default,
            };
            let path = ::std::ffi::CString::new(self.name).unwrap();

            let set = self.setter.as_godot_function();
            let get = self.getter.as_godot_function();

            (api.godot_nativescript_register_property)(
                self.parent.desc,
                self.parent.class_name,
                path.as_ptr() as *const _,
                &mut attr, set, get
            );
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