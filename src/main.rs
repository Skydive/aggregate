#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_id_value, or_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
extern crate chrono;
extern crate petgraph;
extern crate regex;
extern crate crc;

mod config;
mod vinyl;
mod aggregate;
mod processor;
mod log;

use std::path::Path;
use std::sync::Arc;

use std::fs;

use std::env;
use async_std::task;

use ansi_term::Color;


use petgraph::Graph;
use vinyl::Vinyl;

use aggregate::{Aggregate, ProcessTask, TaskGraph};

use log::Log;
use processor::PROCESSOR_MAP;


macro_rules! clone_all {
    ($($i:ident),+) => {
        $(let $i = $i.clone();)+
    }
}


// TODO: watchers
// TODO: macro + code cleanup
// TODO: MODULARITY?!?!
// TODO: SWC?!
// TODO: htmlpages improvements

#[async_std::main]
async fn main() -> std::io::Result<()> {
	use crate::config::{Config};
	
	let argsv = env::args().collect::<Vec<String>>();
	let task_name = match argsv.len() {
		c if c <= 1 => "build",
		_ => &argsv[1]
	};

	let data = fs::read_to_string("./config.json").expect("Error!");
	let conf: Config = serde_json::from_str(data.as_str()).expect("JSON Error!");
	let meta = &conf.meta;
	

	let mut g: TaskGraph = Graph::<ProcessTask, ()>::new();
	let mut build_tasks = Vec::default();
	let mut deploy_tasks = Vec::default();

	let mut processor_list = Vec::default();
	for m in &conf.modules {
		match PROCESSOR_MAP.get(m.processor.as_str()) {
			Some(v) => {
				let (build_task, deploy_task) = v.generate_graphs(&mut g, meta.clone(), m.clone());
				build_tasks.push(build_task.clone());
				deploy_tasks.push(deploy_task.clone());
				processor_list.push(m.processor.clone());
			},
			_ => {
				Log::task(format!("{} {}", Color::Red.paint("Missing processor:"), m.processor.clone()));
			}
		}
	}
	// TODO: MOVE INTO SEPERATE FUNCTION OR FILE!?
	// TODO: MOVE REVISIONING OUT OF Vinyl -> Use Box<dyn Modifier?> API?!
	let rev_replace_task = Aggregate::chain(&mut g, "build:rev_replace".to_string(), Box::new({
		clone_all!(meta);
		move |_v| {
			let out_path_files = Path::new(&meta.base_path).join(&meta.build_path).join("**/*").to_path_buf();
			use glob::glob;
			use std::io::Write;
			for file_path in glob(out_path_files.to_str().unwrap()).unwrap().filter_map(|x| x.ok()) {
				match file_path.extension().map(|x| x.to_str().unwrap()) {
					Some("html" | "js" | "css") => {
						println!("{:?}", file_path);
						let mut file_string = fs::read_to_string(&file_path)?;
						_v.revision_pairs.iter().for_each(|(pre_name, post_name)| {
							file_string = file_string.replace(pre_name, post_name);
						});
						let mut f = fs::File::create(&file_path)?;
						f.write(file_string.as_bytes())?;
					}
					_ => {}
				}
			}
			Ok(_v)
		}
	}), build_tasks, false);

	Log::task(format!("{} {}", Color::Cyan.paint("Loaded procesors:"), processor_list.join(", ")));
	Aggregate::chain(&mut g, "build".to_string(), Box::new(|_v| Ok(_v)), vec![rev_replace_task], false);
	Aggregate::chain(&mut g, "deploy".to_string(), Box::new(|_v| Ok(_v)), deploy_tasks, false);

	let ret = Aggregate::execute_by_name(Arc::new(g), task_name);
	println!("JS TASKS END! {}", ret.unwrap());
    Ok(())
}