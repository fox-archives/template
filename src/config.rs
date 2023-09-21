use anyhow::Result;

pub struct Config {
	pub templates_dir: String,
	pub variables: Variables,
}

pub struct Variables {
	pub full_name: String,
	pub license: String,
}

pub fn get_config() -> Result<Config> {
	Ok(Config {
		templates_dir: "/storage/ur/storage_home/Docs/Programming/Repositories/hyperupcall/templates"
			.to_string(),
		variables: Variables {
			full_name: "Edwin Kofler".to_string(),
			license: "MPL-2.0".to_string(),
		},
	})
}
