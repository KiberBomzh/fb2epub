#[cfg(feature = "zip")]
mod zip_reader;

mod args;


use std::path::{PathBuf, Path};
use std::fs;

use indicatif::{ProgressBar, ProgressStyle};
use threadpool::ThreadPool;


#[cfg(feature = "zip")]
const ALLOWED_EXTENSIONS: [&str; 2] = [
    "fb2",
    "zip"
];
#[cfg(not(feature = "zip"))]
const ALLOWED_EXTENSIONS: [&str; 1] = [
    "fb2",
];


#[cfg(target_os = "windows")]
fn is_windows() -> bool {true}

#[cfg(not(target_os = "windows"))]
fn is_windows() -> bool {false}


pub fn handle_cli() {
    let args = match args::Args::parse() {
        Ok(args) => args,
        Err(err) => panic!("Error while parsing cli args: {err}"),
    };
    if args.pipe {
        handle_pipe(args);
        return;
    }


    let mut inputs = get_inputs(args.inputs, args.recursive);
    if inputs.is_empty() {
        panic!("There's no fb2 books in the input!")
    };

    if let Some(p) = &args.output && inputs.len() > 1 && !p.is_dir() {
        if p.is_file() {
            let result = fs::remove_file(p);
            if let Err(err) = result {
                panic!("Error while setting output path: {err}")
            }
        }

        let result = fs::create_dir_all(p);
        if let Err(err) = result {
            panic!("Error while setting output path: {err}")
        }
    }

    let mut outputs: Vec<PathBuf> = if inputs.len() > 1 {
        match get_outputs(&inputs, args.output) {
            Ok(result) => result,
            Err(err) => panic!("Error while getting output path: {err}"),
        }
    } else {
        if let Some(o) = args.output {
            if o.is_dir() {
                let input_path = &inputs[0];
                let stem = input_path
                    .file_stem()
                    .expect("Cannot get output path!")
                    .to_string_lossy()
                    .to_string();
                let parent = o;
                let p = get_free_path(&stem, "epub", &parent, &[]);

                vec![p]

            } else {
                vec![o]
            }
        } else {
            match get_out_path(&inputs[0], &[]) {
                Some(o) => vec![o],
                None => panic!("Error while getting output path for: {:#?}", inputs[0]),
            }
        }
    };

    if inputs.len() != outputs.len() {
        panic!("Error while getting output path: inputs.len() and outputs.len() doesnt match!");
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


    if files.len() > 1 {
        let pool = ThreadPool::new(10);

        if is_windows() || args.debug {
            while let Some(file) = files.pop() {
                let styles_path = styles_path.clone();
                let metadata = metadata.clone();
                pool.execute(move || {
                    match run(
                        &file.0,
                        &file.1,
                        styles_path.as_deref(),
                        metadata,
                        true,
                        args.debug
                    ) {
                        Ok(_) => println!("Saved to {:#?}", file.1),
                        Err(err) => eprintln!("Error while converting {:#?}: {err}", file.0)
                    }
                });
            }
        } else {
            let bar = ProgressBar::new(files.len().try_into().unwrap());

            while let Some(file) = files.pop() {
                let styles_path = styles_path.clone();
                let metadata = metadata.clone();
                let bar = bar.clone();
                pool.execute(move || {
                    match run(
                        &file.0,
                        file.1,
                        styles_path.as_deref(),
                        metadata,
                        true,
                        args.debug
                    ) {
                        Ok(_) => {}, // bar.println(format!("Saved to {:#?}", o)),
                        Err(err) => bar.println(format!("Error while converting {:#?}: {err}", file.0))
                    };
                    bar.inc(1);
                });
            }
        };

        pool.join();
    } else {
        if is_windows() || args.debug {
            let file = files.pop()
                .expect("Vec should have only one element");
    
            match run(
                file.0,
                &file.1,
                styles_path.as_deref(),
                metadata,
                true,
                args.debug
            ) {
                Ok(_) => println!("Saved to {:#?}", file.1),
                Err(err) => panic!("Converting error: {err}")
            }
        } else {
            let file = files.pop()
                .expect("Vec should have only one element");
        
            let file_name = file.0.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Cannot get file name!");
            
            let sp = ProgressBar::new_spinner();
            sp.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg:.green}").unwrap()
            );
            sp.enable_steady_tick(std::time::Duration::from_millis(100));
            sp.set_message(file_name.to_owned());
        
            if let Err(err) = run(
                file.0,
                file.1,
                styles_path.as_deref(),
                metadata,
                true,
                args.debug
            ) {
                panic!("Converting error: {err}")
            };
            
            sp.finish_and_clear();
        }
    }
}

fn handle_pipe(
    args: args::Args
) {
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
    let result = fb2epub::convert(
        reader,
        writer,
        styles_path.as_deref(),
        metadata,
        false, // suspend_error_messages
        false, // debug
    );

    if let Err(err) = result {
        eprintln!("{err}");
    }
}

fn run<I: AsRef<Path>, O: AsRef<Path>>(
    input: I,
    output: O,
    styles_path: Option<&Path>,
    metadata: Option<fb2epub::Metadata>,
    suspend_error_messages: bool,
    debug: bool
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "zip")]
    if input
        .as_ref()
        .extension()
        .is_some_and(|s|
            s.to_string_lossy().to_lowercase().as_str() == "zip"
        )
    {
        zip_reader::convert_archive(
            input.as_ref(),
            output.as_ref(),
            styles_path,
            metadata,
            suspend_error_messages,
            debug
        )?;
        return Ok(())
    };


    let file = fs::File::open(input)?;
    let reader = std::io::BufReader::new(file);

    let file = fs::File::create(output.as_ref())?;
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
        fs::remove_file(output)?;
    }


    result
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



fn get_inputs(inputs: Vec<PathBuf>, recursive: bool) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    for path in inputs {
        // Проверки
        if !path.exists() {
            panic!("There's no such path: {:?}!", path);
        };
        
        if path.is_dir() {
            if let Err(err) = read_dir(&path, &mut files, recursive) {
                panic!("Error while reading directory {:#?}: {}!", path, err)
            };
            continue
        };
        
        if is_allowed(&path) && !files.contains(&path) {
            files.push(path);
        }
    }
    
    files
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


fn get_outputs( // this funcion only needs when inputs.len() > 1
    inputs: &[PathBuf],
    output: Option<PathBuf> // if output is some, then it is dir and it is already exists
) -> Result<Vec<PathBuf>, String> {
    let mut outputs = Vec::new();
    for input in inputs {
        let out = 
            if let Some(p) = &output { 
                get_out_path_with_parent(input, p, &outputs)
            } else {
                get_out_path(input, &outputs)
            }.ok_or(format!("Cannot get output path for {:#?}", input))?;

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
