use std::cell::RefCell;

use gdnative::asn::{Async, AsyncMethod, Spawner};
use gdnative::prelude::*;

pub(crate) fn run_tests() -> bool {
    // Relevant tests in GDScript
    true
}

thread_local! {
    static EXECUTOR: &'static SharedLocalPool = {
        Box::leak(Box::new(SharedLocalPool::default()))
    };
}

pub(crate) fn register(handle: InitHandle) {
    gdnative::asn::register_runtime(&handle).unwrap();
    gdnative::asn::set_executor(EXECUTOR.with(|e| *e)).unwrap();

    handle.add_class::<AsyncMethods>();
    handle.add_class::<AsyncExecutorDriver>();
}

#[derive(Default)]
struct SharedLocalPool {
    pool: RefCell<futures::executor::LocalPool>,
}

impl futures::task::LocalSpawn for SharedLocalPool {
    fn spawn_local_obj(
        &self,
        future: futures::task::LocalFutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        self.pool.borrow_mut().spawner().spawn_local_obj(future)
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
struct AsyncExecutorDriver;

impl AsyncExecutorDriver {
    fn new(_owner: &Node) -> Self {
        AsyncExecutorDriver
    }
}

#[methods]
impl AsyncExecutorDriver {
    #[export]
    fn _process(&self, _owner: &Node, _delta: f64) {
        EXECUTOR.with(|e| e.pool.borrow_mut().run_until_stalled());
    }
}

#[derive(NativeClass)]
#[inherit(Reference)]
#[register_with(register_methods)]
struct AsyncMethods;

#[methods]
impl AsyncMethods {
    fn new(_owner: TRef<Reference>) -> Self {
        AsyncMethods
    }
}

struct ResumeAddFn;

impl AsyncMethod<AsyncMethods> for ResumeAddFn {
    fn spawn_with(&self, spawner: Spawner<'_, AsyncMethods>) {
        spawner.spawn(|ctx, _this, mut args| {
            let a = args.read::<i32>().get().unwrap();
            let obj = args.read::<Ref<Object>>().get().unwrap();
            let name = args.read::<GodotString>().get().unwrap();

            async move {
                let b = ctx.until_resume().await;
                let b = i32::from_variant(&b).unwrap();

                let c = unsafe { obj.assume_safe().call(name, &[]) };
                let c = Ref::<Reference>::from_variant(&c).unwrap();
                let c = unsafe { c.assume_safe() };
                let c = ctx.signal(c, "completed").unwrap().await;
                assert_eq!(1, c.len());
                let c = i32::from_variant(&c[0]).unwrap();

                (a + b + c).to_variant()
            }
        });
    }
}

fn register_methods(builder: &ClassBuilder<AsyncMethods>) {
    builder
        .build_method("resume_add", Async::new(ResumeAddFn))
        .done();
}
