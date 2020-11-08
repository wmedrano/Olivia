use olivia_core::plugin::PluginInstance;
use plugin_factory::PluginBuilder;

use crate::plugin_factory;
use crate::plugin_factory::PluginFactory;

pub fn new_plugin_factory() -> PluginFactory {
    let mut factory = PluginFactory::new();
    if let Err(e) = factory.register(CloneablePluginBuilder::new(
        plugin_factory::PluginMetadata {
            id: "builtin_silence".to_string(),
            display_name: "Empty".to_string(),
        },
        olivia_core::example_plugin::Silence,
    )) {
        warn!("Failed to register plugin: {:?}", e);
    };
    if let Err(e) = factory.register(CloneablePluginBuilder::new(
        plugin_factory::PluginMetadata {
            id: "builtin_sine".to_string(),
            display_name: "Sine".to_string(),
        },
        // We are assuming a 44100 sample rate which may not be true.
        // The right thing to do is to create a proper builder that
        // takes sample rate as a parameter when constructing.
        olivia_core::example_plugin::Sine::new(44100.0),
    )) {
        warn!("Failed to register plugin: {:?}", e);
    };

    factory
}

struct CloneablePluginBuilder<P> {
    metadata: plugin_factory::PluginMetadata,
    plugin_instance: P,
}

impl<P> CloneablePluginBuilder<P> {
    fn new(
        metadata: plugin_factory::PluginMetadata,
        plugin_instance: P,
    ) -> CloneablePluginBuilder<P> {
        CloneablePluginBuilder {
            metadata,
            plugin_instance,
        }
    }
}

impl<P> PluginBuilder for CloneablePluginBuilder<P>
where
    P: 'static + Clone + PluginInstance,
{
    fn metadata(&self) -> plugin_factory::PluginMetadata {
        self.metadata.clone()
    }

    fn build(&self) -> Result<Box<dyn PluginInstance>, plugin_factory::PluginBuilderError> {
        Ok(Box::new(self.plugin_instance.clone()))
    }
}
