mod fb2_parser;
mod epub_creator;
mod error;

pub use error::Fb2EpubError as Error;


use std::path::Path;
use std::io::{BufRead, Write};

use crate::fb2_parser::metadata_reader::Sequence;


/// Struct for replacing metadata from a book with yours
#[derive(Clone)]
pub struct Metadata {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub language: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<String>,
    pub description: Option<Vec<String>>
}

// Функция для вывода секций, удобно для дебага
fn print_sections(sections: &Vec<crate::fb2_parser::Section>, without_p: bool) {
    let mut s = String::new();
    let mut is_first = true;
    for section in sections {
        if is_first {
            is_first = false;
        } else {
            std::io::stdin().read_line(&mut s).unwrap();
            match s.trim() {
                "q" | "quit" => break,
                _ => {}
            };
            s.clear();
        }
        std::process::Command::new("clear").status().unwrap();
        if without_p {
            dbg!(&section.level);
            dbg!(&section.file_name);
            dbg!(&section.id);
            dbg!(&section.title);
        } else {
            dbg!(&section);
        };
    };
}


pub fn convert<R: BufRead, W: Write>(
    reader: R, 
    writer: W,
    styles_path: Option<&Path>,
    metadata: Option<Metadata>,
    suspend_error_messages: bool,
    debug: bool
) -> Result<(), Error> {
    // Чтение входного FB2
    let mut data = fb2_parser::parse(reader).map_err(Error::Fb2Parser)?;
    if debug {
        print_sections(&data.content, false);
    }
    
    
    if let Some(meta) = metadata {
        if let Some(title) = meta.title {
            data.meta.title = title
        }
        if let Some(authors) = meta.authors {
            data.meta.authors = authors
        }
        if let Some(language) = meta.language {
            data.meta.language = language
        }
        if let Some(series) = meta.series {
            if let Some(ref mut seq) = data.meta.sequence {
                seq.name = series
            } else {
                data.meta.sequence = Some(Sequence {
                    name: series,
                    number: String::new()
                })
            }
        }
        if let Some(series_index) = meta.series_index {
            if let Some(ref mut seq) = data.meta.sequence {
                seq.number = series_index
            } else {
                data.meta.sequence = Some(Sequence {
                    name: String::new(),
                    number: series_index
                })
            }
        }
        if let Some(description) = meta.description {
            data.meta.annotation = Some(description)
        }
    };
    
    // Создание EPUB
    epub_creator::create_epub(
        &mut data,
        writer,
        styles_path,
        suspend_error_messages
    ).map_err(Error::EpubCreator)?;


    Ok(())
}
