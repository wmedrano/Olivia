use crate::instance::{Instance, InstanceImpl};
use crate::node::Node;
use crate::nodes::Nodes;
use crate::plugin_class::PluginClass;
use crate::port::{Port, PortsIter};
use crate::uis::UIs;
use crate::world::InnerWorld;
use lilv_sys as lib;
use parking_lot::RwLock;
use std::ptr::NonNull;
use std::sync::Arc;

unsafe impl Send for Plugin {}
unsafe impl Sync for Plugin {}

/// Metadata and instatiation logic for an LV2 plugin.
pub struct Plugin {
    pub(crate) inner: RwLock<NonNull<lib::LilvPlugin>>,
    pub(crate) world: Arc<InnerWorld>,
}

impl Plugin {
    pub(crate) fn new_borrowed(ptr: NonNull<lib::LilvPlugin>, world: Arc<InnerWorld>) -> Self {
        Self {
            inner: RwLock::new(ptr),
            world,
        }
    }

    /// Check if plugin is valid. This is not a rigorous validator, but can be used to reject some
    /// malformed plugins that could cause bugs (e.g. plugins with missing required fields).
    ///
    /// Note that normal hosts do NOT need to use this as lilv does not load invalid plugins into
    /// plugin lists. This is included for plugin testing utilities, etc.
    pub fn verify(&self) -> bool {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        unsafe { lib::lilv_plugin_verify(plugin.as_ptr()) }
    }

    /// Returns the URI of the plugin.
    ///
    /// Any serialization that refers to plugins should refer to them by this.
    /// Hosts SHOULD NOT save any filesystem paths, plugin indexes, etc. in saved
    /// files; save only the URI.
    ///
    /// The URI is a globally unique identifier for one specific plugin.  Two
    /// plugins with the same URI are compatible in port signature, and should
    /// be guaranteed to work in a compatible and consistent way.  If a plugin
    /// is upgraded in an incompatible way (eg if it has different ports), it
    /// MUST have a different URI than it's predecessor.
    pub fn uri(&self) -> Node {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_uri(plugin.as_ptr()) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// Get the (resolvable) URI of the plugin's "main" bundle.
    /// This returns the URI of the bundle where the plugin itself was found.  Note
    /// that the data for a plugin may be spread over many bundles, that is,
    /// data_uris may return URIs which are not within this bundle.
    ///
    /// Typical hosts should not need to use this function.
    /// Note this always returns a fully qualified URI.  If you want a local
    /// filesystem path, use lilv_file_uri_parse() (not yet implemented).
    pub fn bundle_uri(&self) -> Node {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_bundle_uri(plugin.as_ptr()) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// Get the (resolvable) URIs of the RDF data files that define a plugin.
    /// Typical hosts should not need to use this function.
    ///
    /// Note this always returns fully qualified URIs.  If you want local
    /// filesystem paths, use lilv_file_uri_parse() (not yet implemented).
    pub fn data_uris(&self) -> Nodes {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Nodes::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_data_uris(plugin.as_ptr()) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// Get the (resolvable) URI of the shared library for `plugin`.
    /// Note this always returns a fully qualified URI.  If you want a local
    /// filesystem path, use lilv_file_uri_parse() (not yet implemented).
    pub fn library_uri(&self) -> Option<Node> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Node::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_library_uri(plugin.as_ptr()) as _ })?,
            self.world.clone(),
        ))
    }

    /// Get the name of `plugin`.
    ///
    /// This returns the name (doap:name) of the plugin. The name may be
    /// translated according to the current locale, this value MUST NOT be used
    /// as a plugin identifier (use the URI for that).
    ///
    /// # Panics
    /// May panic if `verify()` returns false.
    pub fn name(&self) -> Node {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_name(plugin.as_ptr()) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// Get the class this plugin belongs to (e.g. Filters).
    pub fn class(&self) -> PluginClass {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        PluginClass::new_borrowed(
            NonNull::new(unsafe { lib::lilv_plugin_get_class(plugin.as_ptr()) as _ }).unwrap(),
            self.world.clone(),
        )
    }

    /// Get a value associated with the plugin in a plugin's data files.
    /// `predicate` must be either a URI or a QName.
    ///
    /// Returns the object of all triples found of the form:
    ///
    /// `<plugin-uri> predicate ?object`
    ///
    /// May return None if the property was not found, or if object(s) is not
    /// sensibly represented as a LilvNodes (e.g. blank nodes).
    pub fn value(&self, predicate: &Node) -> Option<Nodes> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        let predicate = predicate.inner.read();

        Some(Nodes::new(
            NonNull::new(unsafe {
                lib::lilv_plugin_get_value(plugin.as_ptr(), predicate.as_ptr())
            })?,
            self.world.clone(),
        ))
    }

    /// Return whether a feature is supported by a plugin.
    /// This will return true if the feature is an optional or required feature
    /// of the plugin.
    pub fn has_feature(&self, feature_uri: &Node) -> bool {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        let feature_uri = feature_uri.inner.read();

        unsafe { lib::lilv_plugin_has_feature(plugin.as_ptr(), feature_uri.as_ptr()) }
    }

    /// Get the LV2 Features supported (required or optionally) by a plugin.
    /// A feature is "supported" by a plugin if it is required OR optional.
    ///
    /// Since required features have special rules the host must obey, this function
    /// probably shouldn't be used by normal hosts.  Using required_features and optional_features
    /// separately is best in most cases.
    pub fn supported_features(&self) -> Option<Nodes> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_supported_features(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Get the LV2 Features required by a plugin.
    /// If a feature is required by a plugin, hosts MUST NOT use the plugin if they do not
    /// understand (or are unable to support) that feature.
    ///
    /// All values returned here MUST be passed to the plugin's instantiate method
    /// (along with data, if necessary, as defined by the feature specification)
    /// or plugin instantiation will fail.
    pub fn required_features(&self) -> Option<Nodes> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_required_features(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Get the LV2 Features optionally supported by a plugin.
    /// Hosts MAY ignore optional plugin features for whatever reasons. Plugins
    /// MUST operate (at least somewhat) if they are instantiated without being
    /// passed optional features.
    pub fn optional_features(&self) -> Option<Nodes> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_optional_features(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Return whether or not a plugin provides a specific extension data.
    pub fn has_extension_data(&self, uri: &Node) -> bool {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        let uri = uri.inner.read();

        unsafe { lib::lilv_plugin_has_extension_data(plugin.as_ptr(), uri.as_ptr()) }
    }

    /// Get a sequence of all extension data provided by a plugin.
    /// This can be used to find which URIs Instance::get_extension_data()
    /// will return a value for without instantiating the plugin.
    pub fn extension_data(&self) -> Option<Nodes> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_extension_data(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Returns an iterator over all ports in the plugin.
    pub fn ports(&self) -> PortsIter<'_> {
        let _ = self.world.inner.read();
        PortsIter {
            plugin: self,
            port_index: 0,
        }
    }

    /// Returns the number of ports in the plugin.
    pub fn num_ports(&self) -> usize {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        unsafe { lib::lilv_plugin_get_num_ports(plugin.as_ptr()) as _ }
    }

    /// Get the port ranges (minimum, maximum and default values) for all ports.
    /// `min_values`, `max_values` and `def_values` must either point to an slice
    /// of N floats, where N is the value returned by num_ports()
    /// for this plugin, or None.  The elements of the slice will be set to the
    /// the minimum, maximum and default values of the ports on this plugin,
    /// with slice index corresponding to port index.  If a port doesn't have a
    /// minimum, maximum or default value, or the port's type is not float, the
    /// corresponding slice element will be set to NAN.
    ///
    /// This is a convenience method for the common case of getting the range of
    /// all float ports on a plugin, and may be significantly faster than
    /// repeated calls to get range on ports.
    pub fn port_ranges_float<'a, Min, Max, Def>(
        &self,
        min_values: Min,
        max_values: Max,
        def_values: Def,
    ) -> Result<(), ()>
    where
        Min: Into<Option<&'a mut [f32]>>,
        Max: Into<Option<&'a mut [f32]>>,
        Def: Into<Option<&'a mut [f32]>>,
    {
        let min_values = min_values.into();
        let max_values = max_values.into();
        let def_values = def_values.into();

        let (equal_sizes, size) = match (&min_values, &max_values, &def_values) {
            (Some(a), Some(b), None) => (a.len() == b.len(), a.len()),
            (Some(a), None, Some(b)) => (a.len() == b.len(), a.len()),
            (None, Some(a), Some(b)) => (a.len() == b.len(), a.len()),
            (Some(a), Some(b), Some(c)) => (a.len() == b.len() && b.len() == c.len(), a.len()),
            _ => (true, self.num_ports()),
        };

        if !equal_sizes || size != self.num_ports() {
            return Err(());
        }

        let min_ptr = min_values.map_or(std::ptr::null_mut(), |x| x.as_mut_ptr());
        let max_ptr = max_values.map_or(std::ptr::null_mut(), |x| x.as_mut_ptr());
        let def_ptr = def_values.map_or(std::ptr::null_mut(), |x| x.as_mut_ptr());

        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        unsafe {
            lib::lilv_plugin_get_port_ranges_float(plugin.as_ptr(), min_ptr, max_ptr, def_ptr)
        };

        Ok(())
    }

    /// Returns the number of ports that match all the given classes.
    pub fn num_ports_of_class(&self, classes: &[&Node]) -> usize {
        (0..self.num_ports())
            .filter_map(|index| self.port_by_index(index))
            .filter(|port| classes.iter().all(|cls| port.is_a(cls)))
            .count()
    }

    /// Return whether or not the plugin introduces (and reports) latency.
    ///
    /// The index of the latency port can be found with
    /// latency_port_index if this function returns true.
    pub fn has_latency(&self) -> bool {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        unsafe { lib::lilv_plugin_has_latency(plugin.as_ptr()) }
    }

    /// Return the index of the plugin's latency port or None if the plugin does not report latency.
    ///
    /// Any plugin that introduces unwanted latency that should be compensated for
    /// (by hosts with the ability/need) MUST provide this port, which is a control
    /// rate output port that reports the latency for each cycle in frames.
    pub fn latency_port_index(&self) -> Option<usize> {
        if self.has_latency() {
            let _ = self.world.inner.read();
            let plugin = self.inner.read();
            Some(unsafe { lib::lilv_plugin_get_latency_port_index(plugin.as_ptr()) as _ })
        } else {
            None
        }
    }

    /// Get a port on the plugin by index. If the index is out of range, then None is returned.
    pub fn port_by_index(&self, index: usize) -> Option<Port> {
        if index > std::u32::MAX as _ {
            return None;
        }

        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        Some(Port::new_borrowed(
            NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_index(plugin.as_ptr(), index as _) as _
            })?,
            self,
        ))
    }

    /// Get a port on Plugin by `symbol`.
    /// Note this function is slower than Plugin::port_by_index(),
    /// especially on plugins with a very large number of ports.
    pub fn port_by_symbol(&self, symbol: &Node) -> Option<Port> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        let symbol = symbol.inner.read();

        Some(Port::new_borrowed(
            NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_symbol(plugin.as_ptr(), symbol.as_ptr()) as _
            })?,
            self,
        ))
    }

    /// Get a port on the Plugin by its lv2:designation.
    ///
    /// The designation of a port describes the meaning, assignment, allocation or
    /// role of the port, e.g. "left channel" or "gain".  If found, the port with
    /// matching `port_class` and `designation` is be returned, otherwise None is
    /// returned.  The `port_class` can be used to distinguish the input and output
    /// ports for a particular designation.  If `port_class` is None, any port with
    /// the given designation will be returned.
    pub fn port_by_designation(
        &self,
        port_class: Option<&Node>,
        designation: &Node,
    ) -> Option<Port> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        let port_class = port_class.map(|n| n.inner.read());
        let designation = designation.inner.read();

        let port_class_ptr = port_class
            .map(|n| n.as_ptr())
            .unwrap_or_else(std::ptr::null_mut);
        Some(Port::new_borrowed(
            NonNull::new(unsafe {
                lib::lilv_plugin_get_port_by_designation(
                    plugin.as_ptr(),
                    port_class_ptr,
                    designation.as_ptr(),
                ) as _
            })?,
            self,
        ))
    }

    /// Get the project the plugin is a part of.
    ///
    /// More information about the project can be read via `World::find_nodes()`,
    /// typically using properties from DOAP (e.g. doap:name).
    pub fn project(&self) -> Option<Node> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_project(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Get the full name of the plugin's author.
    /// Returns None if author name is not present.
    pub fn author_name(&self) -> Option<Node> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_author_name(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Get the email address of the plugin's author.
    /// Returns None if author email address is not present.
    /// Returned value must be freed by caller.
    pub fn author_email(&self) -> Option<Node> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_author_email(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Get the address of the plugin author's home page.
    /// Returns NULL if author homepage is not present.
    pub fn author_homepage(&self) -> Option<Node> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Node::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_author_homepage(plugin.as_ptr()) })?,
            self.world.clone(),
        ))
    }

    /// Return true iff `plugin` has been replaced by another plugin.
    ///
    /// The plugin will still be usable, but hosts should hide them from their
    /// user interfaces to prevent users from using deprecated plugins.
    pub fn is_replaced(&self) -> bool {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        unsafe { lib::lilv_plugin_is_replaced(plugin.as_ptr()) }
    }

    // MAYBE TODO write_description

    // MAYBE TODO write_manifest_entry

    /// Get the resources related to `plugin` with lv2:appliesTo.
    ///
    /// Some plugin-related resources are not linked directly to the plugin with
    /// rdfs:seeAlso and thus will not be automatically loaded along with the plugin
    /// data (usually for performance reasons).  All such resources of the given @c
    /// type related to `plugin` can be accessed with this function.
    ///
    /// If `type` is None, all such resources will be returned, regardless of type.
    ///
    /// To actually load the data for each returned resource, use
    /// `World::load_resource()`.
    pub fn related(&self, typ: Option<&Node>) -> Option<Nodes> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();
        let typ = typ.map(|n| n.inner.read());
        let typ_ptr = typ.map(|p| p.as_ptr()).unwrap_or_else(std::ptr::null_mut);

        Some(Nodes::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_related(plugin.as_ptr(), typ_ptr) })?,
            self.world.clone(),
        ))
    }

    /// Get all UIs for the Plugin.
    pub fn uis(&self) -> Option<UIs> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(UIs::new(
            NonNull::new(unsafe { lib::lilv_plugin_get_uis(plugin.as_ptr()) })?,
            self,
        ))
    }

    /// Instantiate a plugin.
    /// The returned value is a lightweight handle for an LV2 plugin instance,
    /// it does not refer to the Plugin, or any other Lilv state.
    /// `features` is a NULL-terminated array of features the host supports.
    /// NULL may be passed if the host supports no additional features.
    /// # Safety
    /// May call unsafe ffi code from LV2 plugin.
    /// Will dereference items in features and features must terminate in std::ptr::null().
    pub unsafe fn instantiate(
        &self,
        sample_rate: f64,
        features: *const *const lv2_raw::LV2Feature,
    ) -> Option<Instance> {
        let _ = self.world.inner.read();
        let plugin = self.inner.read();

        Some(Instance {
            inner: NonNull::new(
                lib::lilv_plugin_instantiate(plugin.as_ptr(), sample_rate, features)
                    as *mut InstanceImpl,
            )?,
        })
    }
}
