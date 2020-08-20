use std::fs::{File};
use std::io::Write;

use std::result::Result;
use std::fmt;

use std::fs;
use std::mem::drop;

use std::path::PathBuf;


use ansi_term::Color;

#[derive(Debug)]
pub struct FileHandle {
	pub file_path: PathBuf,
	pub data: String,
}

impl Clone for FileHandle {
    fn clone(&self) -> FileHandle {
    	FileHandle {
    		file_path: self.file_path.clone(),
    		data: self.data.clone()
    	}
    }
}


impl FileHandle {
	pub fn new(path: String, data: String) -> FileHandle {
		FileHandle { 
			file_path: PathBuf::from(path),
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
		let mut f: File; 
		if !(*self.file_path).exists() {
			fs::create_dir_all((*self.file_path).parent().unwrap())?;
			f = File::create(self.file_path.clone())?;
		} else {
			println!("OPEN FILE");
			f = File::open(self.file_path.clone())?;
		}
		f.write_all(self.data.as_bytes())?;
		drop(f);
		Ok(())
	}
}

#[derive(Debug)]
pub struct Vinyl(Vec<FileHandle>);

impl Clone for Vinyl {
    fn clone(&self) -> Vinyl {
    	Vinyl(self.0.iter().map(|fh| fh.clone()).collect())
    }
}

impl Vinyl {
	pub fn new_empty() -> Vinyl {
		Vinyl(vec![])
	}

	pub fn load(files: Vec<PathBuf>) -> Result<Vinyl, VinylError> {
		Ok(Vinyl(files.into_iter().map(|f| FileHandle::load(f)).collect::<Result<Vec<_>, _>>()?))
	}

	pub fn stitch(vs: Vec<Vinyl>) -> Vinyl {
		vs.iter().fold(Vinyl::new_empty(), |acc, v| Vinyl([&acc.0[..], &v.0[..]].concat()) )
	}

	pub fn concat(&self, path: String) -> Vinyl {
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