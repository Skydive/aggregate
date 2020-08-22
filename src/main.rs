#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_id_value)]

#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
extern crate chrono;
extern crate petgraph;
extern crate regex;

mod config;
mod vinyl;
mod aggregate;
mod processor;
mod log;

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

// TODO: revisioning
// TODO: watchers
// TODO: macros + code cleanup
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
	Log::task(format!("{} {}", Color::Cyan.paint("Loaded procesors:"), processor_list.join(", ")));
	Aggregate::chain(&mut g, "build".to_string(), Box::new(|_v| Ok(_v)), build_tasks, false);
	Aggregate::chain(&mut g, "deploy".to_string(), Box::new(|_v| Ok(_v)), deploy_tasks, false);

	let ret = Aggregate::execute_by_name(Arc::new(g), task_name);
	println!("JS TASKS END! {:?}", ret);
    Ok(())
}