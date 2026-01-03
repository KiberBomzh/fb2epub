mod metadata_reader;
mod content_reader;

use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

use quick_xml::reader::Reader;
use crate::fb2_parser::metadata_reader::metadata_reader;
use crate::fb2_parser::content_reader::content_reader;


pub use crate::fb2_parser::metadata_reader::Metadata;
pub use crate::fb2_parser::content_reader::Section;


#[derive(Debug)]
pub struct BookData {
    pub meta: Metadata,
    pub content: Vec<Section>,
    pub images: HashMap<String, Image>
} // в images будут биннарные данные изображений

#[derive(Clone, Debug)]
pub struct Image {
    id: String,
    content_type: String,
    binary: String
}


// Функция для вывода секций, удобно для дебага
fn sections_reader(sections: &Vec<Section>, without_p: bool) {
    let mut s = String::new();
    for section in sections {
        std::process::Command::new("clear").status().unwrap();
        
        if without_p {
            dbg!(&section.level);
            dbg!(&section.title);
        } else {
            dbg!(&section);
        };
        
        std::io::stdin().read_line(&mut s).unwrap();
        s.clear();
    };
}

pub fn get_data(book: &PathBuf) -> BookData {
    let file = File::open(book).unwrap();
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);
    let mut buf = Vec::new();
    
    
    let mut data = BookData {
        meta: metadata_reader(&mut xml_reader, &mut buf),
        content: Vec::new(),
        images: HashMap::new()
    };
    content_reader(&mut data, &mut xml_reader, &mut buf);
    // sections_reader(&data.content, false);
    
    return data
}