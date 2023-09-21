use clap::Parser;
use dialoguer::Confirm;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	fs,
	path::{self, Path, PathBuf},
	process,
	str::FromStr,
};

mod cli;
mod commands;
mod config;
mod util;

use cli::{Cli, Cmd};
use commands::{apply, applygit};

fn main() {
	let cli = Cli::parse();

	match &cli.cmd {
		Cmd::Apply {
			target_dir,
			template_name,
			force,
			watch,
		} => {
			let config = config::get_config().expect("Failed to get config");

			if !PathBuf::from(target_dir).exists() {
				eprintln!("Target directory does not exist: {}", target_dir);
				process::exit(1);
			}

			let source_dir = match template_name {
				Some(name) => {
					let p = PathBuf::from(config.templates_dir)
						.join("templates")
						.join(name);

					if !p.exists() {
						eprintln!("Template name could not be found: '{}'", name);
						process::exit(1);
					}

					p.clone()
				}
				None => {
					let p = PathBuf::from(&config.templates_dir).join("templates");
					if !p.exists() {
						eprintln!("Template subdir could not be found: '{}'", p.display());
						process::exit(1);
					}

					let names = p
						.read_dir()
						.unwrap()
						.filter_map(|e| e.ok())
						.map(|e| e.path())
						.map(|e| e.file_name().unwrap().to_owned())
						.map(|e| e.to_str().unwrap().to_string())
						.collect::<Vec<_>>();

					let name = util::get_template_name(names).expect("Failed to get template name");

					let p = PathBuf::from(&config.templates_dir)
						.join("templates")
						.join(name);

					p.clone()
				}
			};

			if !force {
				let is_empty = PathBuf::from(target_dir)
					.read_dir()
					.unwrap()
					.next()
					.is_none();
				if !is_empty {
					eprintln!("Target directory is not empty: {}", &target_dir);

					let input = Confirm::new()
						.with_prompt("Do you want to continue?")
						.default(false)
						.show_default(false)
						.interact()
						.unwrap();
					if input {
						eprintln!("exiting...");
						process::exit(1);
					}
				}
			}

			apply::run(source_dir, PathBuf::from(target_dir)).expect("Failed to run templater");
		}
		Cmd::ApplyAll {} => {
			applygit::run().unwrap();
		}
		Cmd::Edit { template_name } => {
			// run_template.command_new(template_name.clone()).unwrap();
		}
		Cmd::List {} => {
			// run_template.list().unwrap();
		}
	};
}
