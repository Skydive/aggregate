use serde_json::Value;
use serde::{Deserialize, Serialize};

use std::vec::Vec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::aggregate::{Aggregate, TaskGraph, TaskIndex};
use crate::config::{ConfigModule, ConfigMeta};
use crate::vinyl::Vinyl;

use super::GenerateGraphs;

#[derive(Serialize, Deserialize)]
pub struct OptionsJS {
	pub prefix: String,
	pub dest: String
}
pub type ContentJS = HashMap<String, Vec<String>>;

#[derive(Debug)]
struct ProcessorJS();

macro_rules! clone {
    ($i:ident) => (let $i = $i.clone();)
}

macro_rules! clone_all {
    ($($i:ident),+) => {
        $(clone!($i);)+
    }
}

impl GenerateGraphs for ProcessorJS {
	fn generate_graphs(&self, mut g: &mut TaskGraph, meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (TaskIndex, TaskIndex) {
		//let conf_js: ConfigModule<OptionsJS, ContentJS> = serde_json::from_str(&serde_json::to_string(m).unwrap()).unwrap();
		let conf_js: ConfigModule<OptionsJS, ContentJS> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};

		let build_name = format!("build:{}", conf_js.name);
		let deploy_name = format!("deploy:{}", conf_js.name);
		let mut build_nodes = Vec::default();
		let mut deploy_nodes = Vec::default();
		for (k, v) in &conf_js.content {
			//println!("{}, {:?}", k, v);

			let sub_build_name = format!("{}:{}", build_name, k);
			let sub_deploy_name = format!("{}:{}", deploy_name, k);

			let build_file_path = Path::new(&meta.base_path)
				.join(&meta.build_path)
				.join(&conf_js.options.dest)
				.join(&format!("{}.js", k)).to_path_buf();
			let deploy_file_path = Path::new(&meta.base_path)
				.join(&meta.deploy_path)
				.join(&conf_js.options.dest)
				.join(&format!("{}.js", k)).to_path_buf();

			let paths = v.iter().map(|f|
				Path::new(&meta.base_path)
				.join(&meta.src_path)
				.join(&conf_js.options.prefix)
				.join(&k)
				.join(&f).to_path_buf()
			).collect::<Vec<PathBuf>>();
			let wild_paths = super::path_wildcards(paths.clone());

			let (cap_file, cap_paths) = (build_file_path.clone(), wild_paths.clone());
			build_nodes.push(Aggregate::chain(&mut g, sub_build_name.clone(), Box::new(move |_v| {
				Vinyl::load(cap_paths.clone())?
					.concat(cap_file.clone())
					.save_all()
			}), vec![], false));

			// let (cap_file, cap_paths) = (deploy_file_path.clone(), wild_paths.clone());
			// deploy_nodes.push(Aggregate::chain(&mut g, sub_deploy_name.clone(), Box::new(move |_v| {
			// 	Vinyl::load(cap_paths.clone())?
			// 		.concat(cap_file.clone())
			// 		.save_all()
			// }), vec![], false));

			//println!("{:?} {:?}", build_name.clone(), build_file_path.clone());
		}

		(
			Aggregate::chain(&mut g, build_name.clone(), Box::new(move |_v| Ok(Vinyl::default())), build_nodes, false), 
			Aggregate::chain(&mut g, deploy_name.clone(), Box::new(move |_v| Ok(Vinyl::default())), deploy_nodes, false)
		)
	}
}

pub fn new_processor() -> Box<dyn GenerateGraphs + Sync> { Box::new(ProcessorJS()) }