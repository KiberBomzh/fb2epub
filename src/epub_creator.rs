mod html_builder;

use std::fs::File;
use std::collections::HashMap;

use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ZipLibrary;
use epub_builder::Result;

use base64::{Engine as _, engine::general_purpose};


use crate::fb2_parser;
use crate::epub_creator::html_builder::html_builder;


pub fn create_epub(data: &fb2_parser::BookData) -> Result<()> {
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
        for p in annotation {
            builder.add_description(p);
        };
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
        
        let img_name: String;
        match &image.content_type[..] {
            "image/png" => img_name = format!("images/{}.png", counter),
            "image/jpeg" => img_name = format!("images/{}.jpg", counter),
            "image/jpg" => img_name = format!("images/{}.jpg", counter),
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
    
    
    // level = 0 - это coverpage
    
    // Добавление текстовых документов
    let mut counter = 0;
    for section in &data.content {
        let file_name = if let Some(id) = &section.id {
            if id == "NOTES" {
                format!("text/notes.xhtml")
            } else {
                counter += 1;
                format!("text/Section_{counter}.xhtml")
            }
        } else {
            counter += 1;
            format!("text/Section_{counter}.xhtml")
        };
        
        let title = if section.title.is_empty() {
            format!("Section_{counter}")
        } else if section.title.len() > 1 {
            let mut s = String::new();
            for line in &section.title {
                s.push_str(line);
                if *line != section.title[section.title.len() - 1] {
                    s.push(' ')
                };
            };
            
            s
        } else {
            section.title[0].clone()
        };
        let level: i32 = (section.level + 1).into();
        let html_content = html_builder(&section, &link_map, &title);
        builder.add_content(
            EpubContent::new(&file_name, html_content.as_bytes())
                .title(title)
                .level(level)
        )?;
    };
    
    
    let mut new_book = File::create("new_book.epub")?;
    builder.generate(&mut new_book)?;
 
    Ok(())
}