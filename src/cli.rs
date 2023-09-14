use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
	#[command(subcommand)]
	pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
	Apply {
		target_dir: String,

		#[arg(short = 'n', long)]
		template_name: Option<String>,

		#[arg(short, long, default_value_t = false)]
		watch: bool,
	},
	Edit {
		template_name: Option<String>,
	},
	List {},
}
