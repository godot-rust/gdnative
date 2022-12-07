use super::*;
use serde::{
    de::{EnumAccess, Error, SeqAccess, VariantAccess, Visitor},
    ser::{Error as _, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt::Formatter;

/// Custom implementation to allow using the same visitor for VariantType as well as the discriminant
/// of VariantDispatch.
struct VariantTypeVisitor;

impl<'de> Visitor<'de> for VariantTypeVisitor {
    type Value = VariantType;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a VariantType")
    }

    #[inline]
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if value < VariantType::NAMES.len() as u64 {
            Ok(VariantType::from_sys(value as sys::godot_variant_type))
        } else {
            Err(E::custom(&*format!("invalid VariantType value: {value}")))
        }
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        for (i, &name) in VariantType::NAMES.iter().enumerate() {
            if name == value {
                return Ok(VariantType::from_sys(i as sys::godot_variant_type));
            }
        }
        Err(E::custom(&*format!("invalid VariantType value: {value:?}")))
    }

    #[inline]
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        for (i, &name) in VariantType::NAMES.iter().enumerate() {
            if name.as_bytes() == value {
                return Ok(VariantType::from_sys(i as sys::godot_variant_type));
            }
        }
        Err(E::custom(&*format!("invalid VariantType value: {value:?}")))
    }

    #[inline]
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let (t, v) = data.variant::<VariantType>()?;
        v.unit_variant()?;
        Ok(t)
    }
}

impl<'de> Deserialize<'de> for VariantType {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // NOTE: serde assumes that serialized indices correspond to the indices in the NAMES array.
        // If any non-sequential VariantType values were added in the future, this could break, but
        // that seems extremely unlikely, and would require a breaking change to godot-rust anyway
        // since VariantType is not marked as non-exhaustive.
        deserializer.deserialize_enum("VariantType", VariantType::NAMES, VariantTypeVisitor)
    }
}

/// Enables calling `deserialize_identifier` instead of `deserialize_enum` when deserializing VariantDispatch.
struct VariantDispatchDiscriminant(VariantType);

impl<'de> Deserialize<'de> for VariantDispatchDiscriminant {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_identifier(VariantTypeVisitor)
            .map(Self)
    }
}

/// This allows (de)serializing to/from non-self-describing formats by avoiding serializing `Variant`s
// Can't just use a HashMap because VariantDispatch doesn't implement Hash, and this avoids cloning all of the entries anyway
struct DictionaryDispatch(Dictionary);

#[derive(Serialize, Deserialize)]
struct DictionaryDispatchEntry {
    key: VariantDispatch,
    value: VariantDispatch,
}

impl Serialize for DictionaryDispatch {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut ser = serializer.serialize_seq(Some(self.0.len() as usize))?;
        for (key, value) in self.0.iter() {
            ser.serialize_element(&DictionaryDispatchEntry {
                key: key.dispatch(),
                value: value.dispatch(),
            })?;
        }
        ser.end()
    }
}

impl<'de> Deserialize<'de> for DictionaryDispatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct DictionaryDispatchVisitor;
        impl<'de> Visitor<'de> for DictionaryDispatchVisitor {
            type Value = DictionaryDispatch;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of VariantDispatch pairs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let dict = Dictionary::new();
                while let Some(DictionaryDispatchEntry { key, value }) = seq.next_element()? {
                    dict.insert(Variant::from(&key), Variant::from(&value))
                }
                Ok(DictionaryDispatch(dict.into_shared()))
            }
        }
        deserializer.deserialize_seq(DictionaryDispatchVisitor)
    }
}

impl Serialize for VariantDispatch {
    #[inline]
    fn serialize<S>(&self, ser: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        use VariantDispatch::*;

        const NAME: &str = "VariantDispatch";

        macro_rules! newtype_variant {
            ($t:expr, $v:expr) => {
                ser.serialize_newtype_variant(NAME, $t as u32, $t.name(), $v)
            };
        }
        match self {
            Nil => {
                ser.serialize_unit_variant(NAME, VariantType::Nil as u32, VariantType::Nil.name())
            }
            Bool(v) => newtype_variant!(VariantType::Bool, v),
            I64(v) => newtype_variant!(VariantType::I64, v),
            F64(v) => newtype_variant!(VariantType::F64, v),
            GodotString(v) => newtype_variant!(VariantType::GodotString, v),
            Vector2(v) => newtype_variant!(VariantType::Vector2, v),
            Rect2(v) => newtype_variant!(VariantType::Rect2, v),
            Vector3(v) => newtype_variant!(VariantType::Vector3, v),
            Transform2D(v) => newtype_variant!(VariantType::Transform2D, v),
            Plane(v) => newtype_variant!(VariantType::Plane, v),
            Quat(v) => newtype_variant!(VariantType::Quat, v),
            Aabb(v) => newtype_variant!(VariantType::Aabb, v),
            Basis(v) => newtype_variant!(VariantType::Basis, v),
            Transform(v) => newtype_variant!(VariantType::Transform, v),
            Color(v) => newtype_variant!(VariantType::Color, v),
            NodePath(v) => newtype_variant!(VariantType::NodePath, v),
            Rid(_) => Err(S::Error::custom("Serialization of RID's is not supported")),
            Object(_) => Err(S::Error::custom(
                "Serialization of Objects is not supported",
            )),
            Dictionary(v) => {
                newtype_variant!(VariantType::Dictionary, &DictionaryDispatch(v.new_ref()))
            }
            VariantArray(v) => {
                // Allows serializing to non-self-describing formats by avoiding serializing `Variant`s
                let vec = v.iter().map(|v| v.dispatch()).collect::<Vec<_>>();
                newtype_variant!(VariantType::VariantArray, &vec)
            }
            ByteArray(v) => newtype_variant!(VariantType::ByteArray, v),
            Int32Array(v) => newtype_variant!(VariantType::Int32Array, v),
            Float32Array(v) => newtype_variant!(VariantType::Float32Array, v),
            StringArray(v) => newtype_variant!(VariantType::StringArray, v),
            Vector2Array(v) => newtype_variant!(VariantType::Vector2Array, v),
            Vector3Array(v) => newtype_variant!(VariantType::Vector3Array, v),
            ColorArray(v) => newtype_variant!(VariantType::ColorArray, v),
        }
    }
}

struct VariantDispatchVisitor;

impl<'de> Visitor<'de> for VariantDispatchVisitor {
    type Value = VariantDispatch;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("enum VariantDispatch")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        use VariantType::*;
        let (t, v) = data.variant::<VariantDispatchDiscriminant>()?;
        Ok(match t.0 {
            Nil => {
                v.unit_variant()?;
                VariantDispatch::Nil
            }
            Bool => VariantDispatch::Bool(v.newtype_variant()?),
            I64 => VariantDispatch::I64(v.newtype_variant()?),
            F64 => VariantDispatch::F64(v.newtype_variant()?),
            GodotString => VariantDispatch::GodotString(v.newtype_variant()?),
            Vector2 => VariantDispatch::Vector2(v.newtype_variant()?),
            Rect2 => VariantDispatch::Rect2(v.newtype_variant()?),
            Vector3 => VariantDispatch::Vector3(v.newtype_variant()?),
            Transform2D => VariantDispatch::Transform2D(v.newtype_variant()?),
            Plane => VariantDispatch::Plane(v.newtype_variant()?),
            Quat => VariantDispatch::Quat(v.newtype_variant()?),
            Aabb => VariantDispatch::Aabb(v.newtype_variant()?),
            Basis => VariantDispatch::Basis(v.newtype_variant()?),
            Transform => VariantDispatch::Transform(v.newtype_variant()?),
            Color => VariantDispatch::Color(v.newtype_variant()?),
            NodePath => VariantDispatch::NodePath(v.newtype_variant()?),
            Rid => return Err(A::Error::custom("Not sure how an RID got serialized")),
            Object => return Err(A::Error::custom("Not sure how an Object got serialized")),
            Dictionary => VariantDispatch::Dictionary(v.newtype_variant::<DictionaryDispatch>()?.0),
            VariantArray => VariantDispatch::VariantArray(
                v.newtype_variant::<Vec<VariantDispatch>>()?
                    .iter()
                    .map(Into::<Variant>::into)
                    .collect::<variant_array::VariantArray<Unique>>()
                    .into_shared(),
            ),
            ByteArray => VariantDispatch::ByteArray(v.newtype_variant()?),
            Int32Array => VariantDispatch::Int32Array(v.newtype_variant()?),
            Float32Array => VariantDispatch::Float32Array(v.newtype_variant()?),
            StringArray => VariantDispatch::StringArray(v.newtype_variant()?),
            Vector2Array => VariantDispatch::Vector2Array(v.newtype_variant()?),
            Vector3Array => VariantDispatch::Vector3Array(v.newtype_variant()?),
            ColorArray => VariantDispatch::ColorArray(v.newtype_variant()?),
        })
    }
}

impl<'de> Deserialize<'de> for VariantDispatch {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_enum(
            "VariantDispatch",
            VariantType::NAMES,
            VariantDispatchVisitor,
        )
    }
}
