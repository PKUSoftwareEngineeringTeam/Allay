use allay_plugin_api::{Plugin, SortPageComponent, register_plugin};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct PageSorter {
    // key: json string, value: name of the page
    cache: Mutex<HashMap<String, Arc<String>>>,
}

impl PageSorter {
    fn get_title(&self, page_meta: String) -> Arc<String> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(title) = cache.get(&page_meta) {
            title.clone()
        } else {
            let value: Value = serde_json::from_str(&page_meta).unwrap();
            let title = Arc::new(value["title"].as_str().unwrap_or_default().to_string());
            cache.insert(page_meta.clone(), title.clone());
            title
        }
    }
}

impl SortPageComponent for PageSorter {
    fn enabled(&self) -> bool {
        true
    }

    fn get_sort_order(&self, page_meta1: String, page_meta2: String) -> Ordering {
        let title1 = self.get_title(page_meta1);
        let title2 = self.get_title(page_meta2);
        title1.cmp(&title2)
    }
}

#[derive(Default)]
struct SortPagePlugin {
    sorter: PageSorter,
}

impl Plugin for SortPagePlugin {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "sort-page"
    }
    fn version() -> &'static str
    where
        Self: Sized,
    {
        "0.1.0"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn sort_page_component(&self) -> &dyn SortPageComponent {
        &self.sorter
    }
}

register_plugin!(SortPagePlugin);
