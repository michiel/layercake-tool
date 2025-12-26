use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreErrorKind {
    NotFound,
    Validation,
    Conflict,
    Forbidden,
    Unauthorized,
    Unavailable,
    Internal,
}

#[derive(Debug)]
pub struct CoreError {
    kind: CoreErrorKind,
    message: String,
    fields: Option<BTreeMap<String, String>>,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl CoreError {
    pub fn new(kind: CoreErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            fields: None,
            source: None,
        }
    }

    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        let mut fields = BTreeMap::new();
        fields.insert("entity".to_string(), entity.into());
        fields.insert("id".to_string(), id.into());

        Self {
            kind: CoreErrorKind::NotFound,
            message: "Resource not found".to_string(),
            fields: Some(fields),
            source: None,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(CoreErrorKind::Validation, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(CoreErrorKind::Conflict, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(CoreErrorKind::Forbidden, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(CoreErrorKind::Unauthorized, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(CoreErrorKind::Unavailable, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(CoreErrorKind::Internal, message)
    }

    pub fn with_fields(mut self, fields: BTreeMap<String, String>) -> Self {
        self.fields = Some(fields);
        self
    }

    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    pub fn kind(&self) -> CoreErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn fields(&self) -> Option<&BTreeMap<String, String>> {
        self.fields.as_ref()
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", format!("{:?}", self.kind), self.message)
    }
}

impl StdError for CoreError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn StdError + 'static))
    }
}

impl From<anyhow::Error> for CoreError {
    fn from(err: anyhow::Error) -> Self {
        CoreError::internal("Unhandled error").with_source(err)
    }
}
