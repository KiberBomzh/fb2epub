mod html_builder;

use std::fs::File;

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
    
    
    // level = 0 - это coverpage
    
    // Добавление текстовых документоа
    let mut counter = 1;
    for section in &data.content {
        let html_content = html_builder(&section);
        let file_name = format!("text/Section_{counter}.xhtml");
        builder.add_content(
            EpubContent::new(&file_name, html_content.as_bytes())
        )?;
        
        counter += 1;
    };
    
    
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
                img_name,
                &binary[..],
                image.content_type.clone()
            )?;
        
        counter += 1;
    }};
    
    
    let mut new_book = File::create("new_book.epub")?;
    builder.generate(&mut new_book)?;
 
    Ok(())
}