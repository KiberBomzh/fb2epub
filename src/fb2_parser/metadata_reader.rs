use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::fb2_parser::get_href;


#[derive(Clone, Debug)]
pub struct Sequence {
    pub name: String,
    pub number: String
}

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub authors: Vec<String>,
    pub language: String,
    pub sequence: Option<Sequence>,
    pub annotation: Option<Vec<String>>,
    pub cover: Option<String>
}


pub fn metadata_reader<R>(xml_reader: &mut Reader<R>, buf: &mut Vec<u8>) -> Metadata where R: BufRead {
    let decoder = xml_reader.decoder();
    let mut meta = Metadata {
        title: String::new(),
        authors: Vec::new(),
        language: String::new(),
        sequence: None,
        annotation: None,
        cover: None
    };
    
    let mut in_title_info = false;
    
    let mut in_title = false;
    
    let mut in_author = false;
    let mut in_first_name = false;
    let mut in_middle_name = false;
    let mut in_last_name = false;
    let mut current_author = String::new();
    let mut first_name = String::new();
    let mut middle_name = String::new();
    let mut last_name = String::new();
    
    let mut in_annotation = false;
    let mut annotation: Vec<String> = Vec::new();
    
    let mut in_lang = false;
    let mut in_cover = false;
    
    let mut sequence = Sequence {
        name: String::new(),
        number: String::new()
    };
    
    
    loop {
        match xml_reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"title-info" => in_title_info = true,
                    b"book-title" if in_title_info => in_title = true,
                    
                    b"author" if in_title_info => in_author = true,
                    b"first-name" if in_author => in_first_name = true,
                    b"middle-name" if in_author => in_middle_name = true,
                    b"last-name" if in_author => in_last_name = true,
                    
                    b"annotation" if in_title_info => in_annotation = true,
                    b"lang" if in_title_info => in_lang = true,
                    b"coverpage" if in_title_info => in_cover = true,
                    _ => {}
                }
            }
            
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"title-info" => in_title_info = false,
                    b"book-title" => in_title = false,
                    b"author" => {
                        in_author = false;
                        
                        if !first_name.is_empty() {
                            current_author.push_str(first_name.trim());
                            current_author.push(' ');
                            first_name.clear();
                        };
                        
                        if !middle_name.is_empty() {
                            current_author.push_str(middle_name.trim());
                            current_author.push(' ');
                            middle_name.clear();
                        };
                        
                        if !last_name.is_empty() {
                            current_author.push_str(last_name.trim());
                            current_author.push(' ');
                            last_name.clear();
                        };
                        
                        
                        if !current_author.is_empty() {
                            meta.authors
                                .push(
                                    current_author
                                        .trim()
                                        .to_string()
                                );
                            
                            current_author.clear();
                        }
                    },
                    b"first-name" => in_first_name = false,
                    b"middle-name" => in_middle_name = false,
                    b"last-name" => in_last_name = false,
                    b"annotation" => {
                        in_annotation = false;
                        
                        if !annotation.is_empty() {
                            meta.annotation = Some(annotation.clone());
                            annotation.clear();
                        }
                    },
                    b"lang" => in_lang = false,
                    b"coverpage" => in_cover = false,
                    b"description" => break,
                    _ => {}
                }
            }
            
            Ok(Event::Text(e)) => {
                let text = e
                    .decode()
                    .unwrap()
                    .into_owned();
                
                if in_title {
                    meta.title = text;
                } else if in_first_name {
                    first_name = text;
                } else if in_middle_name {
                    middle_name = text;
                } else if in_last_name {
                    last_name = text;
                } else if in_annotation {
                    if !text.trim().is_empty() {
                        annotation.push(text.trim().to_string());
                    }
                } else if in_lang {
                    meta.language = text;
                }
            }
            
            Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"sequence" if in_title_info => {
                        sequence.name = match e.try_get_attribute("name") {
                            Ok(Some(attr)) => {
                                attr.decode_and_unescape_value(decoder).unwrap_or("".to_string().into()).to_string()
                            },
                            Ok(None) => "".to_string(),
                            Err(_) => "".to_string()
                        };
                        sequence.number = match e.try_get_attribute("number") {
                            Ok(Some(attr)) => {
                                attr.decode_and_unescape_value(decoder).unwrap_or("".to_string().into()).to_string()
                            },
                            Ok(None) => "".to_string(),
                            Err(_) => "".to_string()
                        };
                        
                        if !sequence.name.is_empty() {
                            meta.sequence = Some(sequence.clone());
                        };
                        sequence.name.clear();
                        sequence.number.clear();
                    },
                    b"image" if in_cover => meta.cover = get_href(e, decoder),
                    _ => {}
                }
            }
            
            Ok(Event::Eof) => break,
            
            Err(e) => {
                eprintln!("FB2 parser error while reading metadata: {}", e);
                break;
            }
            
            _ => {}
        }
        
        buf.clear();
    };
    buf.clear();
    
    return meta
}
