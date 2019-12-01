use gdnative::{
    GodotError,
    GodotString,
    init::{SignalArgument, Signal},
    Object,
    Node,
    ToVariant,
    Variant,
    VariantArray,
};

// You can also gather events under an
// [event bus](https://www.gdquest.com/open-source/guidelines/godot-gdscript/#events-bus-observer-pattern-for-godot)
// of sorts. But it's usually a good idea to keep them in a big laundry list, segregated by
// purpose, then alphabetically ordered.
//
// You can also generate these dynamically, but that's an advanced topic that this example doesn't
// cover. However, this will cover connecting and disconnecting signals dynamically.
//
// You can probably write a proc macro for everything in here. Or you can just list everything out.
pub(crate) enum Event {
    TextChanged,
    Pressed,
    ConnectionStatus,
}

/// Functions to connect events.
impl Event {
    // CONNECTION CONSTANTS ARE FOUND HERE: https://github.com/godotengine/godot/blob/master/core/object.h#L406-L409
    // There doesn't seem to be any docs about them...? I guess they are a little self explanatory.
    pub const NONE: i64 = 0;
    pub const DEFERRED: i64 = Object::CONNECT_DEFERRED;
    pub const PERSIST: i64 = Object::CONNECT_PERSIST;
    pub const ONESHOT: i64 = Object::CONNECT_ONESHOT;
    pub const REFERENCE_COUNTED: i64 = Object::CONNECT_REFERENCE_COUNTED;

    pub fn connect(
        &self,
        from: impl Into<Object>,
        to: impl Into<Object>,
        method_of_to_object_to_call: impl Into<GodotString>,
    ) -> Result<(), GodotError> {
        self.connect_with_binds_and_flags(
            from,
            to,
            method_of_to_object_to_call,
            VariantArray::new(),
            Event::NONE,
        )
    }
    pub fn connect_with_binds(
        &self,
        from: impl Into<Object>,
        to: impl Into<Object>,
        method_of_to_object_to_call: impl Into<GodotString>,
        binds: impl Into<VariantArray>,
    ) -> Result<(), GodotError> {
        self.connect_with_binds_and_flags(
            from,
            to,
            method_of_to_object_to_call,
            binds,
            Event::NONE,
        )
    }
    pub fn connect_with_flags<M: Into<GodotString>>(
        &self,
        from: impl Into<Object>,
        to: impl Into<Object>,
        method_of_to_object_to_call: impl Into<GodotString>,
        flags: i64,
    ) -> Result<(), GodotError> {
        self.connect_with_binds_and_flags(
            from,
            to,
            method_of_to_object_to_call,
            VariantArray::new(),
            flags,
        )
    }
    pub fn connect_with_binds_and_flags(
        &self,
        from: impl Into<Object>,
        to: impl Into<Object>,
        method_of_to_object_to_call: impl Into<GodotString>,
        binds: impl Into<VariantArray>,
        flags: i64,
    ) -> Result<(), GodotError> {
        let mut from = from.into();
        let to = to.into();
        let method_of_to_object_to_call = method_of_to_object_to_call.into();
        if (flags & Self::REFERENCE_COUNTED == Self::REFERENCE_COUNTED) && unsafe {
            from.is_connected(self.godot_name(), Some(to), method_of_to_object_to_call.new_ref())
        } {
            let from_name = unsafe { from.cast() }
                .map(|node: Node| unsafe { node.get_name() }.to_string())
                .unwrap_or("<unknown>".to_owned());
            let to_name = unsafe { to.cast() }
                .map(|node: Node| unsafe { node.get_name() }.to_string())
                .unwrap_or("<unknown>".to_owned());
            godot_warn!(
                "Signal {} was already connected from {} to {} through {}. Nothing was done.",
                self.name(),
                from_name,
                to_name,
                method_of_to_object_to_call.to_string(),
            );
            return Ok(());
        }
        let res = unsafe {
            from.connect(
                self.godot_name(),
                Some(to),
                method_of_to_object_to_call.new_ref(),
                binds.into(),
                flags,
            )
        };
        if let Err(e) = res {
            let from_name = unsafe { from.cast() }
                .map(|node: Node| unsafe { node.get_name() }.to_string())
                .unwrap_or("<unknown>".to_owned());
            let to_name = unsafe { to.cast() }
                .map(|node: Node| unsafe { node.get_name() }.to_string())
                .unwrap_or("<unknown>".to_owned());
            godot_warn!(
                "Cannot connect signal {} from {} to {} through {} due to error {:?}.",
                self.name(),
                from_name,
                to_name,
                method_of_to_object_to_call.to_string(),
                e,
            );
        }
        res
    }

    pub fn disconnect(
        &self,
        from: impl Into<Object>,
        to: impl Into<Object>,
        method_of_to_object_to_call: impl Into<GodotString>,
    ) {
        let mut from = from.into();
        let to = to.into();
        unsafe {
            from.disconnect(
                self.godot_name(),
                Some(to),
                method_of_to_object_to_call.into(),
            );
        }
    }
}

/// Isolate the methods used to generate the `Signal` for registration.
mod to_signal {
    use gdnative::{
        GodotString,
        init::{PropertyHint, PropertyUsage, SignalArgument},
        ToVariant,
    };

    // These are `str` names.
    pub(super) const TEXT_CHANGE_STR: &'static str = "text_changed";
    pub(super) const PRESSED_STR: &'static str = "pressed";
    pub(super) const CONNECTION_STATUS_STR: &'static str = "increment_count";

    // These are cached `GodotString` names.
    lazy_static::lazy_static! {
        pub(super) static ref TEXT_CHANGE_STRING: GodotString = TEXT_CHANGE_STR.into();
        pub(super) static ref PRESSED_STRING: GodotString = PRESSED_STR.into();
        pub(super) static ref CONNECTION_STATUS_STRING: GodotString = CONNECTION_STATUS_STR.into();
    }

    // These are cached `SignalArgument`s for constructing the `Signal`,
    lazy_static::lazy_static! {
        pub(super) static ref TEXT_CHANGE_ARGS: Vec<SignalArgument<'static>> = vec![];
        pub(super) static ref PRESSED_ARGS: Vec<SignalArgument<'static>> = vec![];
        pub(super) static ref CONNECTION_STATUS_ARGS: Vec<SignalArgument<'static>> = vec![
            SignalArgument {
                name: "connection_state",
                default: false.to_variant(),
                hint: PropertyHint::None,
                usage: PropertyUsage::DEFAULT,
            },
        ];
    }
}
impl Event {
    /// Get a `str` slice of the `Event`'s name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::TextChanged => to_signal::TEXT_CHANGE_STR,
            Self::Pressed => to_signal::PRESSED_STR,
            Self::ConnectionStatus => to_signal::CONNECTION_STATUS_STR,
        }
    }
    /// Get a `GodotName` with the signal's name.
    pub fn godot_name(&self) -> GodotString {
        match self {
            Self::TextChanged => to_signal::TEXT_CHANGE_STRING.new_ref(),
            Self::Pressed => to_signal::PRESSED_STRING.new_ref(),
            Self::ConnectionStatus => to_signal::CONNECTION_STATUS_STRING.new_ref(),
        }
    }
    /// Get a slice of the `SignalArgument`s passed to the signal.
    pub fn args(&self) -> &'static[SignalArgument<'static>] {
        match self {
            Self::TextChanged => to_signal::TEXT_CHANGE_ARGS.as_slice(),
            Self::Pressed => to_signal::PRESSED_ARGS.as_slice(),
            Self::ConnectionStatus => to_signal::CONNECTION_STATUS_ARGS.as_slice(),
        }
    }
    /// Construct a `Signal` representation of the `Event`. Used for constructing the class.
    pub fn signal_struct(&self) -> Signal<'static> {
        Signal {
            name: self.name(),
            args: self.args(),
        }
    }
}

/// Fire the signals and type safety.
impl Event {
    /// An example of a defaulted argument.
    pub fn emit_increment_button_disconnected_signal(owner: impl Into<Object>) {
        Self::ConnectionStatus.emit_signal_from_with_args(&mut owner.into(), &[]);
    }
    /// A basic example.
    pub fn emit_increment_button_connected_signal(owner: impl Into<Object>) {
        Self::ConnectionStatus.emit_signal_from_with_args(&mut owner.into(), &[true.to_variant()]);
    }

    /// Checks if an owner has a signal.
    fn can_be_emitted_from(&self, owner: &mut Object) -> bool {
        let signals = unsafe { owner.get_signal_list() };
        signals.contains(&self.godot_name().to_variant())
    }
    /// Call this only if you have to.
    fn emit_signal_from_with_args(&self, owner: &mut Object, varargs: &[Variant]) -> Variant {
        // This check might be done by godot? Not sure.
        if self.can_be_emitted_from(owner) {
            unsafe { owner.emit_signal(self.godot_name(), varargs) }
        } else {
            Variant::new()
        }
    }
}

// impl<'a, V: ToVariant> From<&'a [V]> for VariantArray {
//     fn from(v_buf: &'a [V]) -> Self {
//         let arr = VariantArray::new();
//         for v in v_buf {
//             arr.push(&v.to_variant());
//         }
//         arr
//     }
// }
