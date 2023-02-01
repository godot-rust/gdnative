use gdnative::export::hint::{ArrayHint, IntHint, RangeHint};
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
#[register_with(Self::register)]
struct ExportsArrays;

#[methods]
impl ExportsArrays {
    fn new(_owner: &Node) -> Self {
        ExportsArrays
    }

    fn register(builder: &ClassBuilder<Self>) {
        builder
            .property::<VariantArray>("single_array")
            .with_setter(ExportsArrays::set_single_array)
            .done();

        builder
            .property::<VariantArray>("single_array_range")
            .with_setter(ExportsArrays::set_single_array_range)
            .with_hint(ArrayHint::with_element_hint::<i64>(IntHint::Range(
                RangeHint::new(-5, 5),
            )))
            .done();

        builder
            .property::<VariantArray>("double_array")
            .with_setter(ExportsArrays::set_double_array)
            .with_hint(ArrayHint::with_element_hint::<VariantArray>(
                ArrayHint::new(),
            ))
            .done();

        builder
            .property::<VariantArray>("double_array_range")
            .with_setter(ExportsArrays::set_double_array_range)
            .with_hint(ArrayHint::with_element_hint::<VariantArray>(
                ArrayHint::with_element_hint::<i64>(IntHint::Range(RangeHint::new(-5, 5))),
            ))
            .done();
    }

    fn set_single_array(&mut self, _owner: TRef<Node>, value: VariantArray) {
        godot_print!("Single: {:?}", value);
    }
    fn set_single_array_range(&mut self, _owner: TRef<Node>, value: VariantArray) {
        godot_print!("Single Range: {:?}", value);
    }
    fn set_double_array(&mut self, _owner: TRef<Node>, value: VariantArray) {
        godot_print!("Double: {:?}", value);
    }
    fn set_double_array_range(&mut self, _owner: TRef<Node>, value: VariantArray) {
        godot_print!("Double Range: {:?}", value);
    }
}

struct BuilderExportLibrary;

#[gdnative::init::callbacks]
impl GDNativeCallbacks for BuilderExportLibrary {
    fn nativescript_init(handle: InitHandle) {
        handle.add_class::<ExportsArrays>();
    }
}
