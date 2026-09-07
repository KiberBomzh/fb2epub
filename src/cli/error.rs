use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum CliError {
    ParseArgs(lexopt::Error),
    Converting(Box<dyn Error>),
    EmptyInput,
    OutputSetting(std::io::Error),
    InputsOutputsLen,
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParseArgs(e) => Some(e),
            // CliError::Converting(e) => Some(e),
            Self::OutputSetting(e) => Some(e),
            _ => None,
        }
    }
}

impl From<lexopt::Error> for CliError {
    fn from(err: lexopt::Error) -> Self {
        Self::ParseArgs(err)
    }
}
impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::OutputSetting(err)
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::ParseArgs(err) => format!("Error while parsing cli args: {err}"),
            Self::Converting(err) => format!("Converting error: {err}"),
            Self::EmptyInput => String::from("There's no fb2 books in the input!"),
            Self::OutputSetting(err) => format!("Error while setting output path: {err}"),
            Self::InputsOutputsLen => String::from("inputs.len() and outputs.len() doesn't match!"),
        };

        write!(f, "{s}")
    }
}
