#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_id_value)]

#[macro_use]
extern crate ansi_term;
extern crate chrono;

extern crate petgraph;


use petgraph::Graph;

// extern crate notify;
// extern crate glob;

mod config;
mod vinyl;
mod aggregate;
mod processor;

use vinyl::Vinyl;

use aggregate::{Aggregate, AggError, ProcessTask};

use processor::js::*;

use std::sync::Arc;

use std::fs;
use std::path::{Path, PathBuf};
use async_std::task;


// CODE: MAP STRING->PROCESSOR 
// Loop through all PROC

#[async_std::main]
async fn main() -> std::io::Result<()> {
	use crate::config::{Config, ConfigModule};

	let var: Box<i32> = Box::new(50);

	

	let data = fs::read_to_string("./config.json").expect("Error!");
	let conf: Config = serde_json::from_str(data.as_str()).expect("JSON Error!");
	let meta = &conf.meta;

	//println!("{}", serde_json::to_string_pretty(&conf).unwrap());
	
	let mut g = Graph::<ProcessTask, ()>::new();
	let mut processors_tasks = Vec::default();
	for m in &conf.modules {
		match m.processor.as_str() {
			"js" => {
				//let conf_js: ConfigModule<OptionsJS, ContentJS> = serde_json::from_str(&serde_json::to_string(m).unwrap()).unwrap();
				let conf_js: ConfigModule<OptionsJS, ContentJS> = ConfigModule {
					name: m.name.clone(),
					processor: m.processor.clone(),
					options: serde_json::from_value(m.options.clone()).unwrap(),
					content: serde_json::from_value(m.content.clone()).unwrap(),
				};
				
				//println!("{}", serde_json::to_string_pretty(&conf_js).unwrap());
				
				let build_name = format!("build:{}", conf_js.processor);
				let mut build_parents = Vec::default();
				for (k, v) in &conf_js.content {
					//println!("{}, {:?}", k, v);

					let sub_build_name = format!("{}:{}", build_name, k);
					let dest_file = Path::new(&meta.base_path)
						.join(&meta.build_path)
						.join(&conf_js.options.dest)
						.join(&format!("{}.js", k)).to_path_buf();
					let mdestfile = dest_file.clone();

					let paths = v.iter().map(|f|
						Path::new(&meta.base_path)
						.join(&meta.src_path)
						.join(&conf_js.options.prefix)
						.join(&k)
						.join(&f).to_path_buf()
					).collect::<Vec<PathBuf>>();

					build_parents.push(Aggregate::chain(&mut g, sub_build_name.clone(), Box::new(move |_v| {
						//println!("PATHS! {:?}", paths.clone());
						Vinyl::load(paths.clone())?
							.concat(mdestfile.clone())
							.save_all()
					}), vec![]));

					println!("{:?} {:?}", build_name.clone(), dest_file.clone());

				}

				processors_tasks.push(Aggregate::chain(&mut g, build_name.clone(), Box::new(move |_v| {
					Ok(Vinyl::default())
				}), build_parents));

				
			},
			_ => {

			}
		}
	}
	let build_node = Aggregate::chain(&mut g, "build".to_string(), Box::new(move |_v| {
		Ok(Vinyl::default())
	}), processors_tasks);

	let ret = task::block_on(Aggregate::execute(Arc::new(g), build_node));
	println!("JS TASKS END!");

    Ok(())
}