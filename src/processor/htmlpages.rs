use serde_json::Value;
use serde::{Deserialize, Serialize};

use std::vec::Vec;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;

use regex::Regex;

use crate::aggregate::{Aggregate, TaskGraph, TaskIndex};
use crate::config::{ConfigModule, ConfigMeta};
use crate::vinyl::{Vinyl, VinylError, FileHandle};

use super::GenerateGraphs;



#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OptionsHTMLPages {
	pub prefix: String,
	pub dest: String
}

pub type ContentHTMLPages = Vec<String>;


#[derive(Debug, Default)]
pub struct ProcessorHTMLPages();

macro_rules! clone {
    ($i:ident) => (let $i = $i.clone();)
}

macro_rules! clone_all {
    ($($i:ident),+) => {
        $(clone!($i);)+
    }
}


lazy_static! {
	static ref REGEX_CMD: Regex = Regex::new(r"<!--@(.+)[^\S\r\n](.+)[^\S\r\n]-->").unwrap();
	static ref STAGE1_IGNORE_CMD: [&'static str; 2] = ["output-begin", "output-end"];
}

impl ProcessorHTMLPages {
	// TODO: replace with import/require
	pub fn format_import_arg(arg: String, template_dirname: PathBuf, meta: ConfigMeta) -> PathBuf {
		if &arg.clone()[0..1] == "/" {
			Path::new(&meta.base_path)
				.join(&meta.src_path)
				.join(&arg.clone()[1..])
				.to_path_buf()
		} else {
			(*template_dirname).join(&arg)
		}
	}
	// TODO: recursive templating
		/* if &arg.clone()[0..1] == "/" {
			// GET ROOT TEMPLATE...
			let mut parent_path = template_path.parent();
			while let Some(pth) = parent_path {
				if pth.join("template.html").exists() {
					println!("{:?}", pth);	
				} else {
					break;
				}
				parent_path = pth.parent();
			}
		}
		"".to_string() */

	pub fn process_template(template_path: PathBuf, meta: ConfigMeta) -> Result<String, VinylError> {
		// PERFORM IMPORT COMMANDS
		let mut stage1_template = String::default();
		let template_dirname = (*template_path).parent().unwrap().to_path_buf();

		let template = fs::read_to_string(template_path.to_str().unwrap().to_string()).expect("Error!");
		let mut last_index: usize = 0;
		for pm in REGEX_CMD.find_iter(&template) {
			let cap = REGEX_CMD.captures(pm.as_str()).unwrap().iter().map(|om| {
				om.unwrap().clone().as_str()
			}).collect::<Vec<&str>>();

			stage1_template += &template[last_index..pm.start()];

			match &cap[..] {
				[_, "import", import_path] => {
					let file_path = Self::format_import_arg(import_path.to_string(), template_dirname.clone(), meta.clone());
				
					let data_str = fs::read_to_string(file_path)?;
					stage1_template += &data_str;
				}
				[_, "import-js", import_path] => {
					let file_path = Self::format_import_arg(import_path.to_string(), template_dirname.clone(), meta.clone());
				
					let data_str = fs::read_to_string(file_path)?;
					stage1_template += "<script>";
						stage1_template += &data_str;
					stage1_template += "</script>";
				}
				[_, s, ..] => {
					if let Some(_) = STAGE1_IGNORE_CMD.iter().find(|ss| ss == &s) {
						stage1_template += &template[pm.start()..pm.end()]; // INCLUDE IGNORED COMMANDS
					}
				}
				_ => {}
			}
			last_index = pm.end();
		}
		stage1_template += &template[last_index..];
		//println!("{}", stage1_template);
		Ok(stage1_template)
		//Self::write_outputs(stage1_template, template_dirname.clone(), meta)
	}

	pub fn write_outputs(template_path: PathBuf, meta: ConfigMeta, stage1_file: String, path_prefix: PathBuf, out_prefix: PathBuf) -> Result<(), VinylError> {
		let template_dirname = (*template_path).parent().unwrap().to_path_buf();
		//println!("{}", stage1_file.clone());
		let mut output_map: HashMap<&str, usize> = HashMap::new();
		for pm in REGEX_CMD.find_iter(&stage1_file) {
			let cap = REGEX_CMD.captures(pm.as_str()).unwrap().iter().map(|om| {
				om.unwrap().clone().as_str()
			}).collect::<Vec<&str>>();
			match &cap[..] {
				[_, "output-begin", out_path] => {
					output_map.insert(out_path, pm.end());
				},
				[_, "output-end", out_path] => {
					if let Some(&start) = output_map.get(out_path) {

						let pages_post = template_dirname.strip_prefix(path_prefix.clone()).unwrap();
						let page_rel_path = Path::new(pages_post).join(out_path);
						
						FileHandle::new(out_prefix.clone(), page_rel_path, stage1_file.as_str()[start..pm.start()].as_bytes().to_vec()).save()?;
					}
				}
				_ => {}
			}
		}
		Ok(())
	}
}

impl GenerateGraphs for ProcessorHTMLPages {
	fn generate_graphs(&self, mut g: &mut TaskGraph, meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (TaskIndex, TaskIndex) {
		let conf_html: ConfigModule<OptionsHTMLPages, ContentHTMLPages> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};
		
		let build_name = format!("build:{}", conf_html.name.clone());
		let deploy_name = format!("deploy:{}", conf_html.name.clone());
		let mut build_nodes = Vec::default();
		let mut deploy_nodes = Vec::default();

		let build_prefix = Path::new(&meta.base_path)
			.join(&meta.build_path)
			.join(&conf_html.options.dest).to_path_buf();
		let deploy_prefix = Path::new(&meta.base_path)
			.join(&meta.deploy_path)
			.join(&conf_html.options.dest).to_path_buf();

		let path_prefix = Path::new(&meta.base_path)
			.join(&meta.src_path)
			.join(&conf_html.options.prefix).to_path_buf();

		for page in conf_html.content.iter() {
			//println!("{}", page.clone());
			let build_name_page = format!("{}:{}", build_name.clone(), page.clone());
			let deploy_name_page = format!("{}:{}", deploy_name.clone(), page.clone());

			let page_template_path = path_prefix.join(page).join("template.html").to_path_buf();
			
			build_nodes.push(Aggregate::chain(&mut g, build_name_page.clone(), Arc::new({
				clone_all!(meta, page, page_template_path, path_prefix, build_prefix);
				move |_v| {
					if !(*page_template_path).exists() { 
						return Err(VinylError{msg: format!("ERROR: {} {:?}", page.clone(), page_template_path.clone())});
					}
					let stage1_template = Self::process_template(page_template_path.clone(), meta.clone())?;
					Self::write_outputs(page_template_path.clone(), meta.clone(), stage1_template, path_prefix.clone(), build_prefix.clone())?;
					Ok(_v)
				}
			}), vec![], false));

			let (cap_meta, cap_page, cap_template_path, cap_path_prefix, cap_deploy_prefix) = (meta.clone(), page.clone(), page_template_path.clone(), path_prefix.clone(), deploy_prefix.clone());
			deploy_nodes.push(Aggregate::chain(&mut g, deploy_name_page.clone(), Arc::new(move |_v| {
				if !(*cap_template_path).exists() { 
					return Err(VinylError{msg: format!("ERROR: {} {:?}", cap_page.clone(), cap_template_path.clone())});
				}
				let stage1_template = Self::process_template(cap_template_path.clone(), cap_meta.clone())?;
				Self::write_outputs(cap_template_path.clone(), cap_meta.clone(), stage1_template, cap_path_prefix.clone(), cap_deploy_prefix.clone())?;
				Ok(_v)
			}), vec![], false));
		}

		(
			Aggregate::chain(&mut g, build_name.clone(), Arc::new(move |_v| Ok(Vinyl::default())), build_nodes, false), 
			Aggregate::chain(&mut g, deploy_name.clone(), Arc::new(move |_v| Ok(Vinyl::default())), deploy_nodes, false)
		)
	}

	fn watcher_dirs_and_tasks(&self, meta: ConfigMeta, cfg_mod: ConfigModule<Value, Value>) -> (Vec<String>, Vec<String>) {
		let conf_html: ConfigModule<OptionsHTMLPages, ContentHTMLPages> = ConfigModule {
			name: cfg_mod.name.clone(),
			processor: cfg_mod.processor.clone(),
			options: serde_json::from_value(cfg_mod.options.clone()).unwrap(),
			content: serde_json::from_value(cfg_mod.content.clone()).unwrap(),
		};
		let pages_path = Path::new(&meta.base_path)
				.join(&meta.src_path)
				.join(&conf_html.options.prefix)
				.to_str().unwrap().to_string();
		let requires_path = Path::new(&meta.base_path)
				.join(&meta.src_path)
				.join("requires")
				// TODO: .join(&conf_html.options.prefix_requires)
				.to_str().unwrap().to_string();

		let build_name = format!("build:{}", conf_html.name);
		(vec![pages_path, requires_path], vec![build_name])
	}
}

pub fn new_processor() -> Box<dyn GenerateGraphs + Sync> { Box::new(ProcessorHTMLPages()) }