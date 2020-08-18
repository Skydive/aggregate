
extern crate petgraph;

use petgraph::Graph;


use crate::task::ProcessTask;

// trait Exec {
// 	fn execute_tasks()
// }

// impl Exec for Graph<Vec<ProcessTask>, ()> {
// 	fn execute_tasks(&self) {

// 	}
// }


#[test]
fn process_print_parallel() {
	let mut g = Graph::<ProcessTask, ()>::new();

    let start = g.add_node(ProcessTask{name: String::from("start"), fnc:|| { println!("start"); } } );
    let t1 = g.add_node(ProcessTask{name: String::from("t1"), fnc:|| { println!("t1"); } } );
    g.add_edge(start.clone(), t1.clone(), ());

    let t2 = g.add_node(ProcessTask{name: String::from("t2"), fnc:|| { println!("t2"); } } );
    g.add_edge(t1.clone(), t2.clone(), ());

    let end = g.add_node(ProcessTask{name: String::from("end"), fnc:|| { println!("end"); } } );
    g.add_edge(t2.clone(), end.clone(), ());


    let mut optional = Some(start);
    while let Some(node) = optional {
    	let process = g.node_weight(node.clone()).unwrap();
    	(process.fnc)();
    	optional = g.neighbors(node).collect::<Vec<_>>().first().map(|n| n.clone())
    }
}

#[test]
fn process_print_series() {
	let mut g = Graph::<ProcessTask, ()>::new();

    let start = g.add_node(ProcessTask{name: String::from("start"), fnc:|| { println!("start"); } } );
    let t1 = g.add_node(ProcessTask{name: String::from("t1"), fnc:|| { println!("t1"); } } );
    g.add_edge(start.clone(), t1.clone(), ());

    let t2 = g.add_node(ProcessTask{name: String::from("t2"), fnc:|| { println!("t2"); } } );
    g.add_edge(t1.clone(), t2.clone(), ());

    let end = g.add_node(ProcessTask{name: String::from("end"), fnc:|| { println!("end"); } } );
    g.add_edge(t2.clone(), end.clone(), ());


    let mut optional = Some(start);
    while let Some(node) = optional {
    	let process = g.node_weight(node.clone()).unwrap();
    	(process.fnc)();
    	optional = g.neighbors(node).collect::<Vec<_>>().first().map(|n| n.clone())
    }
}

#[test]
fn chain() {
    let mut g = Graph::<i64, ()>::new();

    let start = g.add_node(64);
    let t1 = g.add_node(63);
    g.add_edge(start.clone(), t1.clone(), ());

    let t2 = g.add_node(62);
    g.add_edge(t1.clone(), t2.clone(), ());

    let end = g.add_node(61);
    g.add_edge(t2.clone(), end.clone(), ());


    let mut optional = Some(start);
    while let Some(node) = optional {
	    println!("{}", g.node_weight(node.clone()).unwrap());
    	optional = g.neighbors(node).collect::<Vec<_>>().first().map(|n| n.clone())
    }
}