mod fb2_parser;
mod epub_creator;
mod zip_reader;

use std::path::{PathBuf, Path};
use std::fs;


/*
// Функция для вывода секций, удобно для дебага
fn print_sections(sections: &Vec<crate::fb2_parser::Section>, without_p: bool) {
    let mut s = String::new();
    let mut is_first = true;
    for section in sections {
        if is_first {
            is_first = false;
        } else {
            std::io::stdin().read_line(&mut s).unwrap();
            match s.trim() {
                "q" | "quit" => break,
                _ => {}
            };
            s.clear();
        }
        std::process::Command::new("clear").status().unwrap();
        if without_p {
            dbg!(&section.level);
            dbg!(&section.file_name);
            dbg!(&section.id);
            dbg!(&section.title);
        } else {
            dbg!(&section);
        };
    };
}
*/

fn get_free_output(output: &Path) -> Option<PathBuf> {
    let mut file_name = output.file_stem()?.to_str()?;
    
    if file_name.ends_with(".fb2") {
        if let Some(r_index) = file_name.rfind(".") {
            file_name = &file_name[..r_index]
        }
    };
    
    let parent = output.parent()?;
    let mut free_output = parent.join(format!("{file_name}.epub"));
    
    let mut counter = 1;
    while free_output.exists() {
        free_output = parent.join(format!("{file_name}-{counter}.epub"));
        counter += 1;
    };
    
    return Some(free_output.to_owned())
}


/// Main function, takes path to fb2 book (or zip archive), returns path to new epub book.
///
/// If replace = true input fb2 book will be deleted.
///
/// styles_path is path to custom stylesheet, for default styles use None.
pub fn run(book: &Path, 
        output: &Path, 
        replace: bool, 
        styles_path: Option<&Path>) -> Result<PathBuf, Box<dyn std::error::Error>> {

    if book.extension().and_then(|s| Some(s.to_str()?.to_lowercase())) == Some("zip".to_string()) {
        match crate::zip_reader::convert_archive(book, output, styles_path) {
            Ok(o) if replace => {
                fs::remove_file(book)?;
                return Ok(o)
            },
            Ok(o) => return Ok(o),
            Err(err) => return Err(err)
        }
    };

    // Чтение входного FB2
    let mut data = fb2_parser::get_data(book)?;
    // print_sections(&data.content, true);
    
    
    // Проверка имени файла
    if let Some(p) = output.parent() {
        if !p.exists() {
            fs::create_dir_all(p)?
        }
    };
    
    let output =  &if let Some(o) = get_free_output(output) {o}
    else {output.to_owned()};
    
    
    // Создание EPUB
    match epub_creator::create_epub(&mut data, &output, styles_path) {
        Ok(o) if replace => {
            fs::remove_file(book)?;
            return Ok(o)
        },
        Ok(o) => return Ok(o),
        Err(err) => Err(format!("Error while creating Epub: {}!", err).into())
    }
}
