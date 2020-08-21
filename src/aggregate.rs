
use std::fmt;

use petgraph::Direction;
use petgraph::graph::{Graph, NodeIndex};

use async_std::task;
use futures::future::join_all; 
use futures::future::{BoxFuture, FutureExt};

use crate::vinyl::{Vinyl, VinylError};

use ansi_term::Color;

use chrono::Local;

//type TaskFnc = fn(Vinyl) -> Vinyl;
// TODO: MAKE THIS INTO AN ASYNC CLOSURE<--/--> FUTURE DATATYPE!
pub type TaskFnc = Box<dyn Sync + Send + Fn(Vinyl) -> Result<Vinyl, VinylError>>;
pub struct ProcessTask {
	pub name: String, 
	pub fnc: TaskFnc,
}

pub struct Aggregate();
impl Aggregate {
	pub fn chain(g: &mut Graph<ProcessTask, ()>, name: String, fnc: TaskFnc, dep_node_idx: Vec<NodeIndex>)  -> NodeIndex {
		let node = g.add_node(ProcessTask {name: name,  fnc: fnc});
		dep_node_idx.iter().for_each(|ni| {g.add_edge(ni.clone(), node.clone(), ());} );
		node.clone()
	}

	

	pub fn execute<'a>(g: &'a Graph<ProcessTask, ()>, idx: NodeIndex) -> BoxFuture<'a, Result<Vinyl, AggError>> {
		async move {
			let neighbors = g.neighbors_directed(idx, Direction::Incoming).collect::<Vec<_>>();
			let cur_task = g.node_weight(idx.clone()).unwrap();
			let start_time = Local::now();
			println!("[{}] {} {}", start_time.format("%T"),  Color::Green.paint("Starting Task:"), cur_task.name.clone());

			// DEPENDENCIES
			let mut ov = Vinyl::default();
			let neigh_names = neighbors.iter().map(|n| {
				let parent_task = g.node_weight(n.clone()).unwrap();
				parent_task.name.clone()
			}).collect::<Vec<_>>();
			match neighbors.len() {
				0 => {}
				1 => {
					// SERIES EXECUTOR
					let parent_idx = neighbors.first().map(|n| n.clone()).unwrap();
					println!("[{}] {} {:?}", Local::now().format("%T"), Color::Cyan.paint("Series Dependency:"), neigh_names);
					ov = Self::execute(&g, parent_idx).await?;
				}
				_ => {
					// PARALLEL EXECUTOR
					let neigh_names = neighbors.iter().map(|n| {
						let parent_task = g.node_weight(n.clone()).unwrap();
						parent_task.name.clone()
					}).collect::<Vec<_>>();
					println!("[{}] {} {:?}", Local::now().format("%T"), Color::Yellow.paint("Parallel Dependency:"), neigh_names);
					ov = Vinyl::stitch(join_all(neighbors.iter().map(|n| {
						async move {
							Self::execute(&g, n.clone()).await
						}
					}).collect::<Vec<_>>()).await.into_iter().collect::<Result<Vec<_>, _>>()?);
				}
			}

			
			let result = (cur_task.fnc)(ov); 
			let end_time = Local::now();
			match result {
				Ok(v) => {
					// TODO: CUSTOM SI UNIT DURATIONS
					// TODO: Color duration
					println!("[{}] {} {} \t({}s)", start_time.format("%T"), Color::Green.paint("Task Successful:"), cur_task.name.clone(), (end_time-start_time).to_std().unwrap().as_secs_f64());
					Ok(v)
				}
				Err(e) => {
					println!("{}", Color::Red.paint(format!("[{}] Task Failure: {}, Message: {}", end_time.format("%T"), cur_task.name.clone(), e.to_string())));
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