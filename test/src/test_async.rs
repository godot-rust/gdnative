use std::{cell::RefCell, sync::Arc};

use gdnative::{prelude::*, tasks::Context};

pub(crate) fn run_tests() -> bool {
    // Relevant tests in GDScript
    true
}

thread_local! {
    static EXECUTOR: &'static SharedLocalPool = {
        Box::leak(Box::default())
    };
}

#[cfg(not(feature = "no-manual-register"))]
pub(crate) fn register(handle: InitHandle) {
    gdnative::tasks::register_runtime(&handle);
    gdnative::tasks::set_executor(EXECUTOR.with(|e| *e));

    handle.add_class::<AsyncMethods>();
    handle.add_class::<AsyncExecutorDriver>();
}

#[cfg(feature = "no-manual-register")]
pub(crate) fn register(handle: InitHandle) {
    gdnative::tasks::register_runtime(&handle);
    gdnative::tasks::set_executor(EXECUTOR.with(|e| *e));
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
    #[method]
    fn _process(&self, _delta: f64) {
        EXECUTOR.with(|e| e.pool.borrow_mut().run_until_stalled());
    }
}

#[derive(NativeClass)]
#[inherit(Reference)]
struct AsyncMethods;

#[methods]
impl AsyncMethods {
    fn new(_owner: TRef<Reference>) -> Self {
        AsyncMethods
    }

    #[method(async)]
    fn resume_add(
        &self,
        #[async_ctx] ctx: Arc<Context>,
        a: i32,
        obj: Ref<Object>,
        name: String,
    ) -> impl std::future::Future<Output = i32> + 'static {
        async move {
            let b = ctx.until_resume().await;
            let b = i32::from_variant(&b).unwrap();

            let c = unsafe { obj.assume_safe().call(name, &[]) };
            let c = Ref::<Reference>::from_variant(&c).unwrap();
            let c = unsafe { c.assume_safe() };
            let c = ctx.signal(c, "completed").unwrap().await;
            assert_eq!(1, c.len());
            let c = i32::from_variant(&c[0]).unwrap();

            a + b + c
        }
    }
}
