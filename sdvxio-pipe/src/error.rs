#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    WrongResponseId { expected: u64, got: u64 },
    WrongResponseType,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IO error: {}", err),
            Error::WrongResponseId { expected, got } => {
                write!(f, "Wrong response ID: expected {}, got {}", expected, got)
            }
            Error::WrongResponseType => write!(f, "Wrong response type"),
        }
    }
}
