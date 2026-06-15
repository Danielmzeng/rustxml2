//! XML attribute storage, typed accessors, and the `XmlValue` set-conversion trait.

use crate::error::{Result, XmlError};
use crate::strutil::parse_bool;

/// A single `name="value"` attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

macro_rules! typed_accessor {
    ($method:ident, $ty:ty) => {
        pub fn $method(&self) -> Result<$ty> {
            self.value.trim().parse::<$ty>().map_err(|_| XmlError::WrongAttributeType)
        }
    };
}

impl Attribute {
    pub fn value(&self) -> &str {
        &self.value
    }

    typed_accessor!(as_i32, i32);
    typed_accessor!(as_i64, i64);
    typed_accessor!(as_u32, u32);
    typed_accessor!(as_u64, u64);
    typed_accessor!(as_f32, f32);
    typed_accessor!(as_f64, f64);

    pub fn as_bool(&self) -> Result<bool> {
        parse_bool(&self.value).ok_or(XmlError::WrongAttributeType)
    }
}

/// Types that can be stored as an attribute or text value (port of overloaded
/// `SetAttribute` / `SetText` / `PushText`).
pub trait XmlValue {
    fn to_xml_string(&self) -> String;
}

macro_rules! xml_value_display {
    ($ty:ty) => {
        impl XmlValue for $ty {
            fn to_xml_string(&self) -> String {
                self.to_string()
            }
        }
    };
}

xml_value_display!(i32);
xml_value_display!(i64);
xml_value_display!(u32);
xml_value_display!(u64);
xml_value_display!(f32);
xml_value_display!(f64);

impl XmlValue for bool {
    fn to_xml_string(&self) -> String {
        if *self { "true".into() } else { "false".into() }
    }
}

impl XmlValue for &str {
    fn to_xml_string(&self) -> String {
        (*self).to_string()
    }
}

impl XmlValue for String {
    fn to_xml_string(&self) -> String {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_accessors() {
        let a = Attribute { name: "x".into(), value: "42".into() };
        assert_eq!(a.as_i32(), Ok(42));
        assert_eq!(a.as_i64(), Ok(42));
        assert_eq!(a.as_f64(), Ok(42.0));

        let b = Attribute { name: "b".into(), value: "true".into() };
        assert_eq!(b.as_bool(), Ok(true));

        let bad = Attribute { name: "n".into(), value: "abc".into() };
        assert_eq!(bad.as_i32(), Err(XmlError::WrongAttributeType));
    }

    #[test]
    fn xml_value_to_string() {
        assert_eq!(42i32.to_xml_string(), "42");
        assert_eq!(true.to_xml_string(), "true");
        assert_eq!("hi".to_xml_string(), "hi");
    }
}
