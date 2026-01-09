mod fb2_parser;
mod epub_creator;

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

pub fn run(book: &PathBuf, output: &PathBuf, replace: bool) {
    let data = fb2_parser::get_data(&book);
    if let Err(err) = epub_creator::create_epub(&data, output) {
        eprintln!("Error while creating Epub: {}!", err)
    } else if replace {
        if let Err(er) = fs::remove_file(book) {
            eprintln!("Error while deleting {:#?}: {}", book, er)
        }
    };
    
    // print_sections(&data.content, false);
}
