# fb2epub
Cli tool for convering fb2 books to epub. Written in pure rust.
## Installation
With cargo:
```
cargo install fb2epub --features=bin-deps
```
Or download binary files [here](https://github.com/KiberBomzh/fb2epub/releases/latest).
## Flags
- `-i`, `--input` `path` - input books or directories or zip archive with books
- `-o`, `--output` `path` - output path. If input is one book - can be directory or file name, else - only directory
- `--styles` `path/to/file.css` - use custom css styles
- `-r`, `--recursive` - search books as well in subdirectories 
- `-m`, `--multithreading` - use multithreading (useful if you need to convert many books)
- `--replace` - **REMOVE** input files

## Usage as library
Add to your project with:
```
cargo add fb2epub
```

Then use function `run`:
```rust
use std::path::PathBuf;

fn main() {
    let input_book = PathBuf::from("some_book.fb2");
    let output_book = PathBuf::from("out_book.epub");
    
    // delete input book
    let replace = false;
    
    // dont show small errors (image decoder errors, etc)
    let suspend_error_messages = false;
    
    // path to css styles Option<PathBuf>, if None will be used default styles
    let styles = None;
    
    
    fb2epub::run(
        &input_book,
        &output_book,
        replace,
        &styles,
        suspend_error_messages
    ).unwrap(); // returns Result<PathBuf>
    // PathBuf is path to output book
    
    // as well you can convert zip
    let input_archive = PathBuf::from("some_book.zip");
    let output_archive = PathBuf::from("out_archive.epub");
    
    fb2epub::run(
        &input_archive,
        &output_archive,
        replace,
        &styles,
        suspend_error_messages
    ).unwrap();
    
    
    // or even zip with many books in it
    let zip_with_many_books = PathBuf::from("zip_with_books.zip");
    
    // for it output path must be a directory
    let output_dir = PathBuf::from("some_dir");
    
    fb2epub::run(
        &zip_with_many_books,
        &output_dir,
        replace,
        &styles,
        suspend_error_messages
    ).unwrap();
}
```