use std::fmt::Display;

// TODO: move into different crate
pub mod channel;
mod xml_helpers;

const YOUTUBE_BASE_URL: &str = "https://www.youtube.com";

type InnerError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum YouTubeError {
    ConnectionError,
    ParserError(String),
    SyntaxError(InnerError),
}

impl Display for YouTubeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            YouTubeError::ConnectionError => write!(f, "failed to connect to youtube"),
            YouTubeError::SyntaxError(reason) => write!(f, "malformed RSS: {reason}"),
            YouTubeError::ParserError(reason) => write!(f, "failed to parse: {reason}"),
        }
    }
}
impl std::error::Error for YouTubeError {}

type YouTubeResult<T> = Result<T, YouTubeError>;
