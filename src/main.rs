use std::{path::PathBuf, process};

use clap::Parser;

mod cli;
mod commands;
mod config;
mod util;

use cli::{Cli, Cmd};
use commands::apply;

fn main() {
	let cli = Cli::parse();

	match &cli.cmd {
		Cmd::Apply {
			target_dir,
			template_name,
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

			apply::run(source_dir, PathBuf::from(target_dir)).expect("Failed to run templater");
		}
		Cmd::Edit { template_name } => {
			// run_template.command_new(template_name.clone()).unwrap();
		}
		Cmd::List {} => {
			// run_template.list().unwrap();
		}
	};
}
