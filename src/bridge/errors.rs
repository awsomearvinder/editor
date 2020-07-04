#[derive(Debug, PartialEq)]
pub enum BridgeErrors {
    ConversionError,
}
impl std::fmt::Display for BridgeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConversionError => write!(f, "Got conversion error"),
        }
    }
}
impl std::error::Error for BridgeErrors {}
