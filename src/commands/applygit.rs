use anyhow::Error;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Editor, Input};

use handlebars::handlebars_helper;
use handlebars::{
	to_json, Context, Handlebars, Helper, JsonRender, Output, RenderContext, RenderError,
};
use ignore::{DirEntry, Walk, WalkBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	process,
};
use walkdir::WalkDir;

use crate::commands::apply;
use crate::config;

pub fn run() -> Result<(), Error> {
	let git_dir = "/home/edwin/groups/fox-templates";
	let config = config::get_config().expect("Failed to get config");

	for template in PathBuf::from(&config.templates_dir)
		.join("templates")
		.read_dir()
		.unwrap()
	{
		let template = template?;
		let template_name = template.file_name();

		let source_dir = PathBuf::from(&config.templates_dir)
			.join("templates")
			.join(&template_name);
		let target_dir = PathBuf::from(git_dir).join(&template_name);
		apply::run(source_dir.clone(), target_dir.clone()).unwrap();
		println!(
			"copied: {} :: {}",
			source_dir.display(),
			target_dir.display()
		);
	}

	Ok(())
}
