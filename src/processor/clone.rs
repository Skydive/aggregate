use serde_json::Value;
use serde::{Deserialize, Serialize};

use std::vec::Vec;

use std::path::{Path};





use crate::aggregate::{Aggregate, TaskGraph, TaskIndex};
use crate::config::{ConfigModule, ConfigMeta};
use crate::vinyl::{FileHandle, Vinyl};

use super::GenerateGraphs;

#[derive(Serialize, Deserialize)]
pub struct OptionsClone {
	pub prefix: String,
	pub dest: String,
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

		let build_name = format!("build:{}", conf_clone.name);
		let deploy_name = format!("deploy:{}", conf_clone.name);
		let mut build_nodes = Vec::default();
		let mut deploy_nodes = Vec::default();

		let build_prefix = Path::new(&meta.base_path)
			.join(&meta.build_path)
			.join(&conf_clone.options.dest).to_path_buf();
		let deploy_prefix = Path::new(&meta.base_path)
			.join(&meta.deploy_path)
			.join(&conf_clone.options.dest).to_path_buf();

		let src_prefix = Path::new(&meta.base_path)
			.join(&meta.src_path)
			.join(&conf_clone.options.prefix).to_path_buf();

		let path = conf_clone.content.iter().map(|p| (&src_prefix).join(&p).to_path_buf() ).collect::<Vec<_>>();
		let wild_paths = super::path_wildcards(path.clone());
		let wild_paths_post = wild_paths.iter().filter_map(|p| p.strip_prefix(&src_prefix).ok()).collect::<Vec<_>>();
		//println!("{:?}", ;
		//wild_paths_post.iter().map(|p| build_prefix.join(p)).collect::<Vec<_>>())
		wild_paths_post.iter().for_each(|p| {
			let sub_build_name = format!("{}:{}", build_name, p.to_str().unwrap());
			let sub_deploy_name = format!("{}:{}", deploy_name, p.to_str().unwrap());
			
			//let (moved_path_cap, old_path_cap) = (moved_build_path.clone(), old_build_path.clone());
			build_nodes.push(Aggregate::chain(&mut g, sub_build_name.clone(), Box::new({
				clone_all!(src_prefix, build_prefix, p);
				move |_v| {
					let mut hs = FileHandle::load(src_prefix, p.to_path_buf())?;
					hs.out_path = build_prefix;
					Ok(_v.insert(hs))
				}
			}), vec![], true));

			let moved_deploy_path = deploy_prefix.join(p);
			let old_deploy_path = src_prefix.join(p);
			let (moved_path_cap, old_path_cap) = (moved_deploy_path.clone(), old_deploy_path.clone());
			deploy_nodes.push(Aggregate::chain(&mut g, sub_deploy_name.clone(), Box::new({
				clone_all!(src_prefix, deploy_prefix, p);
				move |_v| {
					let mut hs = FileHandle::load(src_prefix, p.to_path_buf())?;
					hs.out_path = deploy_prefix;
					Ok(_v.insert(hs))
				}
			}), vec![], true));

			//println!("{:?}", moved_path);
		});

		(
			Aggregate::chain(&mut g, build_name.clone(), Box::new(move |_v| {
				Ok(_v.revisions().save_all()?)
			}), build_nodes, false), 
			Aggregate::chain(&mut g, deploy_name.clone(), Box::new(move |_v| Ok(Vinyl::default())), deploy_nodes, false)
		)
	}
}

pub fn new_processor() -> Box<dyn GenerateGraphs + Sync> { Box::new(ProcessorClone()) }