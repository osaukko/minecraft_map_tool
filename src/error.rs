pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self.0 {
            ErrorKind::ImageError(ref err) => err.fmt(f),
            ErrorKind::IoError(ref err) => err.fmt(f),
            ErrorKind::FastNbtError(ref err) => err.fmt(f),
        }
    }
}

impl From<fastnbt::error::Error> for Error {
    fn from(err: fastnbt::error::Error) -> Self {
        Error::new(ErrorKind::FastNbtError(err))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::new(ErrorKind::IoError(err))
    }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::new(ErrorKind::ImageError(err))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    FastNbtError(fastnbt::error::Error),
    ImageError(image::ImageError),
    IoError(std::io::Error),
}
