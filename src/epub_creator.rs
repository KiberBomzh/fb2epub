use std::fs::File;

use epub_builder::EpubBuilder;
use epub_builder::ZipLibrary;
use epub_builder::Result;

use crate::fb2_parser;


pub fn create_epub(metadata: &fb2_parser::Metadata) -> Result<()> {
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    
    // Добавление метаданных
    if let Some(title) = &metadata.title {
        builder.metadata("title", title)?;
    };
    if let Some(authors) = &metadata.authors {
        for author in authors {
            builder.metadata("author", author)?;
        }
    };
    if let Some(annotation) = &metadata.annotation {
        for p in annotation {
            builder.add_description(p);
        };
    };
    if let Some(lang) = &metadata.language {
        builder.metadata("lang", lang)?;
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
    
    builder.metadata("generator", "fb2epub")?;
    
    
    let mut new_book = File::create("new_book.epub")?;
    builder.generate(&mut new_book)?;
 
    Ok(())
}