use handlebars::Handlebars;
use serde_json::json;
use std::{
	fs,
	path::{Path, PathBuf},
	process,
};
use text_io::read;

use anyhow::Error;
use ignore::{DirEntry, Walk, WalkBuilder};
use walkdir::WalkDir;

pub fn run(source_dir: PathBuf, target_dir: PathBuf) -> Result<(), Error> {
	println!("{}", source_dir.display());

	let is_empty = target_dir.read_dir()?.next().is_none();
	if !is_empty {
		eprintln!("Target directory is not empty: {}", target_dir.display());
		eprintln!("Are you sure you want to continue? (y/n)");

		let input: String = read!();
		if input != "y" {
			eprintln!("exiting...");
			process::exit(1);
		}
	}

	for result in WalkBuilder::new(&source_dir)
		.hidden(false)
		.parents(true)
		.git_global(false)
		.git_exclude(false)
		.filter_entry(|entry: &DirEntry| {
			if entry.file_name() == ".git" {
				return false;
			}

			return true;
		})
		.build()
	{
		if let Err(err) = result {
			println!("ERROR: {}", err);
			continue;
		}
		let entry = result?;

		if !entry.path().is_file() {
			continue;
		}

		let relpath = entry.path().strip_prefix(&source_dir).unwrap();
		let source_file = entry.path();
		let target_file = PathBuf::from(&target_dir).join(&relpath);

		// println!("copy:");
		// println!("  src: {}", source_file.display());
		// println!("  dest: {}", target_file.display());

		let input_text = fs::read_to_string(source_file)?;
		let mut reg = Handlebars::new();
		let output_text = reg
			.render_template(
				&input_text,
				&json!({
					"name": "foo"
				}),
			)
			.unwrap();
		fs::create_dir_all(target_file.parent().unwrap())
			.expect("Failed to create parent directories");
		fs::write(&target_file, output_text)?;

		println!("writing to: {}", &target_file.display());
	}

	Ok(())
}
