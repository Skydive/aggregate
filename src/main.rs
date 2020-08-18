#![feature(proc_macro_hygiene, decl_macro)]

// extern crate notify;
// extern crate glob;

// #[macro_use] extern crate serde_json; 

mod config;
mod task;
mod tests;


//use async_std::prelude::*;
//use futures;


fn main() {
	use std::fs;
	use crate::config::Config;

	let data = fs::read_to_string("./config.json").expect("Error!");
	let conf: Config = serde_json::from_str(data.as_str()).expect("JSON Error!");

	println!("{}", serde_json::to_string_pretty(&conf).unwrap());

	for m in conf.modules {
		match m.processor.as_str() {
			"js" => {

				println!("{}", serde_json::to_string_pretty(&m).unwrap());
			},
			_ => {}
		}
		println!("{}", m.processor);
	}
	
	use crate::task::Vinyl;

	let mut v = Vinyl::load(vec![
		String::from("./files/test1"),
		String::from("./files/test2"),
		String::from("./files/test3")
		])
		.concat(String::from("./test_concat"))
		.save_all();


	// map processors

	// for each processor in conf

	// run processor code on contents
	// 


}