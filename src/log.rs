

use std::thread;
use ansi_term::Color;
use chrono::Local;

pub struct Log();

impl Log {
	pub fn task(data: String) {
		println!("[{}] (W{:02}) {}",
			Local::now().format("%T"),
			thread::current().id().as_u64(),
			data);
	}
}