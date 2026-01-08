use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::fb2_parser::{get_href, get_attr};
use crate::fb2_parser::binary_reader::binary_reader;


#[derive(Debug, Clone, PartialEq)]
pub struct TextBlock {
    pub text: String,            // сам текст
    pub strong: bool,            // полужирный
    pub emphasis: bool,          // курсив
    pub strikethrough: bool,     // зачёркнутый
    pub code: bool,              // код
    pub sup: bool,               // верхний индекс
    pub sub: bool,               // нижний индекс
    
    pub link: Option<Link>     // ссылка
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub link: String,
    pub link_type: Option<String>
}

#[derive(Debug, Clone)]
pub struct Poem {
    pub title: Vec<String>,
    pub stanzas: Vec<Stanza>,
    pub paragraphs: Vec<Paragraph>,
    pub date: String
}

#[derive(Debug, Clone)]
pub struct Stanza {
    pub title: Vec<String>,
    pub v: Vec<Paragraph>
}

#[derive(Debug, Clone)]
pub enum Paragraph {
    Text(Vec<TextBlock>),
    Note(Section),
    Epigraph(Section),
    Cite(Section),
    Annotation(Section),
    Poem(Poem),
    V(String),
    TextAuthor(String),
    Subtitle(String),
    Image(Option<String>),
    EmptyLine
}

#[derive(Debug, Clone)]
pub struct Section {
    pub level: u8,
    pub id: Option<String>,
    pub title: Vec<String>,
    pub paragraphs: Vec<Paragraph>
}


pub fn content_reader<R>(b_data: &mut super::BookData, xml_reader: &mut Reader<R>, buf: &mut Vec<u8>, body_name: Option<String>) where R: BufRead {
    let decoder = xml_reader.decoder();
    let mut sections: Vec<Section> = Vec::new();
    
    let mut level: u8 = 0;
    let mut section_id: Vec<String> = Vec::new();
    let mut title: Vec<String> = Vec::new();
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut paragraph: Vec<TextBlock> = Vec::new();
    let mut temp_titles: Vec<Vec<String>> = Vec::new();
    let mut temp_paragraphs: Vec<Vec<Paragraph>> = Vec::new();
    
    let mut in_title = false;
    let mut in_subtitle = false;
    let mut in_p = false;
    
    let mut strong = false;
    let mut emphasis = false;
    let mut strikethrough = false;
    let mut code = false;
    let mut sup = false;
    let mut sub = false;
    
    let mut link: Option<Link> = None;


    let mut stanzas: Vec<Stanza> = Vec::new();
    let mut date = String::new();

    let mut in_text_author = false;

    let mut in_poem = false;
    let mut in_stanza = false;
    let mut in_v = false;
    let mut in_date = false;
    
    
    loop {
        match xml_reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"section" => {
                        section_id.push(get_attr(e, "id", decoder));
                        
                        if !paragraphs.is_empty() | !title.is_empty() {
                            sections.push(Section {
                                level: level,
                                id: if level > 0 {
                                    match section_id.pop() {
                                        Some(id) if id.is_empty() => None,
                                        Some(id) => Some(id),
                                        None => None
                                    }
                                } else {None},
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
                    b"a" => link = match get_href(e, decoder) {
                        Some(l) => {
                            let l_type: Option<String> = match get_attr(e, "type", decoder) {
                                s if s.is_empty() => None,
                                s => Some(s),
                            };
                            
                            Some(Link {
                                link: l,
                                link_type: l_type
                            })
                        },
                        None => None
                    },

                    b"text-author" => in_text_author = true,

                    b"epigraph" | b"annotation" | b"cite" => {
                        // перемещение для разделения
                        temp_titles.push(title.clone());
                        temp_paragraphs.push(paragraphs.clone());

                        title.clear();
                        paragraphs.clear();
                    },

                    b"poem" => {
                        in_poem = true;

                        temp_titles.push(title.clone());
                        temp_paragraphs.push(paragraphs.clone());

                        title.clear();
                        paragraphs.clear();
                    },
                    b"stanza" if in_poem => {
                        in_stanza = true;

                        temp_titles.push(title.clone());
                        temp_paragraphs.push(paragraphs.clone());

                        title.clear();
                        paragraphs.clear();
                    },
                    b"v" if in_stanza => in_v = true,
                    b"date" if in_poem => in_date = true,
                    _ => {}
                }
            }
            
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"body" => break,
                    b"section" => {
                        if !paragraphs.is_empty() | !title.is_empty() {
                            sections.push(Section {
                                level: level,
                                id: match section_id.pop() {
                                    Some(id) if id.is_empty() => None,
                                    Some(id) => Some(id),
                                    None => None
                                },
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

                    b"text-author" => in_text_author = false,
                    
                    b"epigraph" | b"annotation" | b"cite" => {
                        let sub_section = Section {
                            level: level + 1,
                            id: None,
                            title,
                            paragraphs
                        };

                        title = temp_titles.pop().unwrap();
                        paragraphs = temp_paragraphs.pop().unwrap();

                        paragraphs.push(
                            match e.name().as_ref() {
                                b"epigraph" => Paragraph::Epigraph(sub_section),
                                b"annotation" => Paragraph::Annotation(sub_section),
                                b"cite" => Paragraph::Cite(sub_section),
                                _ => Paragraph::EmptyLine
                            }
                        );
                    },

                    b"poem" => {
                        in_poem = false;

                        let poem = Poem {
                            title,
                            stanzas: stanzas.clone(),
                            paragraphs,
                            date: date.clone()
                        };

                        title = temp_titles.pop().unwrap();
                        paragraphs = temp_paragraphs.pop().unwrap();

                        paragraphs.push(Paragraph::Poem(poem));

                        stanzas.clear();
                        date.clear();
                    },
                    b"stanza" => {
                        in_stanza = false;

                        let stanza = Stanza {
                            title,
                            v: paragraphs
                        };

                        title = temp_titles.pop().unwrap();
                        paragraphs = temp_paragraphs.pop().unwrap();

                        stanzas.push(stanza);
                    },
                    b"v" => in_v = false,
                    b"date" => in_date = false,
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
                    } else if in_text_author {
                        paragraphs.push(Paragraph::TextAuthor(t_trimmed))
                    } else if in_date {
                        date = t_trimmed
                    } else if in_v {
                        paragraphs.push(Paragraph::V(t_trimmed))
                    } else if in_p {
                        paragraph.push(TextBlock {
                            text: t_trimmed,
                            strong,
                            emphasis,
                            strikethrough,
                            code,
                            sup,
                            sub,
                            link: link.clone()
                        });
                        link = None;
                    }
                }
            }
            
            Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"p" | b"empty-line" => paragraphs.push(Paragraph::EmptyLine),
                    b"image" => {
                        let href: Option<String> = get_href(e, decoder);
                        paragraphs.push(Paragraph::Image(href));
                    },
                    _ => {}
                }
            }
            
            Ok(Event::Eof) => break,
            
            Err(e) => {
                eprintln!("FB2 parser error while reading content: {}", e);
                break;
            }
            
            _ => {}
        }
        
        buf.clear();
    };
    if let Some(name) = body_name {
        if name == "notes".to_string() {
            let mut section = Section {
                level: 0,
                id: Some("NOTES".to_string()),
                title: Vec::new(),
                paragraphs: Vec::new()
            };
            
            let mut is_first = true;
            for sec in sections {
                if is_first && sec.level == 0 {
                    section.title = sec.title;
                    section.paragraphs = sec.paragraphs;
                    section.level = 0;
                    
                    is_first = false;
                } else if is_first {
                    section.paragraphs.push(Paragraph::Note(sec));
                    is_first = false;
                } else {
                    section.paragraphs.push(Paragraph::Note(sec));
                }
            }
            
            b_data.content.push(section);
        } else {
            b_data.content.extend(sections)
        }
    } else {
        b_data.content.extend(sections)
    }
    
    binary_reader(b_data, xml_reader, buf);
}
