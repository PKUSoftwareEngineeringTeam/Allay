use allay_base::{config::*, file};
use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{FetchOptions, RemoteCallbacks, build::RepoBuilder};
use std::path::Path;
use tracing::{info, instrument};

/// CLI Init Command
pub fn init(args: &InitArgs) -> anyhow::Result<()> {
    new(&NewArgs {
        dir: ".".into(),
        skip_theme: args.skip_theme,
    })
}

/// CLI New Command
#[instrument(name = "initializing the site", skip(args))]
pub fn new(args: &NewArgs) -> anyhow::Result<()> {
    file::create_dir_recursively(file::root())?;
    let config = get_allay_config();

    let dirs = [
        &config.content_dir,
        &config.publish_dir,
        &config.theme_dir,
        &config.statics_dir,
    ];

    for dir_name in dirs {
        file::create_dir(file::workspace(dir_name))?;
    }

    file::write_file(file::workspace(SITE_CONFIG_FILE), DEFAULT_SITE_CONFIG)?;

    if !args.skip_theme {
        ask_to_clone_default_theme()?;
    }

    println!("✅ Site initialized successfully!");
    info!(
        "Site initialized successfully in '{}'",
        file::root().display()
    );
    Ok(())
}

fn ask_to_clone_default_theme() -> anyhow::Result<()> {
    const THEME_URL: &str = "https://github.com/PKUSoftwareEngineeringTeam/Axolotl.git";

    let should_clone = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to clone the default theme from {}?",
            THEME_URL
        ))
        .default(true)
        .interact()?;

    if should_clone {
        clone_default_theme(THEME_URL)?;
    } else {
        println!("⚠️  Skipping theme cloning as per user choice.");
    }

    Ok(())
}

fn clone_default_theme(url: &str) -> anyhow::Result<()> {
    const THEME_NAME: &str = "Axolotl";

    let target_dir = file::workspace(&get_allay_config().theme_dir).join(THEME_NAME);

    if file::dir_exists(&target_dir) {
        println!("⚠️  Theme directory already exists at: {:?}", target_dir);
        return Ok(());
    }

    println!("🌐 Cloning theme from: {}", url);

    match clone_repository_with_progress(url, &target_dir) {
        Ok(_) => {
            println!("✅ Default theme cloned successfully to: {:?}", target_dir);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to clone theme: {}", e);
            eprintln!("💡 Please check your network connection and try again later.");
            Err(e)
        }
    }
}

fn clone_repository_with_progress(url: &str, into: &Path) -> anyhow::Result<()> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|progress| {
        if progress.received_objects() == progress.total_objects() {
            println!(
                "Resolving deltas {}/{}",
                progress.indexed_deltas(),
                progress.total_deltas()
            );
        } else if progress.total_objects() > 0 {
            println!(
                "Receiving objects: {}% ({}/{})",
                progress.received_objects() * 100 / progress.total_objects(),
                progress.received_objects(),
                progress.total_objects()
            );
        }
        true
    });

    info!("Cloning theme repository from {} to {:?}", url, into);

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    RepoBuilder::new().fetch_options(fetch_options).clone(url, into)?;
    Ok(())
}
