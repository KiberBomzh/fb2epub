mod fb2_parser;

use std::path::PathBuf;


pub fn start(book: &str) {
    let path = PathBuf::from(book);
    
    // Проверки
    if !path.exists() {
        panic!("There's no such path: {:?}!", path)
    } else if path.extension().unwrap() != "fb2" {
        panic!("The file {:?} isn't fb2!", path)
    };
    
    
    fb2_parser::get_data(&path);
}
