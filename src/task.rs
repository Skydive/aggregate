
use std::fs::{File};
use std::io::Write;
use std::fs;
use std::mem::drop;

use std::path::PathBuf;

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

	pub fn load(path: String) -> Option<FileHandle> {
		let path_buf = PathBuf::from(path);
		fs::read_to_string(path_buf.clone()).ok().map(|data| FileHandle { 
			file_path: path_buf,
			data: data
		})
	}

	pub fn save(&self) {
		let mut f: Option<File>; 
		if !(*self.file_path).exists() {
			fs::create_dir_all((*self.file_path).parent().unwrap());
			f = File::create(self.file_path.clone()).ok();
		} else {
			f = File::open(self.file_path.clone()).ok();
		}
		if let Some(h) = f.as_mut() {
			h.write_all(self.data.as_bytes());
			drop(h);
		}
	}
}

pub struct Vinyl(Vec<FileHandle>);
impl Clone for Vinyl {
    fn clone(&self) -> Vinyl {
    	Vinyl(self.0.iter().map(|fh| fh.clone()).collect())
    }
}

impl Vinyl {
	pub fn load(files: Vec<String>) -> Vinyl {
		Vinyl(files.into_iter().filter_map(|f| FileHandle::load(f)).collect())
	}

	pub fn concat(&self, path: String) -> Vinyl {
		Vinyl(vec![FileHandle::new(path,  self.0.iter().fold(String::from(""), |acc, h| acc+&h.data))])
	}

	pub fn save_all(&self) -> Vinyl {
		self.0.iter().for_each(|f| f.save());
		self.clone()
	}
}


pub struct ProcessTask {
	pub name: String, 
	pub fnc: fn() -> (),
}