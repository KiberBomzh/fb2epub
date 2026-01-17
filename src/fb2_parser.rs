pub mod metadata_reader;
pub mod content_reader;
pub mod binary_reader;


use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

use quick_xml::reader::Reader;
use quick_xml::events::BytesStart;
use quick_xml::encoding::Decoder;


use crate::fb2_parser::metadata_reader::metadata_reader;
use crate::fb2_parser::content_reader::content_reader;

pub use crate::fb2_parser::metadata_reader::Metadata;
pub use crate::fb2_parser::content_reader::Section;



#[derive(Debug)]
pub struct BookData {
    pub meta: Metadata,
    pub content: Vec<Section>,
    pub images: HashMap<String, Image>,
    pub link_map: HashMap<String, String>
} // в images будут биннарные данные изображений
  // в link_map первое значение это ссылка как она была в fb2
  // второе значение - новая ссылка

#[derive(Clone, Debug)]
pub struct Image {
    pub id: String,
    pub content_type: String,
    pub binary: String
}


fn get_href(e: &BytesStart, decoder: Decoder) -> Option<String> {
    for attr_result in e.attributes() {
        if let Ok(attr) = attr_result {
            let key = String::from_utf8_lossy(attr.key.as_ref());
            if key.contains("href") {
                return match attr.decode_and_unescape_value(decoder)
                    .and_then(|s| Ok(s.to_string())).unwrap_or(String::new()) {
                        s if s.is_empty() => None,
                        s => Some(s)
                    }
            } else { continue }
        }
    };

    return None
}

fn get_attr(e: &BytesStart, query: &str, decoder: Decoder) -> String {
    match e.try_get_attribute(query) {
        Ok(Some(attr)) => {
            attr
                .decode_and_unescape_value(decoder)
                .unwrap_or(
                    "".to_string().into()
                ).to_string()
        },
        Ok(None) => "".to_string(),
        Err(_) => "".to_string()
    }
}

pub fn get_counter_str(c: usize) -> String {
    if c < 10 {
        format!("00{c}")
    } else if c < 100 {
        format!("0{c}")
    } else {
        c.to_string()
    }
}


pub fn get_data(book: &Path) -> Result<BookData, Box<dyn std::error::Error>> {
    let file = File::open(book)?;
    let reader = BufReader::new(file);
    let mut xml_reader = Reader::from_reader(reader);
    let mut buf = Vec::new();
    let sections_counter = 0;
    
    
    let mut data = BookData {
        meta: metadata_reader(&mut xml_reader, &mut buf)?,
        content: Vec::new(),
        images: HashMap::new(),
        link_map: HashMap::new()
    };
    content_reader(&mut data, &mut xml_reader, &mut buf, None, sections_counter)?;
    
    return Ok(data)
}
