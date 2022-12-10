use gdnative::export::user_data::Once;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_map_owned();

    status
}

#[cfg(not(feature = "no-manual-register"))]
pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<VecBuilder>();
}

#[cfg(feature = "no-manual-register")]
pub(crate) fn register(_handle: InitHandle) {}

#[derive(NativeClass)]
#[no_constructor]
#[inherit(Reference)]
#[user_data(Once<Self>)]
struct VecBuilder {
    v: Vec<i32>,
}

#[methods]
impl VecBuilder {
    #[method]
    fn append(mut self, mut numbers: Vec<i32>) -> Instance<Self> {
        self.v.append(&mut numbers);
        Instance::emplace(Self { v: self.v }).into_shared()
    }
}

crate::godot_itest! { test_map_owned {
    let v1 = Instance::emplace(VecBuilder { v: Vec::new() }).into_shared();
    let v1 = unsafe { v1.assume_safe() };

    let v2 = v1
        .map_owned(|s, _base| s.append(vec![1, 2, 3]))
        .unwrap();
    let v2 = unsafe { v2.assume_safe() };
    assert!(v1
        .map_owned(|_, _| panic!("should never be called"))
        .is_err());

    let v3 = v2
        .map_owned(|s, _base| s.append(vec![4, 5, 6]))
        .unwrap();
    let v3 = unsafe { v3.assume_safe() };
    assert!(v2
        .map_owned(|_, _| panic!("should never be called"))
        .is_err());

    let v = v3.map_owned(|s, _| s.v).unwrap();
    assert_eq!(&v, &[1, 2, 3, 4, 5, 6]);
    assert!(v3
        .map_owned(|_, _| panic!("should never be called"))
        .is_err());
}}
