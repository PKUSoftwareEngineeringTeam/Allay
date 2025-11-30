use crate::PluginHost;
use std::cmp::Ordering;

fn to_ordering(order: i8) -> Ordering {
    if order < 0 {
        Ordering::Less
    } else if order > 0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

impl PluginHost {
    pub fn sort_enabled(&mut self) -> wasmtime::Result<bool> {
        self.plugin.allay_plugin_sort_page().call_sort_enabled(&mut self.store)
    }

    pub fn get_sort_order(
        &mut self,
        page_meta1: &str,
        page_meta2: &str,
    ) -> wasmtime::Result<Ordering> {
        self.plugin
            .allay_plugin_sort_page()
            .call_get_sort_order(&mut self.store, page_meta1, page_meta2)
            .map(to_ordering)
    }
}
