use crate::fb2_parser::Section;


fn get_head(head_title: &str) -> String {
    format!(r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en" xmlns:epub="http://www.idpf.org/2007/ops">
    <head>
        <title>{head_title}</title>
        <link href="stylesheet.css" rel="stylesheet" type="text/css"/>
    </head>
    <body>{}"#, "\n")
}

pub fn html_builder(section: &Section) -> String {
    let end = format!("    </body>\n</html>");
    get_head("Title") + &end
}