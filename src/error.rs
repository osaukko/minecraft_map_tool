pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    pub fn invalid_data(msg: &str) -> Error {
        Error(Box::new(ErrorKind::InvalidData(String::from(msg))))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self.0 {
            ErrorKind::ImageError(ref err) => err.fmt(f),
            ErrorKind::InvalidData(ref err) => err.fmt(f),
            ErrorKind::IoError(ref err) => err.fmt(f),
            ErrorKind::Message(ref err) => err.fmt(f),
            ErrorKind::NbtError(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::IoError(err))
    }
}

impl From<valence_nbt::Error> for Error {
    fn from(err: valence_nbt::Error) -> Self {
        Error::new(ErrorKind::NbtError(err))
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::new(ErrorKind::ImageError(err))
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error::new(ErrorKind::Message(message))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    ImageError(image::ImageError),
    InvalidData(String),
    IoError(std::io::Error),
    Message(String),
    NbtError(valence_nbt::Error),
}
