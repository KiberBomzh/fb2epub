use std::io::BufRead;
use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::fb2_parser::get_attr;
use crate::fb2_parser::Image;
use crate::fb2_parser::content_reader::content_reader;


pub fn binary_reader<R>(b_data: &mut super::BookData, xml_reader: &mut Reader<R>, buf: &mut Vec<u8>) where R: BufRead {
    let decoder = xml_reader.decoder();
    let mut images: HashMap<String, Image> = HashMap::new();
    
    let mut in_binary = false;
    let mut current_image = Image {
        id: String::new(),
        content_type: String::new(),
        binary: String::new()
    };
    
    let mut is_it_body = false;
    let mut body_name: Option<String> = None;
    
    
    loop {
        match xml_reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"body" => {
                        is_it_body = true;
                        body_name = match get_attr(e, "name", decoder) {
                            s if s.is_empty() => None,
                            s => Some(s)
                        };
                        
                        break
                    },
                    b"binary" => {
                        in_binary = true;
                        
                        current_image.id = get_attr(e, "id", decoder);
                        current_image.content_type = get_attr(e, "content-type", decoder);
                    },
                    _ => {}
                }
            }
            
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"binary" => {
                        in_binary = false;
                        
                        if !current_image.id.is_empty() {
                            images.insert(
                                current_image.id.clone(),
                                current_image.clone()
                            );
                        };
                        
                        current_image.id.clear();
                        current_image.content_type.clear();
                        current_image.binary.clear();
                    },
                    _ => {}
                }
            }
            
            Ok(Event::Text(e)) => {
                let text = e
                    .decode()
                    .unwrap()
                    .into_owned();
                
                
                let mut text_trimmed = text.trim().to_string();
                if !text_trimmed.is_empty() {
                    if in_binary {
                        text_trimmed = text_trimmed.replace("\r\n", "");
                        text_trimmed = text_trimmed.replace("\n", "");
                        text_trimmed = text_trimmed.replace(" ", "");
                        
                        current_image.binary.push_str(&text_trimmed);
                    }
                }
            }
            
            Ok(Event::Eof) => break,
            
            Err(e) => {
                eprintln!("FB2 parser error while reading binary: {}", e);
                break;
            }
            
            _ => {}
        }
        
        buf.clear();
    };
    
    if is_it_body {
        content_reader(b_data, xml_reader, buf, body_name);
    } else {
        b_data.images.extend(images);
    }
}

