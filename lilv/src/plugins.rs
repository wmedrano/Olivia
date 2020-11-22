use crate::node::Node;
use crate::plugin::Plugin;
use lilv_sys as lib;
use std::ptr::NonNull;
use std::sync::Arc;

/// Contains a list of plugins.
///
/// The list contains enough references for each query, but the plugins are lazily loaded into
/// memory as needed and remain cached.
pub struct Plugins {
    pub(crate) world: Arc<crate::InnerWorld>,
    pub(crate) ptr: NonNull<lib::LilvPlugins>,
}

/// An iterator over plugins.
pub struct PluginsIter<'a> {
    plugins: &'a Plugins,
    iter: *mut lib::LilvIter,
}

impl Plugins {
    /// Returns the number of plugins.
    pub fn size(&self) -> usize {
        let _ = self.world.inner.read();
        let size = unsafe { lib::lilv_plugins_size(self.as_ptr()) };
        size as usize
    }

    /// Returns the plugin by uri or None if it does not exist.
    pub fn get_by_uri(&self, uri: &Node) -> Option<Plugin> {
        let _ = self.world.inner.read();
        let uri_ptr = uri.inner.read().as_ptr();

        let plugin_ptr = unsafe { lib::lilv_plugins_get_by_uri(self.as_ptr(), uri_ptr) };
        Some(Plugin::new_borrowed(
            NonNull::new(plugin_ptr as *mut lib::LilvPlugin)?,
            self.world.clone(),
        ))
    }

    // Returns an iterator over all plugins.
    pub fn iter(&self) -> PluginsIter<'_> {
        let _ = self.world.inner.read();
        let iter = unsafe { lib::lilv_plugins_begin(self.as_ptr()) };
        PluginsIter {
            plugins: self,
            iter,
        }
    }

    /// Returns the underlying Lilv pointer.
    pub fn as_ptr(&self) -> *const lib::LilvPlugins {
        self.ptr.as_ptr()
    }
}

impl<'a> Iterator for PluginsIter<'a> {
    type Item = Plugin;

    fn next(&mut self) -> Option<Plugin> {
        let _ = self.plugins.world.inner.read();
        if unsafe { lib::lilv_plugins_is_end(self.plugins.as_ptr(), self.iter) } {
            return None;
        }
        let plugin_ptr = unsafe { lib::lilv_plugins_get(self.plugins.as_ptr(), self.iter) };
        self.iter = unsafe { lib::lilv_plugins_next(self.plugins.as_ptr(), self.iter) };
        let p = Plugin::new_borrowed(
            NonNull::new(plugin_ptr as *mut _).unwrap(),
            self.plugins.world.clone(),
        );
        Some(p)
    }
}
