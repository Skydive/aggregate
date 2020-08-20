
pub mod es6;

use std::collections::HashMap;

use petgraph::Graph;
use super::aggregate::ProcessTask;




lazy_static! {
	static ref PROCESSOR_NAMES: HashMap<&'static str, ()> = {
		let mut map = HashMap::new();

		map
	};
}

