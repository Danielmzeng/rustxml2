//! Error type mirroring tinyxml2's `XMLError`.

use std::fmt;

/// Result alias used across the crate.
pub type Result<T> = std::result::Result<T, XmlError>;

/// Mirrors tinyxml2's `XMLError`. `XML_SUCCESS` is represented by `Ok(_)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlError {
    NoAttribute,
    WrongAttributeType,
    FileNotFound,
    FileCouldNotBeOpened,
    FileReadError,
    ErrorParsingElement,
    ErrorParsingAttribute,
    ErrorParsingText,
    ErrorParsingCData,
    ErrorParsingComment,
    ErrorParsingDeclaration,
    ErrorParsingUnknown,
    ErrorEmptyDocument,
    ErrorMismatchedElement,
    ErrorParsing,
    CanNotConvertText,
    NoTextNode,
    ElementDepthExceeded,
}

impl XmlError {
    /// Equivalent of tinyxml2 `XMLDocument::ErrorIDToName`.
    pub fn name(self) -> &'static str {
        match self {
            XmlError::NoAttribute => "XML_NO_ATTRIBUTE",
            XmlError::WrongAttributeType => "XML_WRONG_ATTRIBUTE_TYPE",
            XmlError::FileNotFound => "XML_ERROR_FILE_NOT_FOUND",
            XmlError::FileCouldNotBeOpened => "XML_ERROR_FILE_COULD_NOT_BE_OPENED",
            XmlError::FileReadError => "XML_ERROR_FILE_READ_ERROR",
            XmlError::ErrorParsingElement => "XML_ERROR_PARSING_ELEMENT",
            XmlError::ErrorParsingAttribute => "XML_ERROR_PARSING_ATTRIBUTE",
            XmlError::ErrorParsingText => "XML_ERROR_PARSING_TEXT",
            XmlError::ErrorParsingCData => "XML_ERROR_PARSING_CDATA",
            XmlError::ErrorParsingComment => "XML_ERROR_PARSING_COMMENT",
            XmlError::ErrorParsingDeclaration => "XML_ERROR_PARSING_DECLARATION",
            XmlError::ErrorParsingUnknown => "XML_ERROR_PARSING_UNKNOWN",
            XmlError::ErrorEmptyDocument => "XML_ERROR_EMPTY_DOCUMENT",
            XmlError::ErrorMismatchedElement => "XML_ERROR_MISMATCHED_ELEMENT",
            XmlError::ErrorParsing => "XML_ERROR_PARSING",
            XmlError::CanNotConvertText => "XML_CAN_NOT_CONVERT_TEXT",
            XmlError::NoTextNode => "XML_NO_TEXT_NODE",
            XmlError::ElementDepthExceeded => "XML_ELEMENT_DEPTH_EXCEEDED",
        }
    }
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::error::Error for XmlError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_names_match_tinyxml2() {
        assert_eq!(XmlError::NoAttribute.name(), "XML_NO_ATTRIBUTE");
        assert_eq!(
            XmlError::ElementDepthExceeded.name(),
            "XML_ELEMENT_DEPTH_EXCEEDED"
        );
        assert_eq!(
            XmlError::ErrorMismatchedElement.name(),
            "XML_ERROR_MISMATCHED_ELEMENT"
        );
    }
}
