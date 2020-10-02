use serde_json::Value;
use serde::{Deserialize, Serialize};

use std::vec::Vec;

use std::path::{Path};
use std::sync::Arc;




use crate::aggregate::{Aggregate, TaskGraph, TaskIndex};
use crate::config::{ConfigModule, ConfigMeta};
use crate::vinyl::FileHandle;

use super::GenerateGraphs;

#[derive(Serialize, Deserialize)]
pub struct OptionsClone {
	pub prefix: String,
	pub dest: String,
	#[serde(default)]
	pub revision: bool
}
pub type ContentClone = Vec<String>;

#[derive(Debug)]
struct ProcessorClone();

macro_rules! clone {
    ($i:ident) => (let $i = $i.clone();)
}

macro_rules! clone_all {
    ($($i:ident),+) => {
        $(clone!($i);)+
    }
}

impl GenerateGraphs for ProcessorClone {
	fn generate_graphs(&self, mut g: &mut TaskGraph, meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (TaskIndex, TaskIndex) {
		let conf_clone: ConfigModule<OptionsClone, ContentClone> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};

		let rev = conf_clone.options.revision;

		let build_name = format!("build:{}", conf_clone.name);
		let deploy_name = format!("deploy:{}", conf_clone.name);
		let mut build_nodes = Vec::default();
		let mut deploy_nodes = Vec::default();

		let out_path_build = Path::new(&meta.base_path)
			.join(&meta.build_path);
		let out_path_deploy = Path::new(&meta.base_path)
			.join(&meta.deploy_path);

		let src_path = Path::new(&meta.base_path)
			.join(&meta.src_path).to_path_buf();

		let path = conf_clone.content.iter().map(|p| (&src_path).join(&conf_clone.options.prefix).join(&p).to_path_buf() ).collect::<Vec<_>>();
		let wild_paths = super::path_wildcards(path.clone());
		let wild_paths_post = wild_paths.iter().filter_map(|p| p.strip_prefix(&src_path).ok()).map(|p| p.to_path_buf()).collect::<Vec<_>>();
		//println!("{:?}", ;
		//wild_paths_post.iter().map(|p| build_prefix.join(p)).collect::<Vec<_>>())
		wild_paths_post.iter().for_each(|in_rel_path| {
			//println!("{:?}", in_rel_path);
			let no_prefix_rel_path = in_rel_path.strip_prefix(&conf_clone.options.prefix).unwrap();
			let out_rel_path = Path::new(&conf_clone.options.dest).join(&no_prefix_rel_path).to_path_buf();

			let sub_build_name = format!("{}:{}", build_name, no_prefix_rel_path.to_str().unwrap());
			let sub_deploy_name = format!("{}:{}", deploy_name, no_prefix_rel_path.to_str().unwrap());
			
			//let (moved_path_cap, old_path_cap) = (moved_build_path.clone(), old_build_path.clone());
			build_nodes.push(Aggregate::chain(&mut g, sub_build_name.clone(), Arc::new({
				clone_all!(src_path, out_path_build, out_rel_path, in_rel_path);
				move |_v| {
					let mut hs = FileHandle::load(src_path.clone(), in_rel_path.clone().to_path_buf())?;
					hs.out_path = out_path_build.clone();
					hs.rel_path = out_rel_path.clone();
					Ok(_v.insert(hs).save_all()?)
				}
			}), vec![], true));

			deploy_nodes.push(Aggregate::chain(&mut g, sub_deploy_name.clone(), Arc::new({
				clone_all!(src_path, out_path_deploy, out_rel_path, in_rel_path);
				move |_v| {
					let mut hs = FileHandle::load(src_path.clone(), in_rel_path.clone().to_path_buf())?;
					hs.out_path = out_path_deploy.clone();
					hs.rel_path = out_rel_path.clone();
					if rev {Ok(_v.insert(hs))} else {Ok(_v.insert(hs).save_all()?)}
				}
			}), vec![], true));
		});

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
		let conf_clone: ConfigModule<OptionsClone, ContentClone> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};

		let clone_path = Path::new(&meta.base_path)
				.join(&meta.src_path)
				.join(&conf_clone.options.prefix)
				.to_str().unwrap().to_string();

		let build_name = format!("build:{}", conf_clone.name);
		(vec![clone_path], vec![build_name])
	}
}

pub fn new_processor() -> Box<dyn GenerateGraphs + Sync> { Box::new(ProcessorClone()) }