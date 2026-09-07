use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum ZipReaderError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Fb2Epub(Box<dyn Error>),
}

impl Error for ZipReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Zip(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ZipReaderError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
impl From<zip::result::ZipError> for ZipReaderError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::Zip(err)
    }
}

impl fmt::Display for ZipReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Io(err) => format!("ZipReader io error: {err}"),
            Self::Zip(err) => format!("Error while extracting zip: {err}"),
            Self::Fb2Epub(err) => format!("Converting error: {err}"),
        };

        write!(f, "{s}")
    }
}
