extern crate fb2epub;

use std::path::{PathBuf, Path};
use std::fs;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use threadpool::ThreadPool;



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input files. Can be a file or a directory. Also you can use it many times
    #[arg(short, long, num_args = 1.., required = true)]
    input: Vec<String>,
    
    /// Output path. Directory. If there's only one input book also can be a file.
    #[arg(short, long)]
    output: Option<String>,
    
    /// Custom css styles for a book. Path to a .css file
    #[arg(long)]
    styles: Option<String>,
    
    /// Include all books from subdirs of given in --input directory.
    #[arg(short, long)]
    recursive: bool,

    /// DELETE inputs files after convertation.
    #[arg(long)]
    replace: bool,

    /// Use debug mod
    #[arg(long)]
    debug: bool,


    /// Use given title for input book(s)
    #[arg(long)]
    title: Option<String>,

    /// Use given author(s) for input book(s)
    #[arg(long, num_args = 1..)]
    author: Option<Vec<String>>,

    /// Use given language for input book(s)
    #[arg(long)]
    language: Option<String>,

    /// Use given series for input book(s)
    #[arg(long)]
    series: Option<String>,

    /// Use given series index for input book(s)
    #[arg(long)]
    series_index: Option<String>
}

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


fn main() {
    let args = Args::parse();
    let files = get_files(&args.input, args.recursive);
    if files.is_empty() {
        panic!("There's no fb2 books in the input!")
    };
    
    let output = match args.output {
        Some(ref o) => {
            let output_path = PathBuf::from(o);
            if files.len() > 1 {
                if output_path.is_dir() {
                    Some(output_path)
                } else {
                    fs::create_dir_all(&output_path)
                        .expect("Error while creating output folder!");
                    Some(output_path)
                }
            } else {
                Some(output_path)
            }
        }
        None => None
    };
    
    let styles_path: Option<PathBuf> = if let Some(ref styles) = args.styles {
        let s_path = PathBuf::from(styles);
        if s_path.is_file() {Some(s_path)}
        else {None}
    } else {None};

    let metadata = parse_meta_from_args(&args);

    
    if files.len() > 1 {
        let pool = ThreadPool::new(10);

        if is_windows() || args.debug {
            for file in files {
                let output = if let Some(o) = get_out_path(&file, output.clone()) {o}
                else {
                    eprintln!("Cannot get output path for {:#?}", file);
                    continue;
                };
        
                let styles_path = styles_path.clone();
                let metadata = metadata.clone();
                pool.execute(move || {
                    match fb2epub::run(
                        &file,
                        &output,
                        args.replace,
                        styles_path.as_deref(),
                        metadata,
                        true,
                        args.debug
                    ) {
                        Ok(o) => println!("Saved to {:#?}", o),
                        Err(err) => eprintln!("{err}")
                    }
                });
            }
        } else {
            let bar = ProgressBar::new(files.len().try_into().unwrap());

            for file in files {
                let output = if let Some(o) = get_out_path(&file, output.clone()) {o}
                else {continue};

                let styles_path = styles_path.clone();
                let metadata = metadata.clone();
                let bar = bar.clone();
                pool.execute(move || {
                    match fb2epub::run(
                        &file,
                        &output,
                        args.replace,
                        styles_path.as_deref(),
                        metadata,
                        true,
                        args.debug
                    ) {
                        Ok(_) => {}, // bar.println(format!("Saved to {:#?}", o)),
                        Err(err) => bar.println(format!("{}", err))
                    };
                    bar.inc(1);
                });
            }
        };

        pool.join();
    } else {
        if is_windows() || args.debug {
            let file = &files[0];
            let output = get_out_path(file, output.clone())
                .expect("Cannot get output path!");
    
            match fb2epub::run(
                file,
                &output,
                args.replace,
                styles_path.as_deref(),
                metadata,
                true,
                args.debug
            ) {
                Ok(o) => println!("Saved to {:#?}", o),
                Err(err) => eprintln!("{err}")
            }
        } else {
            let file = &files[0];
            let output = get_out_path(file, output.clone())
                .expect("Cannot get output path!");

        
            let file_name = file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Cannot get file name!");
            
            let sp = ProgressBar::new_spinner();
            sp.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg:.green}").unwrap()
            );
            sp.enable_steady_tick(std::time::Duration::from_millis(100));
            sp.set_message(file_name.to_owned());
        
            if let Err(err) = fb2epub::run(
                file,
                &output,
                args.replace,
                styles_path.as_deref(),
                metadata,
                true,
                args.debug
            ) {
                eprintln!("{err}")
            };
            
            sp.finish_and_clear();
        }
    }
}

fn parse_meta_from_args(args: &Args) -> Option<fb2epub::Metadata> {
    let metadata = fb2epub::Metadata {
        title: args.title.clone(),
        authors: args.author.clone(),
        language: args.language.clone(),
        series: args.series.clone(),
        series_index: args.series_index.clone(),
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



fn is_allowed(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext|
        ALLOWED_EXTENSIONS.contains(
            &ext.to_string_lossy().to_lowercase().as_str()
        )
    )
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

fn get_files(inputs: &Vec<String>, recursive: bool) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    for i in inputs {
        let path = PathBuf::from(i);
        
        // Проверки
        if !path.exists() {
            eprintln!("There's no such path: {:?}!", path);
            continue
        };
        
        if path.is_dir() {
            if let Err(err) = read_dir(&path, &mut files, recursive) {
                eprintln!("Error while reading directory {:#?}: {}!", path, err)
            };
            continue
        };
        
        if is_allowed(&path) && !files.contains(&path) {
            files.push(path);
        }
    }
    
    files
}

fn get_out_path(file: &Path, output: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(o_path) = output {
        if o_path.is_dir() {
            Some(o_path
                .join(file
                    .with_extension("epub")
                    .file_name()?
                )
            )
        } else {
            Some(o_path)
        }
    } else {
        let file_name = file
            .with_extension("epub")
            .file_name()?
            .to_str()?
            .to_string();

        let parent = file.parent()?;
        
        get_free_output(&parent.join(file_name))
    }
}

fn get_free_output(output: &Path) -> Option<PathBuf> {
    let mut file_name = output.file_stem()?.to_str()?;
    
    if file_name.ends_with(".fb2")
    && let Some(r_index) = file_name.rfind(".") {
        file_name = &file_name[..r_index]
    };
    
    let parent = output.parent()?;
    let mut free_output = parent.join(format!("{file_name}.epub"));
    
    let mut counter = 1;
    while free_output.exists() {
        free_output = parent.join(format!("{file_name}-{counter}.epub"));
        counter += 1;
    };
    

    Some(free_output.to_owned())
}


