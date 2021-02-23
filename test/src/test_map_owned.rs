use gdnative::nativescript::user_data::Once;
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    let mut status = true;

    status &= test_map_owned();

    status
}

pub(crate) fn register(handle: InitHandle) {
    handle.add_class::<VecBuilder>();
}

#[derive(NativeClass)]
#[no_constructor]
#[inherit(Reference)]
#[user_data(Once<Self>)]
struct VecBuilder {
    v: Vec<i32>,
}

#[methods]
impl VecBuilder {
    #[export]
    fn append(mut self, _owner: TRef<Reference>, mut numbers: Vec<i32>) -> Instance<Self, Shared> {
        self.v.append(&mut numbers);
        Instance::emplace(Self { v: self.v }).into_shared()
    }
}

fn test_map_owned() -> bool {
    println!(" -- test_map_owned");

    let ok = std::panic::catch_unwind(|| {
        let v1 = Instance::emplace(VecBuilder { v: Vec::new() }).into_shared();
        let v1 = unsafe { v1.assume_safe() };

        let v2 = v1
            .map_owned(|s, owner| s.append(owner, vec![1, 2, 3]))
            .unwrap();
        let v2 = unsafe { v2.assume_safe() };
        assert!(v1
            .map_owned(|_, _| panic!("should never be called"))
            .is_err());

        let v3 = v2
            .map_owned(|s, owner| s.append(owner, vec![4, 5, 6]))
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
    })
    .is_ok();

    if !ok {
        gdnative::godot_error!("   !! Test test_map_owned failed");
    }

    ok
}
