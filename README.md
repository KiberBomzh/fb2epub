# fb2epub
Cli tool for convering fb2 books to epub. Written in pure rust.
## Installation
With cargo:
```
cargo install fb2epub
```
Or download binary files [here](https://github.com/KiberBomzh/fb2epub/releases/latest).
## Positional arguments
- `INPUTS...` - input books or directories or zip archives with books
## Flags
- `-o`, `--output` `path` - output path. If input is one book - can be directory or file name, else - only directory
- `--styles` `path/to/file.css` - use custom css styles
- `-r`, `--recursive` - search books as well in subdirectories 
- `-p`, `--pipe` - read book (only fb2) from stdin, write in stdout. `--output`, `--recursive` arguments will be ignored.
### Flags for metadata
- `--title` - set title for output book
- `--author` - set authors for output book
- `--language` - set language for output book
- `--series` - set series for output book
- `--series-index` - set series index for output book

## Usage as library
Add to your project with:
```
cargo add fb2epub --no-default-features
```

Then use function `convert`:
```rust
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    let input_book = File::open("some_book.fb2").unwrap();
    let reader = BufReader::new(input_book);

    let output_book = File::create("out_book.epub").unwrap();
    let writer = BufWriter::new(output_book);
    
    // dont show small errors (image decoder errors, etc)
    let suspend_error_messages = false;

    // use debug mode
    let debug = false;
    
    // path to css styles Option<&Path>, if None will be used default styles
    let styles = Some(PathBuf::from("some/styles.css"));
    
    // override output book metadata
    let metadata = fb2epub::Metadata {
        title: Some( "some title".to_string() ),
        authors: Some( vec![
            "Author One".to_string(),
            "Author NoOne".to_string(),
        ]),
        language: None,
        series: Some( "Very cool series".to_string() ),
        series_index: None,
        description: Some(vec![
            "Paragraph number one, some words...".to_string(),
            "Paragraph two".to_string(),
            "The short one, actually, no. It is the longest paragraph here.".to_string(),
        ]),
    };
    
    fb2epub::convert(
        reader,
        writer,
        styles.as_deref(),
        Some(metadata),
        suspend_error_messages,
        debug
    ).unwrap();
}
```
