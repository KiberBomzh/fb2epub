use std::path::PathBuf;


const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const HELP_MSG: &str = r#"Usage: fb2epub [OPTIONS] [INPUTS]...

Arguments:
  [INPUTS]...  Input files/dirs

Options:
  -o, --output <OUTPUT>              Output path. Directory. If there's only one input also can be a file
      --styles <STYLES>              Custom css styles for a book. Path to a .css file
  -r, --recursive                    Include all books from subdirs of given in inputs directory/directories
  -p, --pipe                         Read input (only fb2) from stdin, write epub in stdout
      --print                        Print output paths
      --debug                        Use debug mod
      --title <TITLE>                Use given title for input book(s)
      --author <AUTHOR>...           Use given author(s) for input book(s)
      --language <LANGUAGE>          Use given language for input book(s)
      --series <SERIES>              Use given series for input book(s)
      --series-index <SERIES_INDEX>  Use given series index for input book(s)
  -h, --help                         Print help
  -V, --version                      Print version"#;


pub struct Args {
    pub inputs: Vec<PathBuf>,
    pub output: Option<String>,
    pub styles: Option<PathBuf>,

    pub recursive: bool,
    pub pipe: bool,
    pub print: bool,
    pub debug: bool,

    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub language: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<String>
}

impl Args {
    #[allow(clippy::bool_comparison)]
    pub fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;

        let mut inputs = Vec::new();
        let mut output = None;
        let mut styles = None;

        let mut recursive = false;
        let mut pipe = false;
        let mut print = false;
        let mut debug = false;

        let mut title = None;
        let mut authors_some = Vec::new();
        let mut language = None;
        let mut series = None;
        let mut series_index = None;

        let mut parser = lexopt::Parser::from_env();
        while let Some(arg) = parser.next()? {
            match arg {
                Value(v) => {
                    let p = PathBuf::from(v.string()?);
                    inputs.push(p);
                },
                Short('o') | Long("output") if output.is_none() => {
                    let v = parser.value()?.string()?;
                    output = Some(v);
                },
                Short('s') | Long("styles") if styles.is_none() => {
                    let v = parser.value()?.string()?;
                    let p = PathBuf::from(v);
                    styles = Some(p);
                },

                Short('r') | Long("recursive") if recursive == false =>
                    recursive = true,
                Short('p') | Long("pipe") if pipe == false =>
                    pipe = true,
                Long("print") if print == false =>
                    print = true,
                Long("debug") if debug == false =>
                    debug = true,

                Long("title") if title.is_none() => {
                    let v = parser.value()?.string()?;
                    title = Some(v);
                },
                Long("author") => {
                    let v = parser.value()?.string()?;
                    authors_some.push(v);
                },
                Long("language") if language.is_none() => {
                    let v = parser.value()?.string()?;
                    language = Some(v);
                },
                Long("series") if series.is_none() => {
                    let v = parser.value()?.string()?;
                    series = Some(v);
                },
                Long("series-index") if series_index.is_none() => {
                    let v = parser.value()?.string()?;
                    series_index = Some(v);
                },

                Short('h') | Long("help") => {
                    println!("{DESCRIPTION}\n\n{HELP_MSG}");
                    std::process::exit(0);
                },
                Short('V') | Long("version") => {
                    println!("{NAME} {VERSION}");
                    std::process::exit(0);
                },

                _ => return Err(arg.unexpected()),
            }
        }

        let authors = 
            if authors_some.is_empty() { None }
            else { Some(authors_some) };


        Ok(
            Self{inputs, output, styles, recursive, pipe, print, debug,
                title, authors, language, series, series_index
            }
        )
    }
}
