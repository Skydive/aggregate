
use std::fmt;

use std::sync::Arc;

use petgraph::Direction;
use petgraph::graph::{Graph, NodeIndex};


use async_std::task;
use futures::future::join_all; 
use futures::future::{BoxFuture, FutureExt};

use crate::vinyl::{Vinyl, VinylError};

use ansi_term::Color;

use chrono::Local;

use crate::log::Log;
//type TaskFnc = fn(Vinyl) -> Vinyl;
// TODO: MAKE THIS INTO AN ASYNC CLOSURE<--/--> FUTURE DATATYPE!
pub type TaskFnc = Arc<dyn Sync + Send + Fn(Vinyl) -> Result<Vinyl, VinylError>>;

#[derive(Clone)]
pub struct ProcessTask {
	pub name: String, 
	pub fnc: TaskFnc,
	pub quiet: bool
}
pub type TaskGraph = Graph<ProcessTask, ()>;
pub type TaskIndex = NodeIndex;

pub struct Aggregate();
impl Aggregate {
	pub fn chain(g: &mut TaskGraph, name: String, fnc: TaskFnc, dep_node_idx: Vec<TaskIndex>, q: bool) -> TaskIndex {
		let node = g.add_node(ProcessTask {name: name,  fnc: fnc, quiet: q});
		dep_node_idx.iter().for_each(|ni| {g.add_edge(ni.clone(), node.clone(), ());} );
		node.clone()
	}

	pub fn execute_by_name(g: Arc<TaskGraph>, name: &str) -> Result<Vinyl, AggError> {
		for idx in g.node_indices() {
			let task = g.node_weight(idx.clone()).unwrap();
			//println!("{}", task.name.clone());
			if &task.name == name {
				Log::task(format!("{}\t{}", Color::Green.paint("Task Found:"), name));
				return task::block_on(Self::execute(g, idx.clone()));
			}
		}
		Log::task(format!("{}\t{}", Color::Red.paint("Task Missing:"), name));
		Err(AggError{name: "???".to_string(), msg: "???".to_string()})
	}

	pub fn execute(g: Arc<TaskGraph>, idx: TaskIndex) -> BoxFuture<'static, Result<Vinyl, AggError>> {
		async move {
			let neighbors = g.neighbors_directed(idx, Direction::Incoming).collect::<Vec<_>>();
			let cur_task = g.node_weight(idx.clone()).unwrap();
			let start_time = Local::now();
			// TODO: IMPLEMENT LOG.RS LOG FUNCTION

			//if !cur_task.quiet { Log::task(format!("{} {}", Color::Green.paint("Task Starting:"), cur_task.name.clone())); }

			// DEPENDENCIES
			let mut ov = Vinyl::default();
			// let neigh_names = neighbors.iter().map(|n| {
			// 	let parent_task = g.node_weight(n.clone()).unwrap();
			// 	parent_task.name.clone()
			// }).collect::<Vec<_>>();
			match neighbors.len() {
				0 => {}
				1 => {
					// SERIES EXECUTOR
					//Log::task(format!("{} {:?}", Color::Green.paint("Parallel Dependency:"), neigh_names));
					if !cur_task.quiet { Log::task(format!("{}\t{}", Color::Green.paint("Task Starting:"), cur_task.name.clone())); }
					
					let parent_idx = neighbors.first().map(|n| n.clone()).unwrap();
					ov = Self::execute(g.clone(), parent_idx).await?;
				}
				_ => {
					// PARALLEL EXECUTOR
					if !cur_task.quiet { Log::task(format!("{} {:23}<{}>", Color::Green.paint("Task Starting:"), cur_task.name.clone(), Color::Yellow.paint(neighbors.len().to_string()))); }
					//if !cur_task.quiet { Log::task(format!("{} {:?}", Color::Yellow.paint("Task Starting:"), neigh_names)); }
					
					ov = Vinyl::flatten(join_all(neighbors.into_iter().map(|n| {
						let new_g = g.clone();
						async move { Self::execute(new_g, n.clone()).await }
					})
					.map(task::spawn)
					.collect::<Vec<_>>()).await.into_iter().collect::<Result<Vec<_>, _>>()?);
				}
			}
			
			let result = (cur_task.fnc)(ov); 
			let end_time = Local::now();
			match result {
				Ok(v) => {
					// TODO: CUSTOM SI UNIT DURATIONS
					// TODO: Color duration
					if !cur_task.quiet { Log::task(format!("{} {:20}({})", Color::Green.paint("Task Success: "), cur_task.name.clone(), Color::Purple.paint(format!("{:.4}s", (end_time-start_time).to_std().unwrap().as_secs_f64())))); }
					Ok(v)
				}
				Err(e) => {
					let gap = (end_time-start_time).to_std().unwrap().as_secs_f64();
					Log::task(format!("{} {:20}({}) \nError Message:\n{}", Color::Red.paint("Task Failure: "), cur_task.name.clone(), Color::Purple.paint(format!("{:.4}s", gap)), e.to_string()));
					Err(AggError{name: cur_task.name.clone(), msg: e.to_string()})
				}
			}
		}.boxed()
	}
}

#[derive(Debug)]
pub struct AggError {
	pub name: String,
	pub msg: String
}

impl From<VinylError> for AggError {
    fn from(e: VinylError) -> Self {
        AggError {
        	name: "vinyl".to_string(),
            msg: e.to_string(),
        }
    }
}


impl fmt::Display for AggError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "AggError: {} {}", self.name, self.msg)
    }
}