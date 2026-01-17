extern crate fb2epub;

use std::path::{PathBuf, Path};
use std::fs;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input files. Can be a file or a directory. Also you can use it many times
    #[arg(short, long, num_args = 1.., required = true)]
    input: Vec<String>,
    
    /// Output path. Directory. If there's only one input book as well can be a file.
    #[arg(short, long)]
    output: Option<String>,
    
    /// Custom css styles for a book. Path to .css file
    #[arg(long)]
    styles: Option<String>,
    
    /// Include all books from subdirs of given in --input dirrctory.
    #[arg(short, long)]
    recursive: bool,
    
    /// DELETE inputs files after convertation.
    #[arg(long)]
    replace: bool
}


#[cfg(target_os = "windows")]
fn is_windows() -> bool {true}

#[cfg(not(target_os = "windows"))]
fn is_windows() -> bool {false}


fn read_dir(dir: &Path, files: &mut Vec<PathBuf>, recursive: bool) -> std::io::Result<()> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            if recursive {read_dir(&path, files, recursive)?}
            continue
        };
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if let Some(ext) = extension.to_str() {
                    if ext.to_lowercase() == "fb2" || ext.to_lowercase() == "zip" {
                        if !files.contains(&path) {files.push(path)}
                    }
                }
            }
        }
    };
    
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
        
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if let Some(ext) = extension.to_str() {
                    if ext.to_lowercase() == "fb2" || ext.to_lowercase() == "zip" {
                        if !files.contains(&path) {files.push(path)}
                    }
                }
            }
        }
    };
    
    return files
}

fn get_out_name(file: &Path, output: Option<PathBuf>) -> Option<PathBuf> {
    let suffix = ".epub";
    let file_stem: &str = if let Some(name) = file.file_stem() {
        if let Some(n) = name.to_str() {
            n
        } else {
            "new_book"
        }
    } else {
        "new_book"
    };
    let file_name = file_stem.to_string() + suffix;
    
    let mut parent: PathBuf = file.parent()?.to_path_buf();
    
    if let Some(output) = output {
        if output.is_dir() {
            let mut o = output.clone();
            o.push(file_name);
            Some(o)
        } else {
            Some(output)
        }
    } else {
        parent.push(file_name);
        Some(parent)
    }
}

fn main() {
    let args = Args::parse();
    let files = get_files(&args.input, args.recursive);
    if files.is_empty() {
        panic!("There's no fb2 books in input!")
    };
    
    let output = match args.output {
        Some(o) => {
            let output_path = PathBuf::from(o);
            if files.len() > 1 {
                if output_path.is_dir() {
                    Some(output_path)
                } else {
                    fs::create_dir_all(&output_path)
                        .expect("Error while creating output folder");
                    Some(output_path)
                }
            } else {
                Some(output_path)
            }
        }
        None => None
    };
    
    let s_path: PathBuf;
    let styles_path: Option<&Path> = if let Some(styles) = args.styles {
        s_path = PathBuf::from(styles);
        if s_path.is_file() {Some(&s_path)}
        else {None}
    } else {None};
    
    
    for file in &files {
        let output = if let Some(o) = get_out_name(file, output.clone()) {
            o
        } else {continue};
        
        
        if is_windows() {
            match fb2epub::run(file, &output, args.replace, styles_path) {
                Ok(o) => println!("Saved to {:#?}", o),
                Err(err) => eprintln!("{err}")
            }
        } else {
            let file_name = if let Some(name) = file.file_name()
                .and_then(|n| n.to_str()) {name}
            else {continue};
            
            let sp = ProgressBar::new_spinner();
            sp.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg:.green}").unwrap()
            );
            sp.enable_steady_tick(std::time::Duration::from_millis(100));
            sp.set_message(file_name.to_owned());
        
            if let Err(err) = fb2epub::run(file, &output, args.replace, styles_path) {
                eprintln!("{err}")
            };
            
            sp.finish_and_clear();
        }
    };
}
