use allay_base::{config::*, data::AllayData, file};
use reqwest::blocking::get;
use std::sync::Arc;

pub fn plugin(command: &PluginCommand) -> anyhow::Result<()> {
    match command {
        PluginCommand::Update(args) => update(args),
    }
}

const PLUGIN: &str = "plugins";
const PATH: &str = "path";
const URL: &str = "url";

fn update(args: &PluginUpdateArgs) -> anyhow::Result<()> {
    let Some(meta) = get_site_config().get(PLUGIN).cloned() else {
        eprintln!("No plugins found in the site configuration.");
        return Ok(());
    };

    let Ok(meta) = meta.as_obj() else {
        eprintln!("plugins configuration must be an object.");
        return Ok(());
    };

    if let Some(name) = &args.name {
        if let Some(info) = meta.get(name) {
            update_on(name, info)?;
        } else {
            eprintln!("No plugin named '{}' found in configuration.", name);
        }
    } else {
        for (name, info) in meta.iter() {
            update_on(name, info)?;
        }
    }

    Ok(())
}

fn update_on(name: &str, info: &Arc<AllayData>) -> anyhow::Result<()> {
    let Ok(info) = info.as_obj() else {
        eprintln!("Plugin '{}' configuration must be an object.", name);
        return Ok(());
    };

    let dir = file::workspace(&get_allay_config().plugin_dir);

    if let Some(path) = info.get(PATH)
        && let Ok(path) = path.as_str()
    {
        println!("Updating plugins in path: {}", path);
        file::copy(path.into(), dir)?;
    } else if let Some(url) = info.get(URL)
        && let Ok(url) = url.as_str()
    {
        println!("Downloading plugin '{}' from URL: {}", name, url);
        let response = get(url)?;
        let status = response.status();
        if status.is_success() {
            let bytes = response.bytes()?;
            let plugin_path = dir.join(format!("{}.wasm", name));
            file::write_file(&plugin_path, &bytes)?;
            println!(
                "Plugin '{}' updated successfully at {:?}",
                name, plugin_path
            );
        } else {
            eprintln!(
                "Failed to download plugin '{}' from URL: {}. Status: {}",
                name, url, status
            );
        }
    } else {
        eprintln!("No plugin source (path or url) found in configuration.");
    }
    Ok(())
}
