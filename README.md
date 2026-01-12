# fb2epub
Cli tool for convering fb2 books to epub. Written in pure rust, very fust.
## Flags
- `-i`, `--input` `path` - input books or directories or zip archive with books
- `-o`, `--output` `path` - output path. If input is one book - can be directory or file name, else - only directory
- `-styles` `path/to/file.css` - use custom css styles
- `-r`, `--recursive` - search books as well in subdirectories 
- `--replace` - *REMOVE* input files
