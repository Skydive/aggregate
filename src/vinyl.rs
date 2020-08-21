use std::fs::{File};
use std::io::Write;

use std::result::Result;
use std::fmt;

use std::fs;
use std::path::PathBuf;


use ansi_term::Color;

#[derive(Debug, Clone, Default)]
pub struct FileHandle {
	pub file_path: PathBuf,
	pub data: String,
}

impl FileHandle {
	pub fn new(path: PathBuf, data: String) -> FileHandle {
		FileHandle { 
			file_path: path.clone(),
			data: data
		}
	}

	pub fn load(path: PathBuf) -> Result<FileHandle, VinylError> {
		match fs::read_to_string(path.clone()) {
			Ok(data) => Ok(FileHandle {file_path: path.clone(), data: data}),
			Err(e) => {
				let msg = format!("ERROR: {}", e);
				println!("{}", Color::Red.paint(msg));
				Err(VinylError{msg: e.to_string()})
			}
		}
	}

	pub fn save(&self) -> Result<(), VinylError> {
		if !(*self.file_path).exists() {
			fs::create_dir_all((*self.file_path).parent().unwrap())?;
		}
	
		if self.data.trim().len() > 0 {
			fs::OpenOptions::new()
				.read(false).write(true)
				.create(true).truncate(true)
				.open(self.file_path.clone())?
				.write(self.data.as_bytes())?;
			Ok(())
		} else {
			Err(VinylError{msg: "Data corrupted/empty".to_string()})
		}

	}
}

#[derive(Debug, Clone, Default)]
pub struct Vinyl(Vec<FileHandle>);

impl Vinyl {
	pub fn load(files: Vec<PathBuf>) -> Result<Vinyl, VinylError> {
		Ok(Vinyl(files.into_iter().map(|f| FileHandle::load(f)).collect::<Result<Vec<_>, _>>()?))
	}

	pub fn stitch(vs: Vec<Vinyl>) -> Vinyl {
		vs.iter().fold(Vinyl::default(), |acc, v| Vinyl([&acc.0[..], &v.0[..]].concat()) )
	}

	pub fn concat(&self, path: PathBuf) -> Vinyl {
		Vinyl(vec![FileHandle::new(path, self.0.iter().fold(String::from(""), |acc, h| acc+&h.data))])
	}

	pub fn save_all(&self) -> Result<Vinyl, VinylError> {
		self.0.iter().map(|f| {
			f.save()
		}).collect::<Result<Vec<_>, _>>()?;
		Ok(self.clone())
	}

}

pub struct VinylError {
	pub msg: String
}

impl From<std::io::Error> for VinylError {
    fn from(error: std::io::Error) -> Self {
        VinylError {
            msg: error.to_string(),
        }
    }
}

impl fmt::Display for VinylError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "VinylError: {}", self.msg)
    }
}