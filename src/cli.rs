#[cfg(feature = "zip")]
mod zip_reader;

mod args;
mod error;


use std::path::{PathBuf, Path, MAIN_SEPARATOR};
use std::fs;

use threadpool::ThreadPool;

#[cfg(not(target_os = "windows"))]
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

use error::CliError;


#[cfg(feature = "zip")]
const ALLOWED_EXTENSIONS: [&str; 2] = [
    "fb2",
    "zip"
];
#[cfg(not(feature = "zip"))]
const ALLOWED_EXTENSIONS: [&str; 1] = [
    "fb2",
];


pub fn handle_cli() -> Result<(), CliError> {
    let args = args::Args::parse()?;

    if args.pipe {
        return handle_pipe(args)
            .map_err(CliError::Converting);
    }


    let mut inputs = get_inputs(args.inputs, args.recursive)
        .map_err(CliError::GetInput)?;
    if inputs.is_empty() {
        return Err(CliError::EmptyInput);
    };

    if let Some(o) = &args.output {
        set_output(o, inputs.len())
            .map_err(CliError::SetOutput)?;
    }

    let mut outputs: Vec<PathBuf> = if inputs.len() > 1 {
        get_outputs(&inputs, args.output)
    } else {
        get_output(&inputs, args.output)
    }.map_err(CliError::GetOutput)?;

    if inputs.len() != outputs.len() {
        panic!("inputs.len() and outputs.len() doesn't match!");
    }
    let mut files: Vec<(PathBuf, PathBuf)> = Vec::with_capacity(inputs.len());
    while let Some(i) = inputs.pop() && let Some(o) = outputs.pop() {
        files.push( (i, o) );
    }

    
    let styles_path: Option<PathBuf> = args.styles.filter(|p| p.is_file());

    let metadata = parse_meta_from_args(
        args.title,
        args.authors,
        args.language,
        args.series,
        args.series_index,
    );


    handle_converting(
        files,
        styles_path,
        metadata,
        args.print,
        args.debug,
    )?;


    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn handle_converting(
    mut files: Vec<(PathBuf, PathBuf)>,
    styles: Option<PathBuf>,
    metadata: Option<fb2epub::Metadata>,
    print_output: bool,
    debug: bool,
) -> Result<(), CliError> {
    const CONVERTING_ERROR_MSG: &str = "Error while converting";
    const SPINNER_TEMPLATE: &str = "{spinner:.green} {msg:.green}";
    const BAR_TEMPLATE: &str = "{elapsed_precise} [{wide_bar}] {human_pos}/{human_len} {percent}% ";

    fn setup_spinner(file: &Path, sp: &ProgressBar) {
        let file_name = file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Cannot get file name!");

        sp.set_style(
            ProgressStyle::default_spinner()
                .template(SPINNER_TEMPLATE).unwrap()
        );
        sp.enable_steady_tick(std::time::Duration::from_millis(100));
        sp.set_message(file_name.to_owned());
    }


    if files.len() > 1 {
        let pool = ThreadPool::new(5);
        let m = MultiProgress::new();
        let bar = m.add(
            ProgressBar::new(files.len().try_into().unwrap())
                .with_style(
                    ProgressStyle::default_bar()
                        .template(BAR_TEMPLATE).unwrap()
                        .progress_chars("=> ")
                )
        );
        let styles = std::sync::Arc::new(styles);

        while let Some(file) = files.pop() {
            let bar = bar.clone();
            let sp = m.insert_before(&bar,ProgressBar::new_spinner());

            let styles = styles.clone();
            let metadata = metadata.clone();
            pool.execute(move || {
                setup_spinner(&file.0, &sp);
                bar.tick();

                match run(
                    &file.0,
                    &file.1,
                    styles.as_deref(),
                    metadata,
                    true,
                    debug
                ) {
                    Ok(_) => if print_output {
                        bar.println(format!("Saved to {:#?}", file.1));
                    },
                    Err(err) => bar.println(format!("{CONVERTING_ERROR_MSG} {:#?}: {err}", file.0)),
                };

                sp.finish_and_clear();
                bar.inc(1);
            });
        }

        pool.join();
    } else {
        let file = files.pop()
            .expect("Vec should have only one element");
    
        let sp = ProgressBar::new_spinner();
        setup_spinner(&file.0, &sp);
    
        run(
            file.0,
            &file.1,
            styles.as_deref(),
            metadata,
            true,
            debug
        )?;
        if print_output {
            sp.println(format!("Saved to {:#?}", file.1));
        }

        sp.finish_and_clear();
    };


    Ok(())
}
#[cfg(target_os = "windows")]
fn handle_converting(
    mut files: Vec<(PathBuf, PathBuf)>,
    styles: Option<PathBuf>,
    metadata: Option<fb2epub::Metadata>,
    print_output: bool,
    debug: bool,
) -> Result<(), CliError> {
    const CONVERTING_ERROR_MSG: &str = "Error while converting";

    if files.len() > 1 {
        let pool = ThreadPool::new(5);
        let styles = std::sync::Arc::new(styles);

        while let Some(file) = files.pop() {
            let styles = styles.clone();
            let metadata = metadata.clone();
            pool.execute(move || {
                match run(
                    &file.0,
                    &file.1,
                    styles.as_deref(),
                    metadata,
                    true,
                    debug
                ) {
                    Ok(_) => if print_output {
                        println!("Saved to {:#?}", file.1);
                    },
                    Err(err) => eprintln!("{CONVERTING_ERROR_MSG} {:#?}: {err}", file.0)
                }
            });
        }
        pool.join();
    } else {
        let file = files.pop()
            .expect("Vec should have only one element");

        run(
            file.0,
            &file.1,
            styles.as_deref(),
            metadata,
            true,
            debug
        )?;
        if print_output {
            println!("Saved to {:#?}", file.1);
        }
    }


    Ok(())
}

fn handle_pipe(
    args: args::Args
) -> Result<(), fb2epub::Error> {
    use std::io::{self, BufReader, BufWriter};


    let styles_path: Option<PathBuf> = if let Some(ref styles) = args.styles {
        let s_path = PathBuf::from(styles);
        if s_path.is_file() {Some(s_path)}
        else {None}
    } else {None};

    let metadata = parse_meta_from_args(
        args.title,
        args.authors,
        args.language,
        args.series,
        args.series_index,
    );

    let reader = BufReader::new(io::stdin());
    let writer = BufWriter::new(io::stdout());
    fb2epub::convert(
        reader,
        writer,
        styles_path.as_deref(),
        metadata,
        false, // suspend_error_messages
        false, // debug
    )
}

fn run<I: AsRef<Path>, O: AsRef<Path>>(
    input: I,
    output: O,
    styles_path: Option<&Path>,
    metadata: Option<fb2epub::Metadata>,
    suspend_error_messages: bool,
    debug: bool
) -> Result<(), CliError> {
    #[cfg(feature = "zip")]
    if input
        .as_ref()
        .extension()
        .is_some_and(|s|
            s.to_string_lossy().to_lowercase().as_str() == "zip"
        )
    {
        return zip_reader::convert_archive(
            input.as_ref(),
            output.as_ref(),
            styles_path,
            metadata,
            suspend_error_messages,
            debug
        ).map_err(CliError::ZipReader)
    };


    let file = fs::File::open(input)
        .map_err(|err| CliError::Other(err.to_string()))?;

    let reader = std::io::BufReader::new(file);


    let file = fs::File::create(output.as_ref())
        .map_err(|err| CliError::Other(err.to_string()))?;

    let writer = std::io::BufWriter::new(file);


    let result = fb2epub::convert(
        reader,
        writer,
        styles_path,
        metadata,
        suspend_error_messages,
        debug
    );
    if result.is_err() {
        fs::remove_file(output)
            .map_err(|err| CliError::Other(err.to_string()))?;
    }


    result.map_err(CliError::Converting)
}


fn parse_meta_from_args(
    title: Option<String>,
    authors: Option<Vec<String>>,
    language: Option<String>,
    series: Option<String>,
    series_index: Option<String>,
) -> Option<fb2epub::Metadata> {
    let metadata = fb2epub::Metadata {
        title,
        authors,
        language,
        series,
        series_index,
        description: None
    };

    if metadata.title.is_none() &&
        metadata.authors.is_none() &&
        metadata.language.is_none() &&
        metadata.series.is_none() &&
        metadata.series_index.is_none() &&
        metadata.description.is_none() 
    { None }
    else { Some(metadata) }
}



fn get_inputs(inputs: Vec<PathBuf>, recursive: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files: Vec<PathBuf> = Vec::new();
    for path in inputs {
        // Проверки
        if !path.exists() {
            return Err(
                std::io::Error::other(
                    format!("There's no such path: {:?}!", path)
                )
            )
        };
        
        if path.is_dir() {
            if let Err(err) = read_dir(&path, &mut files, recursive) {
                return Err(
                    std::io::Error::other(
                        format!("Error while reading directory {:#?}: {}!", path, err)
                    )
                )
            };
            continue
        };
        
        if is_allowed(&path) && !files.contains(&path) {
            files.push(path);
        }
    }


    Ok(files)
}
fn read_dir(dir: &Path, files: &mut Vec<PathBuf>, recursive: bool) -> std::io::Result<()> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            if recursive {read_dir(&path, files, recursive)?}
            continue
        } else if is_allowed(&path) && !files.contains(&path) {
            files.push(path)
        }
    }
    
    Ok(())
}
fn is_allowed(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext|
        ALLOWED_EXTENSIONS.contains(
            &ext.to_string_lossy().to_lowercase().as_str()
        )
    )
}

fn get_output( // this funcion only needs when inputs.len() == 1
    inputs: &[PathBuf],
    output: Option<String>, // if output is some and is dir then is is already exists
) -> Result<Vec<PathBuf>, std::io::Error> {
    use std::io::Error;


    let output = if let Some(o) = &output {
        let p = PathBuf::from(o);

        if p.is_dir() || o.ends_with(MAIN_SEPARATOR) {
            let input_path = &inputs[0];
            let stem = input_path
                .file_stem()
                .ok_or(Error::other("Cannot get file name!"))?
                .to_string_lossy()
                .to_string();
            let parent = p;

            get_free_path(&stem, "epub", &parent, &[])
        } else { p }
    } else {
        get_out_path(&inputs[0], &[])
            .ok_or(Error::other("Cannot get output path!"))?
    };


    Ok(vec![output])
}

fn get_outputs( // this funcion only needs when inputs.len() > 1
    inputs: &[PathBuf],
    output: Option<String> // if output is some, then it is dir and it is already exists
) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut outputs = Vec::new();
    for input in inputs {
        let out = 
            if let Some(o) = &output { 
                let p = PathBuf::from(o);
                get_out_path_with_parent(input, &p, &outputs)
            } else {
                get_out_path(input, &outputs)
            }.ok_or(std::io::Error::other(format!("Cannot get output path for {:#?}", input)))?;

        outputs.push(out);
    }


    Ok(outputs)
}
fn get_out_path(
    input: &Path,
    outputs: &[PathBuf]
) -> Option<PathBuf> {
    let parent = input.parent()?;
    get_out_path_with_parent(input, parent, outputs)
}
fn get_out_path_with_parent(
    input: &Path,
    parent: &Path,
    outputs: &[PathBuf]
) -> Option<PathBuf> {
    let mut stem = input.file_stem()?.to_str()?.to_string();
    if stem.to_lowercase().ends_with(".fb2") {
        stem = stem[..stem.len() - 4].to_string();
    }
    let ext = "epub";

    Some(get_free_path(&stem, ext, parent, outputs))
}
fn get_free_path(
    stem: &str,
    extension: &str,
    parent: &Path,
    outputs: &[PathBuf]
) -> PathBuf {
    let mut path = parent.join(format!("{stem}.{extension}"));
    let mut counter = 1;
    while path.exists() || outputs.contains(&path) {
        path = parent.join(format!("{stem}-{counter}.{extension}"));
        counter += 1;
    }

    path
}

fn set_output(output: &str, inputs_len: usize) -> std::io::Result<()> {
    let output_path = PathBuf::from(output);

    if inputs_len > 1 || output.ends_with(MAIN_SEPARATOR) {
        if output_path.is_file() {
            fs::remove_file(&output_path)?;
        }

        if !output_path.is_dir() {
            fs::create_dir_all(&output_path)?;
        }
    } else if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}
