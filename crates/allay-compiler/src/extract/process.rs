use crate::magic;
use allay_base::config::get_allay_config;
use allay_base::data::{AllayData, AllayObject};
use allay_base::file;
use allay_base::template::TemplateKind;
use allay_base::url::AllayUrlPath;
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
use std::path::Path;
use std::sync::Arc;

/// A hook to allow plugins to process the content before compilation
#[cfg(feature = "plugin")]
pub fn before_compile(content: String, kind: TemplateKind) -> String {
    let plugin_manager = PluginManager::instance();
    plugin_manager.plugins().iter().fold(content, |content, plugin| {
        let mut plugin = plugin.lock().expect("Plugin lock poisoned!");
        plugin.before_compile(content, kind.clone())
    })
}

/// A preprocessing step to add default metadata fields
pub fn meta_preprocess<P: AsRef<Path>>(source: P, mut meta: AllayObject) -> AllayObject {
    meta.entry(magic::URL.into()).or_insert_with(|| {
        // Add the `url` field to the metadata
        let entry =
            match source.as_ref().strip_prefix(file::workspace(&get_allay_config().content_dir)) {
                Ok(e) => e.with_extension(TemplateKind::Html.extension()),
                // ignore if the file is not under the content directory
                Err(_) => return Arc::new(AllayData::default()),
            };
        let url = AllayUrlPath::from(entry).as_ref().to_string_lossy().to_string();
        Arc::new(url.into())
    });
    meta
}
