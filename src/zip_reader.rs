use std::fs::File;
use std::path::{PathBuf, Path};
use std::io;

use tempfile::TempDir;


fn extract_books(path: &PathBuf, temp_path: &Path) -> zip::result::ZipResult<Vec<PathBuf>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut files: Vec<PathBuf> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.is_dir() {continue}

        if let Some(name) = file.enclosed_name() {
            if name.extension().and_then(|s| Some(s.to_str()?.to_lowercase())) == Some("fb2".to_string()) {
                let outpath: PathBuf = temp_path.to_owned().join(
                    if let Some(n) = name.file_name() {n}
                    else {continue}
                );
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
                files.push(outpath);
            };
        };
    };

    return Ok(files)
    
}

pub fn convert_archive(path: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let files = extract_books(path, temp_path)?;
    if files.is_empty() {
        return Err(format!("Nothing to convert in {:#?}", path).into())
    }

    if files.len() == 1 {
        crate::run(&files[0], output, false)?;
        return Ok(())
    };

    let mut parent = output.parent()
            .ok_or(format!("Cannot get parent folder for: {:#?}", path))?
            .to_path_buf();
    
    if !output.exists() {
        let out_folder_name = output.file_name()
            .and_then(|n| n.to_str())
                .ok_or(format!("Cannot get output folder for: {:#?}", path))?;
        
        parent = if let Some(r_index) = out_folder_name.rfind(".epub") {
            parent.join(format!("{}_out", &out_folder_name[..r_index]))
        } else {output.to_path_buf()}
    };
    
    for file in &files {
        let file_name = if let Some(name) = file
            .file_stem().and_then(|os| os.to_str()) {
                name.to_string() + ".epub"
        } else {continue};
        let file_output = parent.join(file_name);
        crate::run(file, &file_output, false)?;
    };

    Ok(())
}
