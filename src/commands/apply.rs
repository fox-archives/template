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

use crate::config;

handlebars_helper!(include_file: |v: Value| {
	let filename = v.as_str().unwrap();

	let config = config::get_config().expect("Failed to get config");
	let filepath = PathBuf::from(config.templates_dir).join("internal/files").join(filename);

	if !filepath.exists() {
		"NOT EXISTS".to_string()
	} else {
		fs::read_to_string(filepath).unwrap()
	}
});

pub fn run(source_dir: PathBuf, target_dir: PathBuf) -> Result<(), Error> {
	let verbose = false;
	let global_config = config::get_config().expect("Failed to get config");

	let template_data = {
		#[derive(Debug, Serialize, Deserialize)]
		struct TemplateConfig {
			variables: Option<Variables>,
		}

		#[derive(Debug, Serialize, Deserialize)]
		struct Variables {
			#[serde(flatten)]
			map: HashMap<String, VariableEntry>,
		}

		#[derive(Debug, Serialize, Deserialize)]
		#[serde(untagged)]
		enum VariableEntry {
			String(String),
			Object(Variable),
		}

		#[derive(Debug, Serialize, Deserialize)]
		struct Variable {
			default: Option<String>,
			r#type: Option<VariableType>,
			prompt: Option<String>,
		}

		#[derive(Debug, Serialize, Deserialize)]
		#[serde(rename_all = "lowercase")]
		enum VariableType {
			String,
			Boolean,
		}

		let toml_file = PathBuf::from(&source_dir).join("template.toml");
		// println!("{:?}", &toml_file);

		let toml_text = fs::read_to_string(toml_file).unwrap_or("[variables]\n".to_string());
		// println!("{}", &toml_text.as_str());

		let config: TemplateConfig = toml::from_str(toml_text.as_str()).unwrap();
		// println!("config: {:#?}", config);

		let mut data: HashMap<String, String> = HashMap::new();
		for var in config.variables.unwrap().map {
			let (key, value) = var;
			let v = match value {
				VariableEntry::String(val) => val,
				VariableEntry::Object(obj) => obj.default.unwrap_or_else(|| {
					let input = Input::new()
						.with_prompt(format!("Value of key: '{}'?", &key).to_string())
						.with_initial_text("")
						.interact_text()
						.unwrap();
					input
				}),
			};
			data.insert(key, v);
		}
		data.insert(
			"project_name".to_string(),
			source_dir
				.as_path()
				.file_name()
				.unwrap()
				.to_string_lossy()
				.to_string(),
		);
		data.insert(
			"full_name".to_string(),
			global_config.variables.full_name.clone(),
		);
		data.insert(
			"license".to_string(),
			global_config.variables.license.clone(),
		);
		data
	};

	for file in WalkBuilder::new(&source_dir)
		.hidden(false)
		.parents(true)
		.git_global(false)
		.git_exclude(false)
		.filter_entry(|entry: &DirEntry| {
			if entry.file_name() == ".git" {
				return false;
			}

			if entry.file_name() == "template.toml" {
				return false;
			}

			return true;
		})
		.build()
	{
		if let Err(err) = file {
			if verbose {
				println!("ERROR: {}", err);
			}

			continue;
		}
		let file = file?;

		if !file.path().is_file() {
			continue;
		}

		let relpath = {
			let raw_relpath = file.path().strip_prefix(&source_dir).unwrap();

			let mut reg = Handlebars::new();
			reg.register_escape_fn(handlebars::no_escape);
			let result = reg.render_template(&raw_relpath.to_str().unwrap(), &template_data)?;

			result
		};
		let input_file = file.path();
		let output_file = PathBuf::from(&target_dir).join(&relpath);
		println!("writing to: {}", &output_file.display());

		let input_text = fs::read_to_string(input_file)?;
		let output_text = {
			let include_resource = |h: &Helper,
			                        _: &Handlebars,
			                        _: &Context,
			                        _: &mut RenderContext,
			                        out: &mut dyn Output|
			 -> Result<(), RenderError> {
				// get parameter from helper or throw an error
				let param = h.param(0).ok_or(handlebars::RenderError::new("A"))?;
				&output_file;
				write!(out, "{} pts", param.value().render())?;
				Ok(())
			};

			let mut reg = Handlebars::new();
			reg.register_escape_fn(handlebars::no_escape);
			reg.register_helper("include_file", Box::new(include_file));
			reg.register_helper("include_resource", Box::new(include_resource));

			let output_text = reg.render_template(
				&input_text,
				&json!({
					"project_name": &source_dir.as_path().file_name().unwrap().to_owned().to_string_lossy()
				}),
			);

			match output_text {
				Err(err) => {
					println!("{}", err.desc);
					return Ok(());
				}
				Ok(val) => val,
			}
			// output_text
		};

		fs::create_dir_all(output_file.parent().unwrap())
			.expect("Failed to create parent directories");
		fs::write(&output_file, output_text)?;
	}

	Ok(())
}
