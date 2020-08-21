
use std::fmt;
use std::thread;
use std::sync::Arc;

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

	

	pub fn execute(g: Arc<Graph<ProcessTask, ()>>, idx: NodeIndex) -> BoxFuture<'static, Result<Vinyl, AggError>> {
		async move {
			let neighbors = g.neighbors_directed(idx, Direction::Incoming).collect::<Vec<_>>();
			let cur_task = g.node_weight(idx.clone()).unwrap();
			let start_time = Local::now();
			println!("[{}] (W{:0twidth$}) {} {}", start_time.format("%T"),  thread::current().id().as_u64(), Color::Green.paint("Starting Task:"), cur_task.name.clone(), twidth=2);

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
					println!("[{}] (W{:0twidth$}) {} {:?}", Local::now().format("%T"), thread::current().id().as_u64(), Color::Cyan.paint("Series Dependency:"), neigh_names, twidth=2);
					let parent_idx = neighbors.first().map(|n| n.clone()).unwrap();
					ov = Self::execute(g.clone(), parent_idx).await?;
				}
				_ => {
					// PARALLEL EXECUTOR
					println!("[{}] (W{:0twidth$}) {} {:?}", Local::now().format("%T"), thread::current().id().as_u64(), Color::Yellow.paint("Parallel Dependency:"), neigh_names, twidth=2);
					ov = Vinyl::stitch(join_all(neighbors.into_iter().map(|n| {
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
					println!("[{}] (W{:0twidth$}) {}: {} \t\t({})", end_time.format("%T"), thread::current().id().as_u64(), Color::Green.paint("Task Successful"), cur_task.name.clone(), Color::Purple.paint(format!("{:.4}s", (end_time-start_time).to_std().unwrap().as_secs_f64())), twidth=2);
					Ok(v)
				}
				Err(e) => {
					println!("[{}] (W{:0twidth$}) {} {} \nError Message:\n{}", end_time.format("%T"), thread::current().id().as_u64(), Color::Red.paint("Task Failure:"), cur_task.name.clone(), e.to_string(), twidth=2);
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