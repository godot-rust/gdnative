use gdnative_bindings::Reference;
use gdnative_core::core_types::{ToVariant, Variant, VariantType};
use gdnative_core::godot_site;
use gdnative_core::nativescript::export::method::StaticArgs;
use gdnative_core::nativescript::export::method::StaticArgsMethod;
use gdnative_core::nativescript::export::{
    ClassBuilder, ExportInfo, PropertyUsage, Signal, SignalArgument,
};
use gdnative_core::nativescript::user_data::LocalCellData;
use gdnative_core::nativescript::user_data::{Map, MapMut};
use gdnative_core::nativescript::{Instance, NativeClass, NativeClassMethods, RefInstance};
use gdnative_core::object::ownership::{Shared, Unique};
use gdnative_derive::FromVarargs;

use crate::future::Resume;

pub(crate) struct FuncState {
    kind: Kind,
}

enum Kind {
    Resolved(Variant),
    Resumable(Resume<Variant>),
    Pending,
}

impl NativeClass for FuncState {
    type Base = Reference;
    type UserData = LocalCellData<FuncState>;

    fn class_name() -> &'static str {
        // Sort of just praying that there will be no duplicates of this.
        "__GDNATIVE_ASYNC_INTERNAL__FuncState"
    }

    fn register_properties(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "completed",
            args: &[SignalArgument {
                name: "value",
                default: Variant::new(),
                export_info: ExportInfo::new(VariantType::Nil),
                usage: PropertyUsage::DEFAULT,
            }],
        });

        builder.add_signal(Signal {
            name: "resumable",
            args: &[],
        });
    }
}

impl FuncState {
    pub fn new() -> Instance<Self, Unique> {
        assert!(
            super::REGISTRATION.get().is_some(),
            "async API must be registered before any async methods can be called"
        );

        Instance::emplace(FuncState {
            kind: Kind::Pending,
        })
    }
}

pub(super) fn resolve(this: RefInstance<'_, FuncState, Shared>, value: Variant) {
    this.script()
        .map_mut(|s| {
            match s.kind {
                Kind::Resolved(_) => {
                    panic!("`resolve` should only be called once for each FuncState")
                }
                Kind::Pending => {}
                Kind::Resumable(_) => {
                    gdnative_core::log::warn(
                        Default::default(),
                        "async function resolved while waiting for a `resume` call",
                    );
                }
            }

            s.kind = Kind::Resolved(value.clone());
        })
        .expect("no reentrancy");

    this.base().emit_signal("completed", &[value]);
}

pub(super) fn make_resumable(this: RefInstance<'_, FuncState, Shared>, resume: Resume<Variant>) {
    let kind = this
        .script()
        .map_mut(|s| std::mem::replace(&mut s.kind, Kind::Resumable(resume)))
        .expect("no reentrancy");

    match kind {
        Kind::Resolved(_) => {
            panic!("`make_resumable` should not be called after resolution")
        }
        Kind::Resumable(_) => {
            gdnative_core::log::warn(
                Default::default(),
                "`make_resumable` called when there is a previous pending future",
            );
        }
        Kind::Pending => {
            this.base().emit_signal("resumable", &[]);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct IsValidFn;

#[derive(FromVarargs)]
struct IsValidArgs {
    #[opt]
    extended_check: Option<bool>,
}

impl StaticArgsMethod<FuncState> for IsValidFn {
    type Args = IsValidArgs;
    fn call(&self, this: RefInstance<'_, FuncState, Shared>, args: Self::Args) -> Variant {
        if args.extended_check.is_some() {
            gdnative_core::log::warn(
                Self::site().unwrap(),
                "`extended_check` is set, but it has no effect on Rust function state objects",
            )
        }

        this.script()
            .map(|s| match &s.kind {
                Kind::Resumable(_) => true,
                Kind::Resolved(_) | Kind::Pending => false,
            })
            .unwrap()
            .to_variant()
    }
    fn site() -> Option<gdnative_core::log::Site<'static>> {
        Some(godot_site!(FunctionState::is_valid))
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ResumeFn;

#[derive(FromVarargs)]
struct ResumeArgs {
    #[opt]
    arg: Variant,
}

impl StaticArgsMethod<FuncState> for ResumeFn {
    type Args = ResumeArgs;
    fn call(&self, this: RefInstance<'_, FuncState, Shared>, args: Self::Args) -> Variant {
        this.map_mut(
            |s, owner| match std::mem::replace(&mut s.kind, Kind::Pending) {
                Kind::Resumable(resume) => {
                    resume.resume(args.arg);
                    owner.to_variant()
                }
                Kind::Pending => owner.to_variant(),
                Kind::Resolved(result) => {
                    s.kind = Kind::Resolved(result.clone());
                    result
                }
            },
        )
        .expect("no reentrancy")
    }
    fn site() -> Option<gdnative_core::log::Site<'static>> {
        Some(godot_site!(FunctionState::is_valid))
    }
}

impl NativeClassMethods for FuncState {
    fn register(builder: &ClassBuilder<Self>) {
        builder
            .build_method("is_valid", StaticArgs::new(IsValidFn))
            .done_stateless();
        builder
            .build_method("resume", StaticArgs::new(ResumeFn))
            .done_stateless();
    }
}
