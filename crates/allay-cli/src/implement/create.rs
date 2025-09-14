use allay_base::{config::*, file};
use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{FetchOptions, RemoteCallbacks, build::RepoBuilder};
use std::path::Path;
use tracing::{info, instrument};

pub fn init(_args: &InitArgs) -> anyhow::Result<()> {
    new(&NewArgs { dir: ".".into() })
}

#[instrument(name = "initializing the site", skip(_args))]
pub fn new(_args: &NewArgs) -> anyhow::Result<()> {
    file::create_dir_recursively(file::root())?;

    let dirs = [
        &CONTENT_CONFIG.dir,
        &PUBLISH_CONFIG.dir,
        &TEMPLATE_CONFIG.dir,
        &THEME_CONFIG.dir,
        &STATIC_CONFIG.dir,
    ];

    for dir_name in dirs {
        file::create_dir(file::workspace(dir_name))?;
    }

    file::write_file(file::workspace(SITE_CONFIG_FILE), DEFAULT_SITE_CONFIG)?;

    if !THEME_CONFIG.default.repository.is_empty() {
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
    let theme_url = &THEME_CONFIG.default.repository;

    let should_clone = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Do you want to clone the default theme from {}?",
            theme_url
        ))
        .default(true)
        .interact()?;

    if should_clone {
        clone_default_theme()?;
    } else {
        println!("⚠️  Skipping theme cloning as per user choice.");
    }

    Ok(())
}

fn clone_default_theme() -> anyhow::Result<()> {
    let theme_url = &THEME_CONFIG.default.repository;

    if theme_url.is_empty() {
        println!("⚠️  No default theme URL configured");
        return Ok(());
    }

    let target_dir = file::workspace(&THEME_CONFIG.dir).join(&THEME_CONFIG.default.name);

    if target_dir.exists() {
        println!("⚠️  Theme directory already exists at: {:?}", target_dir);
        return Ok(());
    }

    println!("🌐 Cloning theme from: {}", theme_url);

    match clone_repository_with_progress(theme_url, &target_dir) {
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
