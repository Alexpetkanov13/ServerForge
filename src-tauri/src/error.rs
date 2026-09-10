use serde::Serialize;
use std::fmt;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone)]
pub struct AppError {
    pub title: String,
    pub message: String,
    pub causes: Vec<String>,
    pub technical: String,
}

impl AppError {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            causes: Vec::new(),
            technical: String::new(),
        }
    }

    pub fn with_causes(mut self, causes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.causes = causes.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_technical(mut self, technical: impl Into<String>) -> Self {
        self.technical = technical.into();
        self
    }

    pub fn message(msg: impl Into<String>) -> Self {
        let message = msg.into();
        Self::new("Something went wrong", message.clone()).with_technical(message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 4)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("causes", &self.causes)?;
        state.serialize_field("technical", &self.technical)?;
        state.end()
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::new("File system error", "A file or folder operation failed.")
            .with_causes(vec![
                "The path may not exist",
                "ServerForge may not have permission to write there",
                "The disk may be full",
            ])
            .with_technical(value.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::new("Database error", "ServerForge could not read or write application metadata.")
            .with_technical(value.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::new("Database error", "ServerForge could not prepare its metadata database.")
            .with_technical(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::new(
            "Network error",
            "Unable to reach a remote download or metadata service.",
        )
        .with_causes(vec![
            "Internet connection unavailable",
            "Provider API unavailable",
            "The selected version may no longer exist",
        ])
        .with_technical(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::new("Data error", "A response could not be parsed.")
            .with_technical(value.to_string())
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::new("Archive error", "A backup archive could not be processed.")
            .with_technical(value.to_string())
    }
}
