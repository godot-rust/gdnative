use crate::core_types::{GodotString, Variant, VariantType};
use crate::export::{ClassBuilder, ExportInfo, NativeClass, PropertyUsage};

/// Class to construct a signal. Make sure to call [`Self::done()`] in the end.
///
/// Signal parameters can be added with the various `param*()` methods.
/// Keep in mind that unlike function parameters, signal parameters (both their lengths and types)
/// are not statically checked in Godot. The parameter signature you specify is simply to assist you
/// in the editor UI and with auto-generation of GDScript signal handlers.
#[must_use = "SignalBuilder left unbuilt -- did you forget to call done()?"]
pub struct SignalBuilder<'a, C> {
    class_builder: &'a ClassBuilder<C>,
    name: GodotString,
    args: Vec<SignalParam>,
}

impl<'a, C: NativeClass> SignalBuilder<'a, C> {
    pub(super) fn new(class_builder: &'a ClassBuilder<C>, signal_name: GodotString) -> Self {
        Self {
            class_builder,
            name: signal_name,
            args: vec![],
        }
    }

    /// Add a parameter for the signal with a name and type.
    ///
    /// Note that GDScript signal parameters are generally untyped and not checked at runtime.
    /// The type is solely used for UI purposes.
    #[inline]
    pub fn with_param(self, parameter_name: &str, parameter_type: VariantType) -> Self {
        self.with_param_custom(SignalParam {
            name: parameter_name.into(),
            default: Variant::nil(),
            export_info: ExportInfo::new(parameter_type),
            usage: PropertyUsage::DEFAULT,
        })
    }

    /// Add a parameter for the signal with a name and default value.
    ///
    /// The type is inferred from the default value.
    /// Note that GDScript signal parameters are generally untyped and not checked at runtime.
    /// The type is solely used for UI purposes.
    #[inline]
    pub fn with_param_default(self, parameter_name: &str, default_value: Variant) -> Self {
        let variant_type = default_value.get_type();

        self.with_param_custom(SignalParam {
            name: parameter_name.into(),
            default: default_value,
            export_info: ExportInfo::new(variant_type),
            usage: PropertyUsage::DEFAULT,
        })
    }

    /// Add a (untyped) parameter for the signal with a name.
    ///
    /// Types are not required or checked at runtime, but they help for editor UI and auto-generation of signal listeners.
    #[inline]
    pub fn with_param_untyped(self, parameter_name: &str) -> Self {
        // Note: the use of 'Nil' to express "untyped" is not following official documentation and could be improved.

        self.with_param_custom(SignalParam {
            name: parameter_name.into(),
            default: Variant::nil(),
            export_info: ExportInfo::new(VariantType::Nil),
            usage: PropertyUsage::DEFAULT,
        })
    }

    /// Add a parameter for the signal, manually configured.
    #[inline]
    pub fn with_param_custom(mut self, parameter: SignalParam) -> Self {
        self.args.push(parameter);
        self
    }

    /// Finish registering the signal.
    #[inline]
    pub fn done(self) {
        self.class_builder.add_signal(Signal {
            name: self.name,
            args: self.args,
        });
    }
}

pub(crate) struct Signal {
    pub name: GodotString,
    pub args: Vec<SignalParam>,
}

/// Parameter in a signal declaration.
///
/// Instead of providing values for each field, check out the `param*()` methods in [`SignalBuilder`].
pub struct SignalParam {
    /// Parameter name.
    pub name: GodotString,

    /// Default value, used when no argument is provided.
    pub default: Variant,

    /// Metadata and UI hints about exporting, e.g. parameter type.
    pub export_info: ExportInfo,

    /// In which context the signal parameter is used.
    pub usage: PropertyUsage,
}
