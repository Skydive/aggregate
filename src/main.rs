#![feature(proc_macro_hygiene, decl_macro)]
#![feature(thread_id_value, or_patterns)]

#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
extern crate chrono;
extern crate petgraph;
extern crate regex;
extern crate notify;
extern crate crc;

#[cfg(feature = "swc")]
extern crate swc_proc;

mod config;
mod vinyl;
mod aggregate;
mod processor;
mod log;


use std::path::Path;
use std::sync::Arc;

use std::fs;
use std::env;

use ansi_term::Color;
use petgraph::Graph;

use aggregate::{Aggregate, ProcessTask, TaskGraph};
use log::Log;
use processor::{PROCESSOR_MAP};
use config::{Config};

macro_rules! clone_all {
    ($($i:ident),+) => {
        $(let $i = $i.clone();)+
    }
}

// TODO: macro + code cleanup
	// TODO: clean main()
	// TODO: PATH --> STRING macro
	// TODO: log format
	// TODO: Shorten aggregate chain
	// TODO: PATH construction shorten
	// TODO: FIX clone!
// TODO: aggregate cleaner error handling (anyhow::Error!?)
// TODO: Speed up revisioning (somehow)

// TITLE: WATCHER IMPROVEMENTS
// TODO: watchers into new file
// TODO: per_file watchers <file_path -> task_name> [DO PER_DIRECTORY + WILDCARD!?]


// TITLE: PROCESSOR IMPROVEMENTS
// TODO: processor (quiet) argument
// TODO: Add revision option ARGS to concat and swc  
// TODO: MODULARITY?!?! http://adventures.michaelfbryan.com/posts/plugins-in-rust/
// TODO: JS MINIFY!?
// TODO: SWC?! (ALMOST DONE)
// TODO: SASS? 
// TODO: htmlpages improvements (recursive templating!, better error handling)

#[async_std::main]
async fn main() -> std::io::Result<()> {
	println!(
"	   ---------------------------------
	   |     {}     |
	   ---------------------------------",
Color::Cyan.paint("Welcome to Aggregate!"));

	
	let argsv = env::args().collect::<Vec<String>>();
	let task_name = match argsv.len() {
		c if c <= 1 => "build",
		_ => &argsv[1]
	};

	let data = fs::read_to_string("./config.json").expect("Error!");
	let conf: Config = serde_json::from_str(data.as_str()).expect("JSON Error!");
	let meta = &conf.meta;

	let mut g: TaskGraph = Graph::<ProcessTask, ()>::new(); // TODO: Make Lazy Static + Arc<RcLock<Graph::new()>>
	let mut build_tasks = Vec::default();
	let mut deploy_tasks = Vec::default();

	let mut processor_list = Vec::default();
	for m in &conf.modules {
		match PROCESSOR_MAP.get(m.processor.as_str()) {
			Some(v) => {
				let (build_task, deploy_task) = v.generate_graphs(&mut g, meta.clone(), m.clone());
				build_tasks.push(build_task.clone());
				deploy_tasks.push(deploy_task.clone());
				processor_list.push(format!("{} ({})", m.name, m.processor.clone()));
			},
			_ => {
				Log::task(format!("{} {}", Color::Red.paint("Missing processor:"), m.processor.clone()));
			}
		}
	}
	// TODO: MOVE INTO SEPERATE FUNCTION OR FILE!?
	// TODO: MOVE REVISIONING OUT OF Vinyl -> Use Box<dyn Modifier?> API?!
	// TODO: PARALELLISE!?!?
	let rev_replace_task = Aggregate::chain(&mut g, "deploy:rev_replace".to_string(), Arc::new({
		clone_all!(meta);
		move |_v| {
			let out_path_files = Path::new(&meta.base_path).join(&meta.deploy_path).join("**/*").to_path_buf();
			use glob::glob;
			use std::io::Write;
			for file_path in glob(out_path_files.to_str().unwrap()).unwrap().filter_map(|x| x.ok()) {
				match file_path.extension().map(|x| x.to_str().unwrap()) {
					Some("html" | "js" | "css") => {
						println!("Replacing in: {:?}", file_path);
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
	}), deploy_tasks, false);

	Log::task(format!("{} {}", Color::Blue.paint("Loaded procesors:"), processor_list.join(", ")));
	Aggregate::chain(&mut g, "build".to_string(), Arc::new(|_v| Ok(_v)), build_tasks, false);
	Aggregate::chain(&mut g, "deploy".to_string(), Arc::new(|_v| Ok(_v)), vec![rev_replace_task], false);


	// WATCH SECTION OF CODE
	use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};
	use std::sync::mpsc::channel;
	use std::time::Duration;
	use std::collections::HashMap;

	let watcher_g = g.clone();
	let conf_modules = conf.modules.clone();
	let meta = meta.clone();
	Aggregate::chain(&mut g, "watch".to_string(), Arc::new(move |_v| {
		let (tx, rx) = channel();
		let mut watcher = watcher(tx, Duration::from_millis(500)).unwrap();
		
		let mut paths_tasks: HashMap<String, Vec<String>> = HashMap::new();
		for m in conf_modules.clone() {
			match PROCESSOR_MAP.get(m.processor.as_str()) {
				Some(v) => {
					let (vec_paths, vec_tasks) = v.watcher_dirs_and_tasks(meta.clone(), m.clone());

					vec_paths.iter().for_each(|p_str| {
						Log::task(format!("{} {} -> {}", Color::Blue.paint("Task Watch:    "), p_str, vec_tasks.join(", ")));
						paths_tasks.insert(p_str.clone(), vec_tasks.clone());
						watcher.watch(p_str, RecursiveMode::Recursive).unwrap();
					})
				},
				_ => {
					Log::task(format!("{} {}", Color::Red.paint("Missing processor:"), m.processor.clone()));
				}
			}
		}

		// 

		loop {
	        match rx.recv() {
	           Ok(event) => {
	           		//println!("{:?}", event);
	           		match event {
	           			DebouncedEvent::Create(p) | DebouncedEvent::Write(p) => {
	           				let rel_dir = p.strip_prefix(env::current_dir().unwrap()).unwrap().to_path_buf();
	           				//println!("{:?}", rel_dir);

	           				Log::task(format!("{} {:?}", Color::Yellow.paint("File Changed:   "), rel_dir));
	           				paths_tasks.iter().filter(|(pth_name, _)| rel_dir.starts_with(pth_name)).for_each(|(_,ts)| {
	           					Log::task(format!("{} {:?}", Color::Yellow.paint("Executing Tasks:"), ts.join(", ")));
	           					ts.iter().for_each(|t| {
	           						let ret = Aggregate::execute_by_name(Arc::new(watcher_g.clone()), &t);
	           						//println!("JS TASKS END! {}", ret.unwrap());
	           					});
	           					
	           				});
	           			}
	           			_ => {}
	           		}
	           },
	           Err(e) => println!("watch error: {:?}", e),
	        }
	    }
	    // TODO: CTRL+C LOOP!? - WATCH may be called in non-main thread....
		//Ok(_v)
	}), vec![], false);

	// COMMAND EXECUTE!
	
	let ret = Aggregate::execute_by_name(Arc::new(g), task_name);
	//println!("JS TASKS END! {}", ret.unwrap());
    Ok(())
}