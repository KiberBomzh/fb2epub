mod fb2_parser;
mod epub_creator;
mod zip_reader;

use std::path::PathBuf;
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
            dbg!(&section.title);
        } else {
            dbg!(&section);
        };
    };
}
*/

fn get_free_output(output: &PathBuf) -> Option<PathBuf> {
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
    
    return Some(free_output.to_path_buf())
}


pub fn run(book: &PathBuf, output: &PathBuf, replace: bool) -> Result<(), Box<dyn std::error::Error>> {
    if book.extension().and_then(|s| Some(s.to_str()?.to_lowercase())) == Some("zip".to_string()) {
        crate::zip_reader::convert_archive(book, &output)?;
        if replace {fs::remove_file(book)?}
        return Ok(())
    };

    // Чтение входного FB2
    let data = fb2_parser::get_data(&book)?;
    
    
    // Проверка имени файла
    if let Some(p) = output.parent() {
        if !p.exists() {
            fs::create_dir_all(p)?
        }
    };
    
    let mut output = output.clone();
    if let Some(o) = get_free_output(&output) {
        output = o;
    };
    
    
    // Создание EPUB
    if let Err(err) = epub_creator::create_epub(&data, &output) {
        return Err(format!("Error while creating Epub: {}!", err).into())
    } else if replace {fs::remove_file(book)?};


    // print_sections(&data.content, false);
    return Ok(())
}
