use std::path::PathBuf;

use allay_base::config::{CLICommand, get_cli_config};
use allay_base::sitemap::SiteMap;
use lol_html::{RewriteStrSettings, element, rewrite_str};
use tracing::warn;

macro_rules! link_handler {
    ($selector: expr, $attr: expr) => {{
        let base_url = SiteMap::read().base_url.clone();

        element!($selector, move |el| {
            let link = el.get_attribute($attr).expect("attribute was required");
            if let Some(stripped) = link.strip_prefix("/") {
                let link = PathBuf::from(&base_url).join(stripped).to_string_lossy().to_string();
                el.set_attribute($attr, &link)?;
            }
            Ok(())
        })
    }};
}

pub fn postprocess(html: &str) -> String {
    let hot_reload = matches!(get_cli_config().command, CLICommand::Serve(_))
        .then_some(include_str!("assets/auto-reload.js"))
        .unwrap_or_default();

    let html = format!(include_str!("assets/wrapper.html"), html, hot_reload);

    let settings = RewriteStrSettings::new()
        .append_element_content_handler(link_handler!("a[href]", "href"))
        .append_element_content_handler(link_handler!("link[href]", "href"))
        .append_element_content_handler(link_handler!("script[src]", "src"))
        .append_element_content_handler(link_handler!("img[src]", "src"))
        .append_element_content_handler(link_handler!("source[src]", "src"))
        .append_element_content_handler(link_handler!("video[src]", "src"))
        .append_element_content_handler(link_handler!("audio[src]", "src"));

    match rewrite_str(&html, settings) {
        Ok(output) => output,
        Err(_) => {
            warn!("Post-processing of HTML failed, returning unprocessed HTML.");
            html
        }
    }
}
