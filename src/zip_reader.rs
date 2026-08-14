use std::fs::{self, File};
use std::path::{PathBuf, Path};
use std::io;

use tempfile::TempDir;


fn extract_books(path: &Path, temp_path: &Path) -> zip::result::ZipResult<Vec<PathBuf>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut files: Vec<PathBuf> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_dir() {continue}

        if let Some(name) = file.enclosed_name()
        &&  name.extension().map(|s| s.to_string_lossy().to_lowercase()) == Some("fb2".to_string()) {
            let outpath: PathBuf = temp_path.to_owned().join(
                if let Some(n) = name.file_name() {n}
                else {continue}
            );
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            files.push(outpath);
        };
    };

    
    Ok(files)
    
}

pub fn convert_archive(
    path: &Path,
    output: &Path,
    styles_path: Option<&Path>,
    metadata: Option<crate::Metadata>,
    suspend_error_messages: bool,
    debug: bool
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let files = extract_books(path, temp_path)?;
    if files.is_empty() {
        return Err(format!("Nothing to convert in {:#?}", path).into())
    }

    if files.len() == 1 {
        return crate::run(
            &files[0],
            output,
            false,
            styles_path,
            metadata,
            suspend_error_messages,
            debug
        );
    };

    if output.is_file() {
        fs::remove_file(output)?;
    }
    if !output.exists() {
        fs::create_dir_all(output)?;
    }
    
    for file in &files {
        let file_name = 
            if let Some(name) = file
                .with_extension("epub")
                .file_name()
                .and_then(|s| s.to_str() ) { name.to_string() }
                else {continue};

        let file_output = output.join(file_name);
        crate::run(
            file,
            &file_output,
            false,
            styles_path,
            metadata.clone(),
            suspend_error_messages,
            debug
        )?;
    };


    Ok(output.to_path_buf())
}
