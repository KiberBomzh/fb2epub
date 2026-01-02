use std::io::BufRead;

use quick_xml::events::Event;
use quick_xml::reader::Reader;


#[derive(Debug, Clone)]
pub struct Text {
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
    Text(Vec<Text>),
    Subtitle(String),
    EmptyLine
}

#[derive(Debug)]
pub struct Section { // добавить cite, epigraph, annotation
    pub level: u8,
    pub title: Vec<String>,
    pub paragraphs: Vec<Paragraph>,
}
// чёт придумать из ссылками на заметки и с картинками


pub fn content_reader<R>(xml_reader: &mut Reader<R>, buf: &mut Vec<u8>) -> Vec<Section> where R: BufRead {
    let mut sections: Vec<Section> = Vec::new();
    
    let mut level: u8 = 0;
    let mut title: Vec<String> = Vec::new();
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut paragraph: Vec<Text> = Vec::new();
    
    let mut in_title = false;
    let mut in_subtitle = false;
    let mut in_p = false;
    
    let mut strong = false;
    let mut emphasis = false;
    let mut strikethrough = false;
    let mut code = false;
    let mut sup = false;
    let mut sub = false;
    
    
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
                        paragraph.push(Text {
                            text: t_trimmed,
                            strong,
                            emphasis,
                            strikethrough,
                            code,
                            sup,
                            sub
                        })
                    }
                }
            }
            
            Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"p" | b"empty-line" => paragraphs.push(Paragraph::EmptyLine),
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
    return sections
}