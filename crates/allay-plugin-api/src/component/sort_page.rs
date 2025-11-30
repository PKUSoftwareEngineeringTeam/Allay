use super::PluginGuest;
use super::wit::sort_page;

use crate::plugin;
use std::cmp::Ordering;

/// Component for sorting pages based on their metadata.
pub trait SortPageComponent {
    /// Whether the component is enabled or not.
    fn enabled(&self) -> bool {
        false
    }

    /// Returns the sort order of the page based on its metadata. Allay will sort pages based on the
    /// returned sort order.
    ///
    /// # Arguments
    /// * `page_meta1` and `page_meta2` - The JSON strings of the page metadata.
    ///
    /// # Returns
    /// An `Ordering` value indicating the sort order of the two pages.
    /// * `Ordering::Less` if `page_meta1` should come before `page_meta2` in the sorted list.
    /// * `Ordering::Greater` if `page_meta2` should come before `page_meta1` in the sorted list.
    /// * `Ordering::Equal` if the order of `page_meta1` and `page_meta2` does not matter.
    fn get_sort_order(&self, _page_meta1: String, _page_meta2: String) -> Ordering {
        Ordering::Equal
    }
}

impl sort_page::Guest for PluginGuest {
    fn sort_enabled() -> bool {
        plugin().sort_page_component().enabled()
    }

    fn get_sort_order(page_meta1: String, page_meta2: String) -> i8 {
        plugin().sort_page_component().get_sort_order(page_meta1, page_meta2) as i8
    }
}
