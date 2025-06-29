use jsonschema::{
    JsonType,
    error::{TypeKind, ValidationErrorKind},
};

pub trait ProblemMessage {
    /// The specific problem's message, should be in the form `this [imperative] [detail]`.
    ///
    /// Examples:
    /// * `this should be less than 5`
    /// * `this needs "someField"`
    fn message(&self) -> Option<String>;

    /// The generic problem's headline, should be in the form `is [issue]`.
    ///
    /// Examples:
    /// * `is missing a required property`
    /// * `is too large`
    fn headline(&self) -> String;
}

impl ProblemMessage for ValidationErrorKind {
    fn message(&self) -> Option<String> {
        match &self {
            Self::AdditionalItems { limit } => {
                Some(format!("this should contain at most {limit} items"))
            }
            Self::AdditionalProperties { unexpected } => Some(format!(
                "this should not have the properties [{}]",
                unexpected.join(", ")
            )),
            Self::Constant { expected_value } => Some(format!("this should be {expected_value}")),
            Self::ContentEncoding { content_encoding } => {
                Some(format!("this should be encoded as {content_encoding}"))
            }
            Self::ContentMediaType { content_media_type } => Some(format!(
                "this should be the {content_media_type} media type"
            )),
            Self::Custom { message } => Some(message.to_string()),
            Self::Enum { options } => Some(format!("this should be one of {options}")),
            Self::ExclusiveMaximum { limit } => Some(format!("this should be less than {limit}")),
            Self::ExclusiveMinimum { limit } => {
                Some(format!("this should be greater then {limit}"))
            }
            Self::Format { format } => Some(format!("this should match the {format} format")),
            Self::MaxItems { limit } => Some(format!("this should have at most {limit} items")),
            Self::Maximum { limit } => Some(format!("this should be at most {limit}")),
            Self::MaxLength { limit } => Some(format!("this should be most {limit} characters")),
            Self::MaxProperties { limit } => {
                Some(format!("this should have at most {limit} properties"))
            }
            Self::MinItems { limit } => Some(format!("this should have at least {limit} items")),
            Self::Minimum { limit } => Some(format!("this should ne at least {limit}")),
            Self::MinLength { limit } => {
                Some(format!("this should be at least {limit} characters"))
            }
            Self::MinProperties { limit } => {
                Some(format!("this should have at least {limit} properties"))
            }
            Self::MultipleOf { multiple_of } => {
                Some(format!("this should be a multiple of {multiple_of}"))
            }
            Self::Not { schema } => Some(format!("this should not be {schema}")),
            Self::Pattern { .. } => Some("this should match the format".to_string()),
            Self::Required { property } => Some(format!("this should have a {property} property")),
            Self::Type { kind } => Some(format!("this should be {}", display_type_kind(kind))),

            Self::FromUtf8 { error } => Some(format!("this could not be decoded: {error}")),
            Self::UnevaluatedItems { unexpected } => Some(format!(
                "this contains unexpected items [{}]",
                unexpected.join(", ")
            )),
            Self::UnevaluatedProperties { unexpected } => Some(format!(
                "this contains unexpected properties [{}]",
                unexpected.join(", ")
            )),
            Self::Referencing(error) => Some(format!("this could not be resolved: {error}")),
            Self::PropertyNames { error } => {
                Some(format!("this contains invalid property names: {error}"))
            }
            Self::BacktrackLimitExceeded { error } => {
                Some(format!("this exceeded the backtrack limit: {error}"))
            }

            _ => None,
        }
    }

    fn headline(&self) -> String {
        match &self {
            Self::AdditionalProperties { .. } => "has unexpected properties".to_string(),
            Self::UniqueItems => "contains duplicate items".to_string(),
            Self::OneOfMultipleValid => "matches multiple valid options".to_string(),
            Self::Required { .. } => "is missing required properties".to_string(),
            Self::OneOfNotValid
            | Self::MultipleOf { .. }
            | Self::AnyOf
            | Self::Constant { .. }
            | Self::Enum { .. }
            | Self::Not { .. } => "is not one of the valid options".to_string(),
            Self::Format { .. } | Self::Pattern { .. } => {
                "does not match the expected format".to_string()
            }
            Self::Type { .. } => "is the wrong type".to_string(),
            Self::ContentEncoding { .. } => "is not encoded correctly".to_string(),
            Self::ContentMediaType { .. } => "is not the right media type".to_string(),
            Self::Contains => "does not contain a valid item".to_string(),
            Self::Custom { .. } => "is not valid".to_string(),
            Self::ExclusiveMaximum { .. } | Self::Maximum { .. } => "is too large".to_string(),
            Self::MaxItems { .. } | Self::AdditionalItems { .. } => {
                "has too many items".to_string()
            }
            Self::MaxLength { .. } => "is too long".to_string(),
            Self::MaxProperties { .. } => "has too many properties".to_string(),
            Self::ExclusiveMinimum { .. } | Self::Minimum { .. } => "is too small".to_string(),
            Self::MinItems { .. } => "has too few items".to_string(),
            Self::MinLength { .. } => "is too short".to_string(),
            Self::MinProperties { .. } => "has too few properties".to_string(),
            Self::FromUtf8 { .. }
            | Self::FalseSchema
            | Self::Referencing(_)
            | Self::BacktrackLimitExceeded { .. }
            | Self::PropertyNames { .. }
            | Self::UnevaluatedItems { .. }
            | Self::UnevaluatedProperties { .. } => "could not be validated".to_string(),
        }
    }
}

fn display_type_kind(kind: &TypeKind) -> String {
    match kind {
        TypeKind::Single(json_type) => display_json_type(json_type).to_string(),
        TypeKind::Multiple(json_type_set) => {
            let values = json_type_set
                .iter()
                .map(|json_type| display_json_type(&json_type))
                .collect::<Vec<_>>()
                .join(", ");

            format!("one of {values}")
        }
    }
}
fn display_json_type(json_type: &JsonType) -> &'static str {
    match json_type {
        JsonType::Array => "an array",
        JsonType::Boolean => "a boolean",
        JsonType::Integer => "an integer",
        JsonType::Null => "null",
        JsonType::Number => "a number",
        JsonType::Object => "an object",
        JsonType::String => "a string",
    }
}
