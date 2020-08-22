
use super::aggregate::{TaskGraph, TaskIndex};

use super::config::{ConfigModule, ConfigMeta};

use serde_json::value::Value;

pub mod concat;
pub mod clone;
pub mod htmlpages;

use std::collections::HashMap;
use std::path::PathBuf;
use glob::glob;



lazy_static! {
	pub static ref PROCESSOR_MAP: HashMap<&'static str, Box<dyn Sync + GenerateGraphs>> = {
		let mut map = HashMap::new();
		map.insert("concat", concat::new_processor());
		map.insert("clone", clone::new_processor());
		map.insert("htmlpages", htmlpages::new_processor());
		map
	};
}

pub trait GenerateGraphs {
	fn generate_graphs(&self, g: &mut TaskGraph, cfg_meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (TaskIndex, TaskIndex);
	fn watcher_dirs_and_tasks(&self, cfg_meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (Vec<String>, Vec<String>);
}

pub fn path_wildcards(paths: Vec<PathBuf>) -> Vec<PathBuf> {
	paths.iter().flat_map(|pth| {
		glob(pth.to_str().unwrap()).unwrap().filter_map(|x| x.ok()).collect::<Vec<PathBuf>>()
	}).collect::<Vec<PathBuf>>()
}