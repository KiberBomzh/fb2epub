use std::collections::HashMap;

use crate::fb2_parser::Section;
use crate::fb2_parser::content_reader::*;


const TAB: &str = "    ";


fn get_head(head_title: &str, id: &Option<String>) -> String {
    let mut s = format!(r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
{TAB}<head>
{TAB}{TAB}<title>{head_title}</title>
{TAB}{TAB}<link href="../stylesheet.css" rel="stylesheet" type="text/css"/>
{TAB}</head>{}"#, "\n");
    
    s.push_str(
        &match id {
            Some(i) => format!("{TAB}<body id=\"{i}\">\n"),
            None => format!("{TAB}<body>\n")
        }
    );
    
    return s
}

fn unwrap_title(level: u8, title: &Vec<String>, indent: usize) -> String {
    if !title.is_empty() {
        let tag_name = match level {
            0 | 1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            _ => "h6",
        };
        let tag_start = format!("<{tag_name}>");
        let tag_end = format!("</{tag_name}>");
        
        if title.len() > 1 {
            let start_line = TAB.repeat(indent).to_string() + &tag_start;
            let end_line = TAB.repeat(indent).to_string() + &tag_end;
            let mut result = String::new();
            
            result.push_str(&start_line);
            result.push('\n');
            let p_indent = TAB.repeat(indent + 1);
            
            for block in title {
                result.push_str(
                    &format!("{p_indent}<p>{block}</p>\n")
                )
            };
            
            result.push_str(&end_line);
            result.push('\n');
            
            return result
        } else {
            return TAB.repeat(indent).to_string() + &tag_start + &title[0] + &tag_end + "\n"
        }
    } else {
        return "".to_string()
    }
}

fn push_style_tags(s: &mut String, block: &TextBlock, end_tag: bool) {
    let mut tags: Vec<char> = Vec::new();
    
    if block.strong {
        tags.push('b')
    }
    if block.emphasis {
        tags.push('i')
    }
    if block.strikethrough {
        tags.push('s')
    }
    if block.code {
        tags.push('c')
    }
    if block.sup {
        tags.push('u') // от слова upper
    }
    if block.sub {
        tags.push('l') // от слова lower
    }
    
    
    if end_tag {
        for tag in tags.into_iter().rev() {
            s.push_str(match tag {
                'b' => "</b>",
                'i' => "</i>",
                's' => "</s>",
                'c' => "</code>",
                'u' => "</sup>",
                'l' => "/<sub>",
                _ => ""
            })
        }
    } else {
        for tag in &tags {
            s.push_str(&match tag {
                'b' => "<b>",
                'i' => "<i>",
                's' => "<s>",
                'c' => "<code>",
                'u' => "<sup>",
                'l' => "<sub>",
                _ => ""
            })
        }
    }
}

fn get_link_start(link: &Link) -> String {
    match &link.link_type {
        Some(t) if t == "note" => {
            format!("<a class=\"reference\" epub:type=\"noteref\" href=\"notes.xhtml#{0}\" id=\"{0}\">", link.link)
        },
        Some(t) => format!("<a href=\"{0}\" epub:type=\"{t}\">", link.link),
        None => format!("<a href=\"{}\">", link.link)
    }
}

fn unwrap_blocks(blocks: &Vec<TextBlock>, tabs: &str) -> String {
    let mut s = String::new();
    s.push_str(tabs);
    s.push_str("<p>");
    
    for (index, block) in blocks.into_iter().enumerate() {
        let mut left_part = String::new();
        let mut right_part = String::new();
        let mut is_note = false;
        push_style_tags(&mut right_part, &block, true);
        if let Some(link) = &block.link {
            right_part.push_str("</a>");
            left_part.push_str(&get_link_start(&link));
            if let Some(t) = &link.link_type {
                if t == "note" {is_note = true}
            };
        };
        push_style_tags(&mut left_part, &block, false);
        
        if index != 0 && !is_note {
            let punctuation_chars = ['.', ',', '!', '?', '-', ';', ':', '}', ']', ')', '»'];
            let start_bracets = ['«', '(', '{', '['];
            
            if !punctuation_chars.iter().any(|c| block.text.starts_with(*c)) {
                if !start_bracets.iter().any(|c| blocks[index - 1].text.ends_with(*c)) {
                    s.push(' ')
                }
            }
        }
        
        if !left_part.is_empty() {
            s.push_str(&left_part);
            s.push_str(&block.text);
            s.push_str(&right_part);
        } else {
            s.push_str(&block.text)
        }
    }
    
    s.push_str("</p>\n");
    
    return s
}

fn unwrap_img(href: &Option<String>, link_map: &HashMap<String, String>, tabs: &str) -> String {
    if let Some(k) = href {
        if let Some(link) = link_map.get(k) {
            format!("{tabs}<img alt=\"\" src=\"{link}\"/>\n")
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

fn unwrap_paragraph(paragraph: &Paragraph, link_map: &HashMap<String, String>, indent: usize) -> String {
    let tabs = TAB.repeat(indent);
    
    match paragraph {
        Paragraph::Text(blocks) => unwrap_blocks(blocks, &tabs),
        Paragraph::EmptyLine => format!("{tabs}<empty-line/>\n"),
        Paragraph::Subtitle(text) => format!("{tabs}<subtitle>{text}</subtitle>\n"),
        Paragraph::Image(href) => unwrap_img(href, link_map, &tabs),
        Paragraph::V(text) => format!("{tabs}<p class=\"v\">{text}</p>\n"),
        Paragraph::TextAuthor(text) => format!("{tabs}<p class=\"text-author\">{text}</p>\n"),
        Paragraph::Epigraph(sub_section) => unwrap_section(&sub_section, link_map, indent + 1, "epigraph"),
        Paragraph::Cite(sub_section) => unwrap_section(&sub_section, link_map, indent + 1, "cite"),
        Paragraph::Annotation(sub_section) => unwrap_section(&sub_section, link_map, indent + 1, "annotation"),
        Paragraph::Poem(poem) => unwrap_poem(&poem, link_map, indent + 1),
        Paragraph::Note(sub_section) => unwrap_section(&sub_section, link_map, indent + 2, "note")
    }
}

fn unwrap_poem(poem: &Poem, link_map: &HashMap<String, String>, indent: usize) -> String {
    let mut s = String::new();
    s.push_str(
        &unwrap_title(
            poem.level,
            &poem.title,
            indent
        )
    );
    
    for stanza in &poem.stanzas {
        s.push_str(&unwrap_stanza(&stanza, link_map, indent + 1))
    };
    
    for paragraph in &poem.paragraphs {
        s.push_str(&unwrap_paragraph(&paragraph, link_map, indent))
    };
    
    if !poem.date.is_empty() {
        s.push_str(
            &format!("{}<p class\"date\">{}</p>\n", TAB.repeat(indent), poem.date)
        )
    };
    
    let tabs = TAB.repeat(indent - 1);
    s = if let Some(i) = &poem.id {
        format!("{tabs}<div class=\"poem\" id=\"{i}\">\n{s}{tabs}</div>\n")
    } else {
        format!("{tabs}<div class=\"poem\">\n{s}{tabs}</div>\n")
    };
    
    return s
}

fn unwrap_stanza(stanza: &Stanza, link_map: &HashMap<String, String>, indent: usize) -> String {
    let mut s = String::new();
    s.push_str(
        &unwrap_title(
            stanza.level,
            &stanza.title,
            indent
        )
    );
    
    for paragraph in &stanza.v {
        s.push_str(&unwrap_paragraph(&paragraph, link_map, indent))
    };
    
    let tabs = TAB.repeat(indent - 1);
    s = if let Some(i) = &stanza.id {
        format!("{tabs}<div class=\"stanza\" id=\"{i}\">\n{s}{tabs}</div>\n")
    } else {
        format!("{tabs}<div class=\"stanza\">\n{s}{tabs}</div>\n")
    };
    
    return s
}

fn unwrap_section(section: &Section, link_map: &HashMap<String, String>, indent: usize, section_type: &str) -> String {
    let mut s = String::new();
    s.push_str(
        &unwrap_title(
            section.level,
            &section.title,
            indent
        )
    );
    
    for paragraph in &section.paragraphs {
        s.push_str(&unwrap_paragraph(&paragraph, link_map, indent))
    };
    
    let tabs = TAB.repeat(indent - 1);
    s = match section_type {
        "epigraph" => {
            if let Some(i) = &section.id {
                format!("{tabs}<div class=\"epigraph\" id=\"{i}\">\n{s}{tabs}</div>\n")
            } else {
                format!("{tabs}<div class=\"epigraph\">\n{s}{tabs}</div>\n")
            }
        },
        "cite" => {
            if let Some(i) = &section.id {
                format!("{tabs}<div class=\"cite\" id=\"{i}\">\n{s}{tabs}</div>\n")
            } else {
                format!("{tabs}<div class=\"cite\">\n{s}{tabs}</div>\n")
            }
        },
        "annotation" => {
            if let Some(i) = &section.id {
                format!("{tabs}<div class=\"annotation\" id=\"{i}\">\n{s}{tabs}</div>\n")
            } else {
                format!("{tabs}<div class=\"annotation\">\n{s}{tabs}</div>\n")
            }
        },
        "note" => {
            let first_tabs = TAB.repeat(indent - 2);
            let left_part = if let Some(i) = &section.id {
                format!("{first_tabs}<div class=\"notes\">\n{tabs}<aside epub:type=\"footnote\" class=\"note\" id=\"{i}\">\n")
            } else {
                format!("{first_tabs}<div class=\"notes\">\n{tabs}<aside epub:type=\"footnote\" class=\"note\">\n")
            };
            let right_part = format!("{tabs}</aside>\n{first_tabs}</div>\n");
            
            left_part + &s + &right_part
        },
        "section" | _ => s
    };
    
    return s
}

pub fn html_builder(section: &Section, link_map: &HashMap<String, String>, title: &str) -> String {
    let mut html = String::new();
    let indent = 2;
    
    html.push_str(&get_head(title, &section.id));
    html.push_str(&unwrap_section(section, link_map, indent, "section"));
    html.push_str(&format!("{TAB}</body>\n</html>"));
    
    // println!("{html}\n\n");
    return html
}