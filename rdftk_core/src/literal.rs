/*!
The `Literal` type used in the object component of a statement.

# Example

TBD

*/

use crate::QName;
use rdftk_iri::IRIRef;
use rdftk_names::xsd;
use std::fmt::{Display, Formatter};
use std::time::Duration;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
///
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Denotes a literal of type `xsd::string`.
    String,
    /// Denotes a literal of type `xsd::qname`.
    QName,
    /// Denotes a literal of type `xsd::anyURI`.
    #[allow(clippy::upper_case_acronyms)]
    IRI,
    /// Denotes a literal of type `xsd::boolean`.
    Boolean,
    /// Denotes a literal of type `xsd::float`.
    Float,
    /// Denotes a literal of type `xsd::double`.
    Double,
    /// Denotes a literal of type `xsd::long`.
    Long,
    /// Denotes a literal of type `xsd::int`.
    Int,
    /// Denotes a literal of type `xsd::short`.
    Short,
    /// Denotes a literal of type `xsd::byte`.
    Byte,
    /// Denotes a literal of type `xsd::unsignedLong`.
    UnsignedLong,
    /// Denotes a literal of type `xsd::unsignedInt`.
    UnsignedInt,
    /// Denotes a literal of type `xsd::unsignedShort`.
    UnsignedShort,
    /// Denotes a literal of type `xsd::unsignedByte`.
    UnsignedByte,
    /// Denotes a literal of type `xsd::duration`.
    Duration,
    /// Denotes a literal where the type is indicated by the provided `IRI`.
    Other(IRIRef),
}

///
///
///
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Literal {
    lexical_form: String,
    data_type: Option<DataType>,
    language: Option<String>,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_iri())
    }
}

impl From<IRIRef> for DataType {
    fn from(v: IRIRef) -> Self {
        DataType::Other(v)
    }
}

impl From<DataType> for IRIRef {
    fn from(v: DataType) -> Self {
        v.as_iri().clone()
    }
}

impl DataType {
    pub fn as_iri(&self) -> &IRIRef {
        match &self {
            DataType::String => xsd::string(),
            DataType::QName => xsd::q_name(),
            DataType::IRI => xsd::any_uri(),
            DataType::Boolean => xsd::boolean(),
            DataType::Float => xsd::float(),
            DataType::Double => xsd::double(),
            DataType::Long => xsd::long(),
            DataType::Int => xsd::int(),
            DataType::Short => xsd::short(),
            DataType::Byte => xsd::byte(),
            DataType::UnsignedLong => xsd::unsigned_long(),
            DataType::UnsignedInt => xsd::unsigned_int(),
            DataType::UnsignedShort => xsd::unsigned_short(),
            DataType::UnsignedByte => xsd::unsigned_byte(),
            DataType::Duration => xsd::duration(),
            DataType::Other(iri) => iri,
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{}\"{}",
            self.lexical_form(),
            match (self.data_type(), self.language()) {
                (Some(data_type), None) => format!(
                    "^^<{}>",
                    match data_type {
                        DataType::String => xsd::string(),
                        DataType::QName => xsd::q_name(),
                        DataType::IRI => xsd::any_uri(),
                        DataType::Boolean => xsd::boolean(),
                        DataType::Float => xsd::float(),
                        DataType::Double => xsd::double(),
                        DataType::Long => xsd::long(),
                        DataType::Int => xsd::int(),
                        DataType::Short => xsd::short(),
                        DataType::Byte => xsd::byte(),
                        DataType::UnsignedLong => xsd::unsigned_long(),
                        DataType::UnsignedInt => xsd::unsigned_int(),
                        DataType::UnsignedShort => xsd::unsigned_short(),
                        DataType::UnsignedByte => xsd::unsigned_byte(),
                        DataType::Duration => xsd::duration(),
                        DataType::Other(iri) => iri,
                    }
                ),
                (None, Some(language)) => format!("@{}", language.to_lowercase()),
                _ => String::new(),
            }
        )
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Self {
            lexical_form: Self::escape_string(&value),
            data_type: Some(DataType::String),
            language: None,
        }
    }
}

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Self {
            lexical_form: Self::escape_string(value),
            data_type: Some(DataType::String),
            language: None,
        }
    }
}

impl From<IRIRef> for Literal {
    fn from(value: IRIRef) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::IRI),
            language: None,
        }
    }
}

impl From<QName> for Literal {
    fn from(value: QName) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::QName),
            language: None,
        }
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Boolean),
            language: None,
        }
    }
}

impl From<f32> for Literal {
    fn from(value: f32) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Float),
            language: None,
        }
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Double),
            language: None,
        }
    }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Long),
            language: None,
        }
    }
}

impl From<i32> for Literal {
    fn from(value: i32) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Int),
            language: None,
        }
    }
}

impl From<i16> for Literal {
    fn from(value: i16) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Short),
            language: None,
        }
    }
}

impl From<i8> for Literal {
    fn from(value: i8) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::Byte),
            language: None,
        }
    }
}

impl From<u64> for Literal {
    fn from(value: u64) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::UnsignedLong),
            language: None,
        }
    }
}

impl From<u32> for Literal {
    fn from(value: u32) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::UnsignedInt),
            language: None,
        }
    }
}

impl From<u16> for Literal {
    fn from(value: u16) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::UnsignedShort),
            language: None,
        }
    }
}

impl From<u8> for Literal {
    fn from(value: u8) -> Self {
        Self {
            lexical_form: value.to_string(),
            data_type: Some(DataType::UnsignedByte),
            language: None,
        }
    }
}

impl From<Duration> for Literal {
    fn from(value: Duration) -> Self {
        Self {
            lexical_form: format!("T{}.{:09}S", value.as_secs(), value.subsec_nanos()),
            data_type: Some(DataType::Duration),
            language: None,
        }
    }
}

impl Literal {
    pub fn new(value: &str) -> Self {
        Self {
            lexical_form: Self::escape_string(value),
            data_type: None,
            language: None,
        }
    }

    pub fn with_type(value: &str, data_type: DataType) -> Self {
        Self {
            lexical_form: Self::escape_string(value),
            data_type: Some(data_type),
            language: None,
        }
    }

    pub fn with_language(value: &str, language: &str) -> Self {
        Self {
            lexical_form: Self::escape_string(value),
            data_type: None,
            language: Some(language.to_string()),
        }
    }

    pub fn lexical_form(&self) -> &String {
        &self.lexical_form
    }

    pub fn has_data_type(&self) -> bool {
        self.data_type.is_some()
    }

    pub fn data_type(&self) -> &Option<DataType> {
        &self.data_type
    }

    pub fn has_language(&self) -> bool {
        self.language.is_some()
    }

    pub fn language(&self) -> &Option<String> {
        &self.language
    }

    fn escape_string(value: &str) -> String {
        let formatted = format!("{:?}", value);
        formatted[1..formatted.len() - 1].to_string()
    }
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_untyped() {
        let value = Literal::new("a string");
        assert!(!value.has_data_type());
        assert!(!value.has_language());
        assert_eq!(value.lexical_form(), "a string");
        assert_eq!(value.to_string(), "\"a string\"");
    }

    #[test]
    fn test_needs_escape() {
        let value = Literal::new(r#"\ta "string"#);
        assert!(!value.has_data_type());
        assert!(!value.has_language());
        assert_eq!(value.lexical_form(), "\\\\ta \\\"string");
        assert_eq!(value.to_string(), "\"\\\\ta \\\"string\"");
    }

    #[test]
    fn test_language_string() {
        let value = Literal::with_language("a string", "en_us");
        assert!(!value.has_data_type());
        assert!(value.has_language());
        assert_eq!(value.lexical_form(), "a string");
        assert_eq!(value.to_string(), "\"a string\"@en_us");
    }

    #[test]
    fn test_typed_as_string() {
        let value: Literal = "a string".to_string().into();
        assert!(value.has_data_type());
        assert!(!value.has_language());
        assert_eq!(value.lexical_form(), "a string");
        assert_eq!(
            value.to_string(),
            "\"a string\"^^<http://www.w3.org/2001/XMLSchema#string>"
        );
    }

    #[test]
    fn test_typed_as_boolean() {
        let value: Literal = true.into();
        assert!(value.has_data_type());
        assert!(!value.has_language());
        assert_eq!(value.lexical_form(), "true");
        assert_eq!(
            value.to_string(),
            "\"true\"^^<http://www.w3.org/2001/XMLSchema#boolean>"
        );
    }

    #[test]
    fn test_typed_as_long() {
        let value: Literal = 1u64.into();
        assert!(value.has_data_type());
        assert!(!value.has_language());
        assert_eq!(value.lexical_form(), "1");
        assert_eq!(
            value.to_string(),
            "\"1\"^^<http://www.w3.org/2001/XMLSchema#unsignedLong>"
        );
    }

    #[test]
    fn test_typed_as_duration() {
        let start = Instant::now();
        std::thread::sleep(Duration::from_secs(2));
        let duration = start.elapsed();
        println!("Duration  In: {:#?}", duration);

        let value: Literal = duration.into();
        println!("Duration Out: {}", value);
        assert!(value.has_data_type());
        assert!(!value.has_language());

        let value_str = value.to_string();
        assert!(value_str.starts_with("\"T2."));
        assert!(value_str.ends_with("S\"^^<http://www.w3.org/2001/XMLSchema#duration>"));
    }
}
