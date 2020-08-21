
use std::{
	fmt,
	fs,
	io::Write,
	path::PathBuf
};


use ansi_term::Color;

#[derive(Debug, Clone, Default)]
pub struct FileHandle {
	pub file_path: PathBuf,
	pub data: Vec<u8>,
}

impl FileHandle {
	pub fn new(path: PathBuf, data: Vec<u8>) -> FileHandle {
		FileHandle { 
			file_path: path.clone(),
			data: data
		}
	}

	pub fn load(path: PathBuf) -> Result<FileHandle, VinylError> {
		match fs::read(path.clone()) {
			Ok(data) => Ok(FileHandle {file_path: path.clone(), data: data}),
			Err(e) => {
				let msg = format!("ERROR: {}", e);
				println!("{}", Color::Red.paint(msg));
				Err(VinylError{msg: format!("[{:?}]:{}", path.clone(), e.to_string())})
			}
		}
	}

	pub fn save(&self) -> Result<(), VinylError> {
		//println!("Saving: {:?}", &self.file_path);
		if !(*self.file_path).exists() {
			fs::create_dir_all((*self.file_path).parent().unwrap())?;
		}
	
		if self.data.len() > 0 {
			fs::OpenOptions::new()
				.read(false).write(true)
				.create(true).truncate(true)
				.open(self.file_path.clone())?
				.write(&self.data)?;
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
		let mut files_dedup = files.clone();
		files_dedup.dedup(); // dedup culls duplicates!
		Ok(Vinyl(files_dedup.into_iter().map(|f| FileHandle::load(f)).collect::<Result<Vec<_>, _>>()?))
	}

	pub fn stitch(vs: Vec<Vinyl>) -> Vinyl {
		vs.iter().fold(Vinyl::default(), |acc, v| Vinyl([&acc.0[..], &v.0[..]].concat()) )
	}

	pub fn concat(&self, path: PathBuf) -> Vinyl {
		Vinyl(vec![FileHandle::new(path, self.0.iter().fold(Vec::default(), |acc, h| [acc.as_slice(), &h.data].concat()))])
	}

	pub fn save_all(&self) -> Result<Vinyl, VinylError> {
		self.0.iter().map(|f| {
			f.save()
		}).collect::<Result<Vec<_>, _>>()?;
		Ok(self.clone())
	}

}

#[derive(Debug, Clone, Default)]
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