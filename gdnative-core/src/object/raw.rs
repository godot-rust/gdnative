use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ptr::{self, NonNull};

use crate::core_types::GodotString;
use crate::object::memory::RefCounted;
use crate::private::get_api;
use crate::sys;

use super::GodotObject;

/// An opaque struct representing Godot objects. This should never be created on the stack.
///
/// This is an internal interface. Users are expected to use references to named generated types
/// instead.
#[repr(C)]
pub struct RawObject<T> {
    _opaque: [u8; 0],
    _marker: PhantomData<(T, *const ())>,
}

impl<T: GodotObject> RawObject<T> {
    /// Creates a typed reference from a pointer, without checking the type of the pointer.
    ///
    /// # Safety
    ///
    /// The `obj` pointer must be pointing to a valid Godot object of type `T` during the
    /// entirety of `'a`.
    #[inline]
    pub unsafe fn from_sys_ref_unchecked<'a>(obj: NonNull<sys::godot_object>) -> &'a Self {
        &*(obj.as_ptr() as *mut Self)
    }

    /// Creates a typed reference from a pointer if the pointer is pointing to an object of
    /// the correct type. Returns `None` otherwise.
    ///
    /// # Safety
    ///
    /// The `obj` pointer must be pointing to a valid Godot object during the entirety of `'a`.
    #[inline]
    pub unsafe fn try_from_sys_ref<'a>(obj: NonNull<sys::godot_object>) -> Option<&'a Self> {
        if ptr_is_class(obj.as_ptr(), T::class_name()) {
            Some(Self::from_sys_ref_unchecked(obj))
        } else {
            None
        }
    }

    /// Casts a reference to this opaque object to `*const sys::godot_object`.
    #[inline]
    pub fn sys(&self) -> NonNull<sys::godot_object> {
        // SAFETY: references should never be null
        unsafe { NonNull::new_unchecked(self as *const _ as *mut _) }
    }

    /// Checks whether the object is of a certain Godot class.
    #[inline]
    pub fn is_class<U: GodotObject>(&self) -> bool {
        self.is_class_by_name(U::class_name())
    }

    /// Checks whether the object is of a certain Godot class by name.
    #[inline]
    pub fn is_class_by_name(&self, class_name: &str) -> bool {
        unsafe { ptr_is_class(self.sys().as_ptr(), class_name) }
    }

    /// Returns the class name of this object dynamically using `Object::get_class`.
    #[inline]
    pub fn class_name(&self) -> String {
        let api = crate::private::get_api();
        let get_class_method = crate::private::ObjectMethodTable::get(api).get_class;
        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut class_name = sys::godot_string::default();
        let ret_ptr = &mut class_name as *mut sys::godot_string;

        unsafe {
            (api.godot_method_bind_ptrcall)(
                get_class_method,
                self.sys().as_ptr(),
                argument_buffer.as_mut_ptr() as *mut _,
                ret_ptr as *mut _,
            );
        }

        let string = GodotString::from_sys(class_name);
        string.to_string()
    }

    /// Attempt to cast a Godot object to a different class type.
    #[inline]
    pub fn cast<U>(&self) -> Option<&RawObject<U>>
    where
        U: GodotObject,
    {
        unsafe { RawObject::try_from_sys_ref(self.sys()) }
    }

    /// Attempt to cast a Godot object to a different class type without checking the type at
    /// runtime.
    ///
    /// # Safety
    ///
    /// The types must be compatible.
    #[inline]
    pub unsafe fn cast_unchecked<U>(&self) -> &RawObject<U>
    where
        U: GodotObject,
    {
        RawObject::from_sys_ref_unchecked(self.sys())
    }

    /// Free the underlying object.
    ///
    /// # Safety
    ///
    /// Further operations must not be performed on the same reference.
    #[inline]
    pub unsafe fn free(&self) {
        (get_api().godot_object_destroy)(self.sys().as_ptr());
    }
}

impl<T: GodotObject<Memory = RefCounted>> RawObject<T> {
    /// Increase the reference count of the object.
    #[inline]
    pub fn add_ref(&self) {
        let api = crate::private::get_api();
        let addref_method = crate::private::ReferenceMethodTable::get(api).reference;
        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut ok = false;
        let ok_ptr = &mut ok as *mut bool;

        unsafe {
            (api.godot_method_bind_ptrcall)(
                addref_method,
                self.sys().as_ptr(),
                argument_buffer.as_mut_ptr() as *mut _,
                ok_ptr as *mut _,
            );
        }

        // If this assertion blows up it means there is a reference counting bug
        // and we tried to increment the ref count of a dead object (who's ref
        // count is equal to zero).
        debug_assert!(ok);
    }

    /// Decrease the reference count of the object. Returns `true` if this is the last
    /// reference.
    ///
    /// # Safety
    ///
    /// Further operations must not be performed on the same reference if this is the last
    /// reference.
    #[inline]
    pub unsafe fn unref(&self) -> bool {
        let api = crate::private::get_api();
        let unref_method = crate::private::ReferenceMethodTable::get(api).unreference;

        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut last_reference = false;
        let ret_ptr = &mut last_reference as *mut bool;
        (api.godot_method_bind_ptrcall)(
            unref_method,
            self.sys().as_ptr(),
            argument_buffer.as_mut_ptr() as *mut _,
            ret_ptr as *mut _,
        );

        last_reference
    }

    /// Decrease the reference count of the object. Frees the object and returns `true` if this
    /// is the last reference.
    ///
    /// # Safety
    ///
    /// Further operations must not be performed on the same reference if this is the last
    /// reference.
    #[inline]
    pub unsafe fn unref_and_free_if_last(&self) -> bool {
        let last_reference = self.unref();

        if last_reference {
            self.free();
        }

        last_reference
    }

    /// Initialize the reference count of the object.
    ///
    /// # Safety
    ///
    /// This function assumes that no other references are held at the time.
    #[inline]
    pub unsafe fn init_ref_count(&self) {
        let obj = self.sys().as_ptr();

        let api = crate::private::get_api();
        let init_method = crate::private::ReferenceMethodTable::get(api).init_ref;

        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut ok = false;
        let ret_ptr = &mut ok as *mut bool;
        (api.godot_method_bind_ptrcall)(
            init_method,
            obj,
            argument_buffer.as_mut_ptr() as *mut _,
            ret_ptr as *mut _,
        );

        debug_assert!(ok);
    }
}

impl<T: GodotObject> Debug for RawObject<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:p})", T::class_name(), self.sys())
    }
}

/// Checks whether the raw object pointer is of a certain Godot class.
///
/// # Safety
///
/// The `obj` pointer must be pointing to a valid Godot object.
#[inline]
unsafe fn ptr_is_class(obj: *mut sys::godot_object, class_name: &str) -> bool {
    let api = crate::private::get_api();
    let method_bind = crate::private::ObjectMethodTable::get(api).is_class;

    let mut class_name = (api.godot_string_chars_to_utf8_with_len)(
        class_name.as_ptr() as *const _,
        class_name.len() as _,
    );

    let mut argument_buffer = [ptr::null() as *const libc::c_void; 1];
    argument_buffer[0] = (&class_name) as *const _ as *const _;

    let mut ret = false;
    let ret_ptr = &mut ret as *mut _;
    (api.godot_method_bind_ptrcall)(
        method_bind,
        obj,
        argument_buffer.as_mut_ptr() as *mut _,
        ret_ptr as *mut _,
    );

    (api.godot_string_destroy)(&mut class_name);

    ret
}
