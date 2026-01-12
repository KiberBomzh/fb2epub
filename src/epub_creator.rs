mod html_builder;

use std::fs::{self, File};
use std::collections::HashMap;
use std::path::PathBuf;

use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ZipLibrary;
use epub_builder::Result;

use base64::{Engine as _, engine::general_purpose};


use crate::fb2_parser;
use crate::epub_creator::html_builder::html_builder;
use crate::fb2_parser::content_reader::*;


fn get_counter_str(c: usize) -> String {
    if c < 10 {
        format!("00{c}")
    } else if c < 100 {
        format!("0{c}")
    } else {
        c.to_string()
    }
}

fn unwrap_title(title: &Vec<Paragraph>) -> String {
    if title.is_empty() {
        return String::new()
    };
    
    let punctuation_chars = ['.', ',', '!', '?', '-', ';', ':', '}', ']', ')', '»'];
    let start_bracets = ['«', '(', '{', '['];
                
    
    let mut result = String::new();
    for p in title {
        let text = &if let Paragraph::Text(blocks) = p {
            let mut s = String::new();
            for (index, block) in blocks.into_iter().enumerate() {
                if block == &blocks[0] {
                    s.push_str(&block.text);
                    continue
                };
                
                
                if !punctuation_chars.iter().any(|c| block.text.starts_with(*c)) {
                    if !start_bracets.iter().any(|c| blocks[index - 1].text.ends_with(*c)) {
                        s.push(' ')
                    }
                };
                
                s.push_str(&block.text);
            };
            
            s
        } else {continue};
        
        if text.is_empty() {continue}
        
        if p != &title[0] {
            result.push_str(". ")
        };
        
        result.push_str(text)
    };
    
    return result
}

fn get_css() -> String {
r#"h1, h2, h3, h4, h5, h6 {
    text-align: center;
}

p {
    text-align: justify;
}

.refernce {
	line-height: 0.1;
	vertical-align: super;
}"#.to_string()
}

fn get_css_from_file(s_path: &PathBuf) -> std::io::Result<Vec<u8>> {
    fs::read(s_path)
}

pub fn create_epub(data: &fb2_parser::BookData, output: &PathBuf, styles_path: &Option<PathBuf>) -> Result<()> {
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    let cover_key = &data.meta.cover;
    let mut link_map: HashMap<String, String> = HashMap::new();
    
    
    // Добавление метаданных
    {let metadata = &data.meta;
    builder
        .metadata("lang", &metadata.language)?
        .metadata("title", &metadata.title)?;
    
    for author in &metadata.authors {
        builder.metadata("author", author)?;
    };
    
    if let Some(annotation) = &metadata.annotation {
        let mut description = String::new();
        for (i, p) in annotation.into_iter().enumerate() {
            if i != 0 {
                let punctuation_chars = ['.', ',', '!', '?', '-', ';', ':', '}', ']', ')', '»'];
                let start_bracets = ['«', '(', '{', '['];
                
                if !punctuation_chars.iter().any(|c| p.starts_with(*c)) {
                    if !start_bracets.iter().any(|c| annotation[i - 1].ends_with(*c)) {
                        description.push('\n')
                    }
                }
            };
            
            description.push_str(p);
        };
        builder.metadata("description", description)?;
    };
    if let Some(seq) = &metadata.sequence {
        if !seq.name.is_empty() {
            builder.add_metadata_opf(
                Box::new(epub_builder::MetadataOpf{
                    name: String::from("calibre:series"),
                    content: seq.name.clone()
                })
            );
        };
        if !seq.number.is_empty() {
            builder.add_metadata_opf(
                Box::new(epub_builder::MetadataOpf{
                    name: String::from("calibre:series_index"),
                    content: seq.number.clone()
                })
            );
        };
    };
    
    if let Some(k) = cover_key {
        if let Some(img) = &data.images.get(k) {
            let cover_name =  match &img.content_type[..] {
                "image/png" => format!("images/cover.png"),
                "image/jpeg" => format!("images/cover.jpg"),
                "image/jpg" => format!("images/cover.jpg"),
                _ => "".to_string()
            };
            
            if !cover_name.is_empty() {
                match general_purpose::STANDARD.decode(&img.binary) {
                    Ok(b) => {
                        builder.add_resource(
                            cover_name,
                            &b[..],
                            img.content_type.clone()
                        )?;
                    },
                    Err(err) => eprintln!("Image decoder error: {}", err)
                }
            }
        }
    };
    
    builder.metadata("generator", "fb2epub")?;}
    
    
    // Добавление картинок
    {let mut counter = 1;
    for (key, image) in &data.images {
        if let Some(k) = cover_key {
             if k == key { continue }
        };
        let counter_str = get_counter_str(counter);
        
        let img_name: String;
        match &image.content_type[..] {
            "image/png" => img_name = format!("images/{}.png", counter_str),
            "image/jpeg" => img_name = format!("images/{}.jpg", counter_str),
            "image/jpg" => img_name = format!("images/{}.jpg", counter_str),
            _ => continue
        };
        
        let binary = match general_purpose::STANDARD.decode(&image.binary) {
            Ok(b) => b,
            Err(err) => {
                eprintln!("Image decoder error: {}", err);
                continue
            }
        };
        
        builder
            .add_resource(
                &img_name,
                &binary[..],
                image.content_type.clone()
            )?;
        
        link_map.insert(key.to_string(), format!("../{img_name}"));
        counter += 1;
    }};
    
    
    // Добавление текстовых документов
    let mut counter = 0;
    for section in &data.content {
        let counter_str = get_counter_str(counter + 1);
        let prefix = "text/".to_string();
        let suffix = ".xhtml";
        let file_name = &if let Some(id) = &section.id {
            if id == "NOTES" {
                "notes".to_string()
            } else {
                counter += 1;
                format!("section_{counter_str}")
            }
        } else {
            counter += 1;
            format!("section_{counter_str}")
        };
        
        let title = unwrap_title(&section.title);
        let level: i32 = (section.level + 1).into();
        let html_content = html_builder(&section, &link_map, &title);
        if title.is_empty() {
            builder.add_content(EpubContent::new(prefix + file_name + suffix, html_content.as_bytes()))?;
        } else {
            builder.add_content(
                EpubContent::new(prefix + file_name + suffix, html_content.as_bytes())
                    .title(title)
                    .level(level)
            )?;
        }
        
    };
    
    // Добавление стилей
    if let Some(s_path) = styles_path {
        builder.stylesheet(&get_css_from_file(s_path)?[..])?;
    } else {
        builder.stylesheet(get_css().as_bytes())?;
    };
    
    
    let mut new_book = File::create(output)?;
    builder.generate(&mut new_book)?;
    
    #[cfg(target_os = "windows")]
    println!("Saved to {:#?}", output);
    
    Ok(())
}
