/// StatusCode describes various failure modes of database operations.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum StatusCode {
    OK,

    AlreadyExists,
    Corruption,
    CompressionError,
    IOError,
    InvalidArgument,
    InvalidData,
    LockError,
    NotFound,
    NotSupported,
    PermissionDenied,
    AsyncError,
    Unknown,
    #[cfg(feature = "fs")]
    Errno(errno::Errno),
}

/// Status encapsulates a `StatusCode` and an error message. It can be displayed, and also
/// implements `Error`.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub code: StatusCode,
    pub err: String,
}

impl Default for Status {
    fn default() -> Self {
        Status {
            code: StatusCode::OK,
            err: String::new(),
        }
    }
}

impl Display for Status {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_str(&self.err)
    }
}

impl Error for Status {
    fn description(&self) -> &str {
        &self.err
    }
}

impl Status {
    pub fn new(code: StatusCode, err: String) -> Self {
        let err = if err.is_empty() {
            format!("{:?}", code)
        } else {
            format!("{:?}: {}", code, err)
        };
        Status {code, err}
    }
}

pub type Result<T> = result::Result<T, Status>;

pub fn err<T>(code: StatusCode, err: String) -> Result<T> {
    Err(Status::new(code, err))
}

impl From<io::Error> for Status {
    fn from(err: io::Error) -> Self {
        let c = match e.kind() {
            io::ErrorKind::NotFound => StatusCode::NotFound,
            io::ErrorKind::InvalidData => StatusCode::Corruption,
            io::ErrorKind::InvalidInput => StatusCode::InvalidArgument,
            io::ErrorKind::PermissionDenied => StatusCode::PermissionDenied,
            _ => StatusCode::IOError,
        };

        Status::new(c, &e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{Status, StatusCode};
    #[test]
    fn test_status_to_string() {
        let s = Status::new(StatusCode::InvalidData, "Invalid data!");
        assert_eq!("InvalidData: Invalid data!", s.to_string());
    }
}