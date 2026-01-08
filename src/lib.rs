mod fb2_parser;
mod epub_creator;

use std::path::PathBuf;


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

pub fn start(book: &str) {
    let path = PathBuf::from(book);
    
    // Проверки
    if !path.exists() {
        panic!("There's no such path: {:?}!", path)
    } else if path.extension().unwrap() != "fb2" {
        panic!("The file {:?} isn't fb2!", path)
    };
    
    
    let data = fb2_parser::get_data(&path);
    epub_creator::create_epub(&data).expect("Error while creating Epub!");
    // print_sections(&data.content, false);
}
