use std::{env::{self, Args}, fs, io, path::{Path, PathBuf}};

pub fn transfer(config: Config) -> Result<(), String>{
    let download_dir = format!("{}/Downloads", config.home);
    let calibre_dir = format!("{}/Calibre Library", config.home);
    let digital_editions_dir = format!("{}/Documents/Digital Editions", config.home);
    let epub_extension = "epub";
    let acsm_extension = "acsm";

    // Getting all .epub files in Downloads dir
    let mut mv_files = match search_dir(&download_dir, epub_extension) {
        Err(err) => {
            eprintln!("Error searching downloads dir for epub files {err:?}");
            Vec::new()
        }
        Ok(files) => {
            println!("Found epub files in Downloads folder: {files:?}");
            files
        }
    };

    // Adding .epub file in Calibre Library to files
    match search_subdirs(calibre_dir, epub_extension) {
        None => println!("No epub files found in Calibre Library"),
        Some(calibre_files) => {
            println!("Found epub files in Calibre Library: {calibre_files:?}");
            mv_files.extend(calibre_files);
        }
    }

    // Moving files to kobo and collecting errors
    let mut errors: Vec<String> = mv_files.iter()
        .filter_map(|file| {
            let Some(filename) = file.file_name() else {
                return None
            };

            let Some(filename) = filename.to_str() else {
                return None
            };
            fs::rename(file, format!("{}/{}", config.dest, filename)).err()
        })
        .map(|error| { error.to_string() })
        .collect();
    
    // Getting all .acsm files in Downloads dir to remove
    let mut rm_files = match search_dir(&download_dir, acsm_extension) {
        Err(err) => {
            eprintln!("Error searching downloads dir for acsm files {err:?}");
            Vec::new()
        }   
        Ok(files) => {
            println!("Found acsm files in downloads dir {files:?}");
            files
        }
    };

    // Adding .epub files in Digital Editions Library to remove files
    match search_dir(&digital_editions_dir, epub_extension) {
        Err(err) => errors.push(err.to_string()),
        Ok(search_results) => rm_files.extend(search_results),
    }

    // Adding errors from removing acsm files
    errors.extend(
        rm_files.iter()
        .filter_map(|file|fs::remove_file(file).err())
        .map(|err| err.to_string())
    );

    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors.join("\n"))
    }
}

fn search_dir(dir: &str, extension: &str) -> Result<Vec<PathBuf>, io::Error> {
    let dir = fs::read_dir(dir)?;
    Ok(dir
        .filter_map(|file| file.ok())
        .map(|file| file.path())
        .filter(|file| file.is_file())
        .filter(|file| {
            let Some(ext) = file.extension() else {
                return false
            };
            ext == extension})
        .collect::<Vec<PathBuf>>())
}

fn search_subdirs<P: AsRef<Path>>(dir: P, extension: &str) -> Option<Vec<PathBuf>> {
    let Ok(dir) = fs::read_dir(dir) else {
        return None;
    };

    let (mut files, dirs) = dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .fold((Vec::new(), Vec::new()), |(mut files, mut dirs), path| {
            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() {
                let Some(ext) = path.extension() else {
                    return (files, dirs)
                };
                if ext == extension {
                    files.push(path);
                }
            }
            (files, dirs)
        });
    
    dirs.iter()
        .filter_map(|dir| search_subdirs(dir, extension))
        .for_each(|dir| files.extend(dir));

    match !files.is_empty() {
        true => Some(files),
        false => None
    }
}

pub fn help() {
    println!("Usage: kobo-transfer [options...] <destination>");
}
pub struct Config {
    dest: String,
    home: String
}

impl Config {
    /// Returns a Config with the flags set on the cmdline
    /// 
    /// # Panics
    /// 
    /// Panics if there is not atleast one additional arg
    ///
    pub fn build(mut args: Args) -> Option<Config> {
        let home = match env::var("HOME") {
            Err(_) => return None,
            Ok(home) => home
        };
        let mut dest = None;
        while let Some(arg) = args.next() {
            match arg {
                // Add other options here..
                other => dest = Some(other)
            }
        }

        match dest {
            Some(dest) => Some(Config { dest, home }),
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_subdirs_some() {  
        let Ok(home) = env::var("HOME") else {
            panic!("Can't get HOME env var");
        };

        let Some(mut files) = search_subdirs(&format!("{}/code/kobo-transfer", home), "epub") else {
            panic!("Files should be some, but got none")
        };

        assert_eq!(
            files.sort(),
            vec![
                PathBuf::from(format!("{}/code/kobo-transfer/tests/fixtures/recipe.epub", home)),
                PathBuf::from(format!("{}/code/kobo-transfer/tests/fixtures/wow.epub", home)),
                PathBuf::from(format!("{}/code/kobo-transfer/tests/fixtures/grub.epub", home)),
            ].sort());
    }

    #[test]
    fn test_search_subdirs_none() {
        let Ok(home) = env::var("HOME") else {
            panic!("Can't get HOME env var");
        };
         
        let files = search_subdirs(format!("{}/code/kobo-transfer/src", home), "epub");
        assert!(files.is_none())
    }

}
