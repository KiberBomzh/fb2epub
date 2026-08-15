use std::path::PathBuf;

use clap::Parser;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Input files. Can be a file or a directory. Also you can use it many times
    #[arg(short, long, num_args = 1..)]
    pub input: Vec<PathBuf>,
    
    /// Output path. Directory. If there's only one input book also can be a file.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Custom css styles for a book. Path to a .css file
    #[arg(long)]
    pub styles: Option<PathBuf>,
    
    /// Include all books from subdirs of given in --input directory.
    #[arg(short, long)]
    pub recursive: bool,

    /// Read input (only fb2) from stdin, write epub in stdout.
    #[arg(short, long)]
    pub pipe: bool,

    /// Use debug mod
    #[arg(long)]
    pub debug: bool,


    /// Use given title for input book(s)
    #[arg(long)]
    pub title: Option<String>,

    /// Use given author(s) for input book(s)
    #[arg(long, num_args = 1..)]
    pub author: Option<Vec<String>>,

    /// Use given language for input book(s)
    #[arg(long)]
    pub language: Option<String>,

    /// Use given series for input book(s)
    #[arg(long)]
    pub series: Option<String>,

    /// Use given series index for input book(s)
    #[arg(long)]
    pub series_index: Option<String>
}
