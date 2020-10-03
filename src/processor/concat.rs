use serde_json::Value;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use std::vec::Vec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::aggregate::{Aggregate, TaskGraph, TaskIndex};
use crate::config::{ConfigModule, ConfigMeta};
use crate::vinyl::Vinyl;

use super::GenerateGraphs;

#[derive(Serialize, Deserialize)]
pub struct OptionsConcat {
	pub prefix: String,
	pub dest: String,
	pub ext: String,
	#[serde(default)]
	pub revision: bool
}
pub type ContentConcat = HashMap<String, Vec<String>>;

#[derive(Debug)]
struct ProcessorConcat();

macro_rules! clone {
    ($i:ident) => (let $i = $i.clone();)
}

macro_rules! clone_all {
    ($($i:ident),+) => {
        $(clone!($i);)+
    }
}

impl GenerateGraphs for ProcessorConcat {
	fn generate_graphs(&self, mut g: &mut TaskGraph, meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (TaskIndex, TaskIndex) {
		//let conf_js: ConfigModule<OptionsJS, ContentJS> = serde_json::from_str(&serde_json::to_string(m).unwrap()).unwrap();
		let conf_concat: ConfigModule<OptionsConcat, ContentConcat> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};

		let rev = conf_concat.options.revision;
		let build_name = format!("build:{}", conf_concat.name);
		let deploy_name = format!("deploy:{}", conf_concat.name);
		let mut build_nodes = Vec::default();
		let mut deploy_nodes = Vec::default();
		for (k, v) in &conf_concat.content {
			//println!("{}, {:?}", k, v);

			let sub_build_name = format!("{}:{}", build_name, k);
			let sub_deploy_name = format!("{}:{}", deploy_name, k);

			let out_path_build = Path::new(&meta.base_path).join(&meta.build_path).to_path_buf();
			let out_path_deploy = Path::new(&meta.base_path).join(&meta.deploy_path).to_path_buf();

			let src_path = Path::new(&meta.base_path)
				.join(&meta.src_path).to_path_buf();

			let path = v.iter().map(|p| (&src_path).join(&conf_concat.options.prefix).join(&k).join(&p).to_path_buf() ).collect::<Vec<_>>();
			let wild_paths = super::path_wildcards(path.clone());
			let wild_paths_post = wild_paths.iter().filter_map(|p| p.strip_prefix(&src_path).ok()).map(|p| p.to_path_buf()).collect::<Vec<_>>();

			let out_rel_path = Path::new(&conf_concat.options.dest)
				.join(&format!("{}.{}", k, conf_concat.options.ext)).to_path_buf();

			build_nodes.push(Aggregate::chain(&mut g, sub_build_name.clone(), Arc::new({
				clone_all!(src_path, out_path_build, out_rel_path, wild_paths_post);
				move |_v| {
					Vinyl::load(src_path.clone(), wild_paths_post.clone())?
						.concat(out_path_build.clone(), out_rel_path.clone())
						.save_all()
				}
			}), vec![], false));


			deploy_nodes.push(Aggregate::chain(&mut g, sub_deploy_name.clone(), Arc::new({
				clone_all!(src_path, out_path_deploy, out_rel_path, wild_paths_post);
				move |_v| {
					if rev { 
						Ok(Vinyl::load(src_path.clone(), wild_paths_post.clone())?
						.concat(out_path_deploy.clone(), out_rel_path.clone()))
					}
					else {
						Vinyl::load(src_path.clone(), wild_paths_post.clone())?
						.concat(out_path_build.clone(), out_rel_path.clone())
						.save_all()
					}
				}
			}), vec![], false));
		}

		(
			Aggregate::chain(&mut g, build_name.clone(), Arc::new(move |_v| {
				Ok(_v)
			}), build_nodes, false), 
			Aggregate::chain(&mut g, deploy_name.clone(), Arc::new(move |_v| {
				if rev {Ok(_v.revisions().save_all()?)} else {Ok(_v)}
			}), deploy_nodes, false)
		)
	}

	fn watcher_dirs_and_tasks(&self, meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (Vec<String>, Vec<String>) {
		let conf_concat: ConfigModule<OptionsConcat, ContentConcat> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};
		let prefix_path = Path::new(&meta.base_path)
				.join(&meta.src_path)
				.join(&conf_concat.options.prefix)
				.to_str().unwrap().to_string();

		let build_name = format!("build:{}", conf_concat.name);
		(vec![prefix_path], vec![build_name])
	}
}

pub fn new_processor() -> Box<dyn GenerateGraphs + Sync> { Box::new(ProcessorConcat()) }