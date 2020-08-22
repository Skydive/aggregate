
use std::{
	fmt,
	fs,
	io::Write,
	path::{Path, PathBuf}
};


use crc::crc32;
use ansi_term::Color;

#[derive(Debug, Clone, Default)]
pub struct FileHandle {
	pub out_path: PathBuf,
	pub rel_path: PathBuf,
	pub data: Vec<u8>,
}

impl FileHandle {
	pub fn new(out_path: PathBuf, rel_path: PathBuf, data: Vec<u8>) -> FileHandle {
		FileHandle { 
			out_path: out_path.clone(),
			rel_path: rel_path.clone(),
			data: data
		}
	}

	pub fn file_path(&self) -> PathBuf {
		Path::new(&self.out_path).join(&self.rel_path).to_path_buf()
	}

	pub fn load(out_path: PathBuf, rel_path: PathBuf) -> Result<FileHandle, VinylError> {
		let path = Path::new(&out_path).join(&rel_path).to_path_buf();
		match fs::read(&path) {
			Ok(data) => Ok(FileHandle {out_path: out_path.clone(), rel_path: rel_path.clone(), data: data}),
			Err(e) => {
				let msg = format!("ERROR: {}", e);
				println!("{}", Color::Red.paint(msg));
				Err(VinylError{msg: format!("[{:?}]:{}", path, e.to_string())})
			}
		}
	}

	pub fn save(&self) -> Result<(), VinylError> {
		//println!("Saving: {:?}", &self.file_path);
		if !(*self.file_path()).exists() {
			fs::create_dir_all((*self.file_path()).parent().unwrap())?;
		}
	
		if self.data.len() > 0 {
			fs::OpenOptions::new()
				.read(false).write(true)
				.create(true).truncate(true)
				.open(self.file_path())?
				.write(&self.data)?;
			Ok(())
		} else {
			Err(VinylError{msg: "Data corrupted/empty".to_string()})
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Vinyl{
	handles: Vec<FileHandle>,
	pub revision_pairs: Vec<(String, String)>
}

impl Vinyl {
	pub fn new(handles: Vec<FileHandle>) -> Vinyl {
		Vinyl {
			handles: handles,
			..Default::default()
		}
	}
	pub fn load(out_path: PathBuf, files: Vec<PathBuf>) -> Result<Vinyl, VinylError> {
		let mut files_dedup = files.clone();
		files_dedup.dedup(); // dedup culls duplicates!
		Ok(Vinyl::new(files_dedup.into_iter().map(|f| FileHandle::load(out_path.clone(), f)).collect::<Result<Vec<_>, _>>()?))
	}

	pub fn concat(self, out_path: PathBuf, rel_path: PathBuf) -> Vinyl {
		Vinyl::new(vec![FileHandle::new(out_path, rel_path, self.handles.iter().fold(Vec::default(), |acc, h| [acc.as_slice(), &h.data].concat()))])
	}

	pub fn flatten(vs: Vec<Vinyl>) -> Vinyl {
		vs.iter().fold(Vinyl::default(), |acc, v| Vinyl { 
			handles: [&acc.handles[..], &v.handles[..]].concat(), 
			revision_pairs: [&acc.revision_pairs[..], &v.revision_pairs[..]].concat()
		})
	}

	pub fn insert(mut self, f: FileHandle) -> Vinyl {
		self.handles.push(f);
		self
	}

	pub fn revisions(mut self) -> Vinyl {
		let mut rev_pairs = Vec::default();
		self.handles.iter_mut().for_each(|f| {
			let old_path = &f.rel_path;
			let file_name = old_path.file_stem().unwrap().to_str().unwrap().to_string();
			let new_name = format!("{}-{:x}.{}", 
				file_name,
				crc32::checksum_ieee(&f.data),
				old_path.extension().unwrap().to_str().unwrap().to_string()
			);
			let new_path = f.rel_path.with_file_name(&new_name).to_path_buf();
			rev_pairs.push((old_path.to_str().unwrap().to_string(), new_path.to_str().unwrap().to_string()));
			f.rel_path = new_path;
		});
		self.revision_pairs.append(&mut rev_pairs);
		self
	}

	pub fn save_all(self) -> Result<Vinyl, VinylError> {
		self.handles.iter().map(|f| {
			f.save()
		}).collect::<Result<Vec<_>, _>>()?;
		Ok(self.clone())
	}

	// pub fn save_manifest(self) -> Result<Vinyl, VinylError> {
	// 	// LOCK!?
	// }
}

impl fmt::Display for Vinyl {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		//write!(f, "Vinyl: {}", self.handles.iter().map(|v| v.file_path.to_str().unwrap().to_string()).collect::<Vec<String>>().join(", "))
		write!(f, "Vinyl: {:?}", self.revision_pairs)
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