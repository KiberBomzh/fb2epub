pub mod metadata_reader;
pub mod content_reader;
pub mod binary_reader;


use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::borrow::Cow;

use quick_xml::reader::Reader;
use quick_xml::events::BytesStart;


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
    pub id: String,
    pub content_type: String,
    pub binary: String
}


fn get_href(e: &BytesStart) -> Option<String> {
    for attr_result in e.attributes() {
        if let Ok(attr) = attr_result {
            let key = String::from_utf8_lossy(attr.key.as_ref());
            if key.contains("href") {
                return match attr.unescape_value() {
                    Ok(Cow::Borrowed(v)) => Some(
                        if v.starts_with("#") {
                            v[1..].to_string()
                        } else {
                            v.to_string()
                        }
                        ),
                    Ok(Cow::Owned(v)) => Some(
                        if v.starts_with("#") {
                            v[1..].to_string()
                        } else {v.clone()}
                        ),
                    _ => None
                }
            } else { continue }
        }
    };

    return None
}

fn get_attr(e: &BytesStart, query: &str) -> String {
    match e.try_get_attribute(query) {
        Ok(Some(attr)) => {
            attr
                .unescape_value()
                .unwrap_or(
                    "".to_string().into()
                ).to_string()
        },
        Ok(None) => "".to_string(),
        Err(_) => "".to_string()
    }
}


// Функция для вывода секций, удобно для дебага
fn print_sections(sections: &Vec<Section>, without_p: bool) {
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
    // print_sections(&data.content, false);
    
    return data
}
