mod html_builder;

use std::fs::File;

use epub_builder::EpubBuilder;
use epub_builder::ZipLibrary;
use epub_builder::Result;

use base64::{Engine as _, engine::general_purpose};


use crate::fb2_parser;
use crate::epub_creator::html_builder::html_builder;


pub fn create_epub(data: &fb2_parser::BookData) -> Result<()> {
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    
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
    
    builder.metadata("generator", "fb2epub")?;}
    
    
    // Добавление текстовых документоа
    // for section in &data.content {
    //     let html_content = html_builder(&section);
        
    // };
    
    
    // Добавление картинок
    {let mut counter = 1;
    for (_, image) in &data.images {
        let img_name: String;
        match &image.content_type[..] {
            "image/png" => img_name = format!("image_{}.png", counter),
            "image/jpeg" => img_name = format!("image_{}.jpg", counter),
            _ => continue
        }
        let binary = general_purpose::STANDARD.decode(&image.binary).unwrap();
        
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