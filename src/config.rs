use anyhow::Result;

pub struct Config {
	pub templates_dir: String,
}

pub fn get_config() -> Result<Config> {
	Ok(Config {
		templates_dir: "/storage/ur/storage_home/Docs/Programming/Repositories/hyperupcall/templates"
			.to_string(),
	})
}
