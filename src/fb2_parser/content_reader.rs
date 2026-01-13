use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::fb2_parser::{get_href, get_attr};
use crate::fb2_parser::binary_reader::binary_reader;
use crate::fb2_parser::get_counter_str;


#[derive(Debug, Clone, PartialEq)]
pub struct TextBlock {
    pub text: String,            // сам текст
    pub strong: bool,            // полужирный
    pub emphasis: bool,          // курсив
    pub strikethrough: bool,     // зачёркнутый
    pub code: bool,              // код
    pub sup: bool,               // верхний индекс
    pub sub: bool,               // нижний индекс
    
    pub link: Option<Link>       // ссылка
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub link: String,
    pub link_type: Option<String>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Poem {
    pub level: u8,
    pub id: Option<String>,
    pub title: Vec<Paragraph>,
    pub stanzas: Vec<Stanza>,
    pub paragraphs: Vec<Paragraph>,
    pub date: Vec<TextBlock>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stanza {
    pub level: u8,
    pub id: Option<String>,
    pub title: Vec<Paragraph>,
    pub v: Vec<Paragraph>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Paragraph {
    Text(Vec<TextBlock>),
    V(Vec<TextBlock>),
    TextAuthor(Vec<TextBlock>),
    Subtitle(Vec<TextBlock>),
    Note(Section),
    Epigraph(Section),
    Cite(Section),
    Annotation(Section),
    Poem(Poem),
    Image(Option<String>),
    EmptyLine
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub level: u8,
    pub id: Option<String>,
    pub file_name: Option<String>,
    pub title: Vec<Paragraph>,
    pub paragraphs: Vec<Paragraph>
}


pub fn content_reader<R>(
        b_data: &mut super::BookData,
        xml_reader: &mut Reader<R>,
        buf: &mut Vec<u8>, 
        body_name: Option<String>,
        mut sections_counter: usize
    ) -> Result<(), Box<dyn std::error::Error>> where R: BufRead {

    let decoder = xml_reader.decoder();
    let mut sections: Vec<Section> = Vec::new();
    
    let mut level: u8 = 0;
    let mut section_id: Vec<String> = Vec::new();
    let mut title: Vec<Paragraph> = Vec::new();
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut paragraph: Vec<TextBlock> = Vec::new();
    let mut temp_titles: Vec<Vec<Paragraph>> = Vec::new();
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
    let mut date: Vec<TextBlock> = Vec::new();

    let mut in_text_author = false;

    let mut in_poem = false;
    let mut in_stanza = false;
    let mut in_v = false;
    let mut in_date = false;

    let mut current_file_name: String;
    let is_it_notes = match body_name {
        Some(ref s) if s == "notes" || s == "comments" => {
            current_file_name = s.clone() + ".xhtml";
            true
        },
        _ => {
            current_file_name = format!("section_{}.xhtml", get_counter_str(sections_counter + 1));
            false
        }
    };
    
    
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
                                        Some(id) => {
                                            let l_id = format!("#{id}");
                                            b_data.link_map.insert(l_id.clone(), current_file_name.clone() + &l_id);

                                            Some(id)
                                        },
                                        None => None
                                    }
                                } else {None},
                                file_name: if is_it_notes {None} 
                                    else {
                                        sections_counter += 1;
                                        Some(format!(
                                            "section_{}", get_counter_str(sections_counter)))
                                    },
                                title: title.clone(),
                                paragraphs: paragraphs.clone()
                            });
                            
                            if !is_it_notes {
                                current_file_name = format!("section_{}.xhtml", get_counter_str(sections_counter + 1))
                            };

                            title.clear();
                            paragraphs.clear();
                        };
                        
                        level += 1;
                    },
                    b"title" => {
                        in_title = true;
                        temp_paragraphs.push(paragraphs.clone());
                        paragraphs.clear();
                    },
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
                        section_id.push(get_attr(e, "id", decoder));
                        
                        // перемещение для разделения
                        temp_titles.push(title.clone());
                        temp_paragraphs.push(paragraphs.clone());

                        title.clear();
                        paragraphs.clear();
                    },

                    b"poem" => {
                        in_poem = true;
                        section_id.push(get_attr(e, "id", decoder));

                        temp_titles.push(title.clone());
                        temp_paragraphs.push(paragraphs.clone());

                        title.clear();
                        paragraphs.clear();
                    },
                    b"stanza" if in_poem => {
                        in_stanza = true;
                        section_id.push(get_attr(e, "id", decoder));

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
                                    Some(id) => {
                                            let l_id = format!("#{id}");
                                            b_data.link_map.insert(l_id.clone(), current_file_name.clone() + &l_id);

                                            Some(id)
                                        },
                                    None => None
                                },
                                file_name: if is_it_notes {None} 
                                    else {
                                        sections_counter += 1;
                                        Some(format!(
                                            "section_{}", get_counter_str(sections_counter)))
                                    },
                                title: title.clone(),
                                paragraphs: paragraphs.clone()
                            });
                            
                            if !is_it_notes {
                                current_file_name = format!("section_{}.xhtml", get_counter_str(sections_counter + 1))
                            };

                            title.clear();
                            paragraphs.clear();
                        };
                        
                        level -= 1;
                    },
                    b"title" => {
                        in_title = false;
                        title = paragraphs;
                        paragraphs = temp_paragraphs.pop().ok_or("Error while parsing fb2 content!")?;
                    },
                    b"subtitle" => {
                        in_subtitle = false;
                        if !paragraph.is_empty() {
                            paragraphs.push(Paragraph::Subtitle(paragraph.clone()));
                            paragraph.clear();
                        }
                    },
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

                    b"text-author" => {
                        in_text_author = false;
                        if !paragraph.is_empty() {
                            paragraphs.push(Paragraph::TextAuthor(paragraph.clone()));
                            paragraph.clear();
                        }
                    },
                    
                    b"epigraph" | b"annotation" | b"cite" => {
                        let sub_section = Section {
                            level: level + 1,
                            id: match section_id.pop() {
                                Some(id) if id.is_empty() => None,
                                Some(id) => {
                                    let l_id = format!("#{id}");
                                    b_data.link_map.insert(l_id.clone(), current_file_name.clone() + &l_id);

                                    Some(id)
                                },

                                None => None
                            },
                            file_name: None,
                            title,
                            paragraphs
                        };

                        title = temp_titles.pop().ok_or("Error while parsing fb2 content!")?;
                        paragraphs = temp_paragraphs.pop().ok_or("Error while parsing fb2 content!")?;

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
                            level: level + 1,
                            id: match section_id.pop() {
                                Some(id) if id.is_empty() => None,
                                Some(id) => {
                                    let l_id = format!("#{id}");
                                    b_data.link_map.insert(l_id.clone(), current_file_name.clone() + &l_id);

                                    Some(id)
                                },

                                None => None
                            },
                            title,
                            stanzas: stanzas.clone(),
                            paragraphs,
                            date: date.clone()
                        };

                        title = temp_titles.pop().ok_or("Error while parsing fb2 content!")?;
                        paragraphs = temp_paragraphs.pop().ok_or("Error while parsing fb2 content!")?;

                        paragraphs.push(Paragraph::Poem(poem));

                        stanzas.clear();
                        date.clear();
                    },
                    b"stanza" => {
                        in_stanza = false;

                        let stanza = Stanza {
                            level: level + 1,
                            id: match section_id.pop() {
                                Some(id) if id.is_empty() => None,
                                Some(id) => {
                                    let l_id = format!("#{id}");
                                    b_data.link_map.insert(l_id.clone(), current_file_name.clone() + &l_id);

                                    Some(id)
                                },
                                None => None
                            },
                            title,
                            v: paragraphs
                        };

                        title = temp_titles.pop().ok_or("Error while parsing fb2 content!")?;
                        paragraphs = temp_paragraphs.pop().ok_or("Error while parsing fb2 content!")?;

                        stanzas.push(stanza);
                    },
                    b"v" => {
                        in_v = false;
                        if !paragraph.is_empty() {
                            paragraphs.push(Paragraph::V(paragraph.clone()));
                            paragraph.clear();
                        }
                    },
                    b"date" => in_date = false,
                    _ => {}
                }
            }
            
            Ok(Event::Text(e)) => {
                let text = e
                    .decode()?
                    .into_owned();
                
                if !text.trim().is_empty() {
                    let t_trimmed = text.trim().to_string();
                    if in_date {
                        date.push(TextBlock {
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
                    } else if in_p || in_v || in_text_author || in_subtitle || in_title {
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
        if &name == "notes" || &name == "comments" {
            let mut section = Section {
                level: 0,
                id: None,
                file_name: Some(name.clone()),
                title: Vec::new(),
                paragraphs: Vec::new()
            };
            
            let mut is_first = true;
            for sec in sections {
                if is_first && sec.level == 0 {
                    section.id = sec.id;
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
    
    binary_reader(b_data, xml_reader, buf, sections_counter)?;

    Ok(())
}
