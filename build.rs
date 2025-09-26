use std::error::Error;

use license_fetcher::PackageList;
use license_fetcher::build::config::{Config, ConfigBuilder};
use license_fetcher::build::package_list_with_licenses;

fn fetch_and_embed_licenses() -> Result<(), Box<dyn Error>> {
    // Config with environment variables set by cargo, to fetch licenses at build time.
    let config: Config = ConfigBuilder::from_build_env().build()?;

    let packages: PackageList = package_list_with_licenses(config)?;

    // Write packages to out dir to be embedded.
    packages.write_package_list_to_out_dir()?;

    Ok(())
}

// Create empty dummy file so that the embedding does not fail.
fn dummy_file() {
    let mut path = std::env::var_os("OUT_DIR")
        .expect("Creation of dummy file failed: Environment variable 'OUT_DIR' not set.");
    path.push("/LICENSE-3RD-PARTY.bincode.deflate");
    let _ = std::fs::File::create(path).expect("Creation of dummy file failed: Write failed.");
}

fn main() {
    if let Some(mode) = std::env::var_os("LICENSE_FETCHER") {
        match mode.to_ascii_lowercase().to_string_lossy().as_ref() {
            "production" => fetch_and_embed_licenses().unwrap(),
            "development" => {
                eprintln!("Skipping license fetching.");
                dummy_file();
            }
            &_ => {
                eprintln!("Wrong environment variable `LICENSE_FETCHER`!");
                eprintln!("Expected either ``, `production` or `development`.");

                dummy_file();
            }
        }
    } else {
        if let Err(err) = fetch_and_embed_licenses() {
            eprintln!("An error occurred during license fetch:\n\n");
            eprintln!("{:?}", err);

            dummy_file();
        }
    }

    // Rerun only if one of the following files changed:
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.lock");
    println!("cargo::rerun-if-changed=Cargo.toml");
}
