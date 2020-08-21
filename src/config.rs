use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config<V=serde_json::Value> {
	pub meta: ConfigMeta,
	pub modules: Vec<ConfigModule<V, V>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigMeta {
	pub base_path: String,
	pub src_path: String,
	pub build_path: String,
	pub deploy_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigModule<T, U> {
	pub name: String,
	pub processor: String,
	pub options: T,
	pub content: U
}