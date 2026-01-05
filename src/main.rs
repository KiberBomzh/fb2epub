// Здесь будет только то что нужно для запуска в cli
// Остальное в lib.rs для возможности использования проекта как библиотеки

use clap::{Command, Arg};


fn main() {
	let matches = Command::new("fb2epub")
	    .version(env!("CARGO_PKG_VERSION"))
        .author("KiberBomzh")
        .about("Converter fb2 to epub")
        .arg(
            Arg::new("input")
                .help("A book in fb2")
                .required(true)
                .index(1)
        )
        .get_matches();
        
    match matches.get_one::<String>("input") {
        Some(input) => fb2epub::start(&input),
        None => println!("{}", "Must be at least one input!")
    }
}
