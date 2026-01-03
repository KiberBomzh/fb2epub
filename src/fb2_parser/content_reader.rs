use std::io::BufRead;
use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::fb2_parser::Image;


#[derive(Debug, Clone)]
pub struct TextBlock {
    pub text: String,
    pub strong: bool,            // полужирный
    pub emphasis: bool,          // курсив
    pub strikethrough: bool,     // зачёркнутый
    pub code: bool,              // код
    pub sup: bool,               // верхний индекс
    pub sub: bool,               // нижний индекс
} // добавить сюда ещё и ссылки

#[derive(Debug, Clone)]
pub enum Paragraph {
    Text(Vec<TextBlock>),
    Subtitle(String),
    Image(Option<String>),
    EmptyLine
}

#[derive(Debug)]
pub struct Section { // добавить cite, epigraph, annotation
    pub level: u8,
    pub title: Vec<String>,
    pub paragraphs: Vec<Paragraph>,
}
// чёт придумать из ссылками на заметки и с картинками


pub fn content_reader<R>(b_data: &mut super::BookData, xml_reader: &mut Reader<R>, buf: &mut Vec<u8>) where R: BufRead {
    let mut sections: Vec<Section> = Vec::new();
    let mut images: HashMap<String, Image> = HashMap::new();
    
    let mut level: u8 = 0;
    let mut title: Vec<String> = Vec::new();
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut paragraph: Vec<TextBlock> = Vec::new();
    
    let mut in_title = false;
    let mut in_subtitle = false;
    let mut in_p = false;
    
    let mut strong = false;
    let mut emphasis = false;
    let mut strikethrough = false;
    let mut code = false;
    let mut sup = false;
    let mut sub = false;
    
    let mut in_binary = false;
    let mut current_image = Image {
        id: String::new(),
        content_type: String::new(),
        binary: String::new()
    };
    
    
    loop {
        match xml_reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"section" => {
                        if !paragraphs.is_empty() | !title.is_empty() {
                            sections.push(Section {
                                level: level,
                                title: title.clone(),
                                paragraphs: paragraphs.clone()
                            });
                            
                            
                            title.clear();
                            paragraphs.clear();
                        };
                        
                        level += 1;
                    },
                    b"title" => in_title = true,
                    b"subtitle" => in_subtitle = true,
                    b"p" => in_p = true,
                    
                    b"strong" => strong = true,
                    b"emphasis" => emphasis = true,
                    b"strikethrough" => strikethrough = true,
                    b"code" => code = true,
                    b"sup" => sup = true,
                    b"sub" => sub = true,
                    b"binary" => {
                        in_binary = true;
                        
                        current_image.id = match e.try_get_attribute("id") {
                            Ok(Some(attr)) => {
                                attr.unescape_value().unwrap_or("".to_string().into()).to_string()
                            },
                            Ok(None) => "".to_string(),
                            Err(_) => "".to_string()
                        };
                        current_image.content_type = match e.try_get_attribute("content-type") {
                            Ok(Some(attr)) => {
                                attr.unescape_value().unwrap_or("".to_string().into()).to_string()
                            },
                            Ok(None) => "".to_string(),
                            Err(_) => "".to_string()
                        };
                    },
                    _ => {}
                }
            }
            
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"section" => {
                        if !paragraphs.is_empty() | !title.is_empty() {
                            sections.push(Section {
                                level: level,
                                title: title.clone(),
                                paragraphs: paragraphs.clone()
                            });
                            
                            
                            title.clear();
                            paragraphs.clear();
                        };
                        
                        level -= 1;
                    },
                    b"title" => in_title = false,
                    b"subtitle" => in_subtitle = false,
                    b"p" => {
                        in_p = false;
                        if !paragraph.is_empty() {
                            paragraphs.push(Paragraph::Text(paragraph.clone()));
                            paragraph.clear();
                        }
                    },
                    
                    b"strong" => strong = false,
                    b"emphasis" => emphasis = false,
                    b"strikethrough" => strikethrough = false,
                    b"code" => code = false,
                    b"sup" => sup = false,
                    b"sub" => sub = false,
                    
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
                
                if !text.trim().is_empty() {
                    let t_trimmed = text.trim().to_string();
                    if in_title {
                        title.push(text)
                    } else if in_subtitle {
                        paragraphs.push(Paragraph::Subtitle(t_trimmed))
                    } else if in_p {
                        paragraph.push(TextBlock {
                            text: t_trimmed,
                            strong,
                            emphasis,
                            strikethrough,
                            code,
                            sup,
                            sub
                        })
                    } else if in_binary {
                        current_image.binary.push_str(&t_trimmed)
                    }
                }
            }
            
            Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"p" | b"empty-line" => paragraphs.push(Paragraph::EmptyLine),
                    b"image" => {
                        use std::borrow::Cow;
                        
                        let href: Option<String> = match e.try_get_attribute("l:href") {
                            Ok(Some(attr)) => {
                                match attr.unescape_value() {
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
                            },
                            Ok(None) => None,
                            Err(_) => None
                        };
                        paragraphs.push(Paragraph::Image(href));
                    },
                    _ => {}
                }
            }
            
            Ok(Event::Eof) => break,
            
            Err(e) => {
                eprintln!("FB2 parser error: {}", e);
                break;
            }
            
            _ => {}
        }
        
        buf.clear();
    };
    
    b_data.content = sections;
    b_data.images = images;
}