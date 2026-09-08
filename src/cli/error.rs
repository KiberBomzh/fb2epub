use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum CliError {
    ParseArgs(lexopt::Error),
    Converting(fb2epub::Error),
    EmptyInput,
    GetInput(std::io::Error),
    SetOutput(std::io::Error),
    GetOutput(std::io::Error),
    Other(String),

    #[cfg(feature = "zip")]
    ZipReader(super::zip_reader::Error),
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParseArgs(e) => Some(e),
            CliError::Converting(e) => Some(e),
            Self::GetInput(e) => Some(e),
            Self::SetOutput(e) => Some(e),
            Self::GetOutput(e) => Some(e),

            #[cfg(feature = "zip")]
            Self::ZipReader(e) => Some(e),
            _ => None,
        }
    }
}

impl From<fb2epub::Error> for CliError {
    fn from(err: fb2epub::Error) -> Self {
        Self::Converting(err)
    }
}
impl From<lexopt::Error> for CliError {
    fn from(err: lexopt::Error) -> Self {
        Self::ParseArgs(err)
    }
}
impl From<String> for CliError {
    fn from(err: String) -> Self {
        Self::Other(err)
    }
}
#[cfg(feature = "zip")]
impl From<super::zip_reader::Error> for CliError {
    fn from(err: super::zip_reader::Error) -> Self {
        Self::ZipReader(err)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::ParseArgs(err) => format!("Error while parsing cli args: {err}"),
            Self::Converting(err) => format!("Converting error: {err}"),
            Self::EmptyInput => String::from("There's no fb2 books in the input!"),
            Self::GetInput(err) => format!("Error while getting inputs: {err}"),
            Self::SetOutput(err) => format!("Error while setting output: {err}"),
            Self::GetOutput(err) => format!("Error while getting output: {err}"),
            Self::Other(err) => err.clone(),

            #[cfg(feature = "zip")]
            Self::ZipReader(err) => err.to_string(),
        };

        write!(f, "{s}")
    }
}
