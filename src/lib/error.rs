use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum Fb2EpubError {
    Fb2Parser(String),
    EpubCreator(epub_builder::Error),
}

impl Error for Fb2EpubError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Fb2Parser(_) => None,
            Self::EpubCreator(e) => Some(e),
        }
    }
}

impl fmt::Display for Fb2EpubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Fb2Parser(e) => format!("FB2 parser error: {e}"),
            Self::EpubCreator(e) => format!("Error while creating epub: {e}"),
        };

        write!(f, "{s}")
    }
}
