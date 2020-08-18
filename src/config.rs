use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize)]
pub struct Config {
	pub base_path: String,
	pub src_path: String,
	pub build_path: String,
	pub deploy_path: String,
	pub modules: Vec<ConfigModule>
}

#[derive(Serialize, Deserialize)]
pub struct ConfigModule {
	pub name: String,
	pub processor: String,
	pub options: serde_json::Value,
	pub content: serde_json::Value
}

