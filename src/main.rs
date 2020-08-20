#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
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
use processor::es6;


use async_std::task;


// CODE: MAP STRING->PROCESSOR 
// Loop through all PROC

fn main(){
	use crate::config::Config;


	let data = std::fs::read_to_string("./config.json").expect("Error!");
	let conf: Config = serde_json::from_str(data.as_str()).expect("JSON Error!");

	println!("{}", serde_json::to_string_pretty(&conf).unwrap());

	let mut g = Graph::<ProcessTask, ()>::new();

	for m in &conf.modules {
		match m.processor.as_str() {
			"js" => {

				for (k, v) in m.content.as_object().unwrap() {
					println!("{}, {}", k, v);

					let files = v.as_array().unwrap().iter().map(|x| String::from(x.as_str().unwrap())).collect::<Vec<String>>();
					use std::path::{Path, PathBuf};
					let paths = files.iter().map(|f| Path::new(&conf.base_path).join(&conf.src_path).join(&f).to_path_buf()).collect::<Vec<PathBuf>>();


					let js_load = Aggregate::chain(&mut g, "js_load".to_string(), Box::new(move |_v| {
						println!("PATHS! {:?}", paths.clone());
						Vinyl::load(paths.clone())
					}), vec![]);

					let js_concat = Aggregate::chain(&mut g, "js_concat".to_string(), Box::new(move |v| {
						Ok(v.concat(String::from("./test_concat")))
					}), vec![js_load]);

					let js_save = Aggregate::chain(&mut g, "js_save".to_string(), Box::new(move |v| {
						v.save_all()
					}), vec![js_concat]);

					println!("JS TASKS!");
					let ret = task::block_on(Aggregate::execute(&g, js_save));
					println!("JS TASKS END! {:?}", ret);
					break;


				}
			},
			_ => {

			}
		}
	}
	
	// let mut g = Graph::<ProcessTask, ()>::new();
	// let start = ProcessTask::chain(&mut g, "start".to_string(), |_v| {
 //    	Vinyl::load(vec![
	// 		String::from("./files/test1"),
	// 		String::from("./files/test2"),
	// 		String::from("./files/test3")
	// 	])
 //    }, vec![]);

	// let start_1 = ProcessTask::chain(&mut g, "start".to_string(), |_v| {
 //    	Vinyl::load(vec![String::from("./files/test1")])
 //    }, vec![]);
	// let start_2 = ProcessTask::chain(&mut g, "start".to_string(), |_v| {
 //    	Vinyl::load(vec![String::from("./files/test2")])
 //    }, vec![]);
 //    let start_3 = ProcessTask::chain(&mut g, "start".to_string(), |_v| {
 //    	Vinyl::load(vec![String::from("./files/test3")])
 //    }, vec![]);

 //    let t1 = ProcessTask::chain(&mut g, "t1".to_string(), |v| {
 //    	v.concat(String::from("./test_concat"))
 //    }, vec![start_1, start_2, start_3]);
	   
 //    let end = ProcessTask::chain(&mut g, "end".to_string(), |v| {
 //    	v.save_all()
 //    }, vec![t1]);

 //    task::block_on(ProcessTask::execute_tasks(&g, end));


    // let mut optional = Some(start);
    // let mut vi = Vinyl::new_empty();
    // while let Some(node) = optional {
    // 	let task = g.node_weight(node.clone()).unwrap();
    // 	println!("Starting task: {}", task.name);
    // 	vi = (task.fnc)(vi);
    // 	println!("Completed task: {}", task.name);
    // 	optional = g.neighbors(node).collect::<Vec<_>>().first().map(|n| n.clone())
    // }

}