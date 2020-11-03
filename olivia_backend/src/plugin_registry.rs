use olivia_lib::plugin_factory;
use olivia_lib::plugin_factory::PluginFactory;

pub fn new_plugin_factory() -> PluginFactory {
    let mut factory = PluginFactory::new();
    if let Err(e) = factory.register(Box::new(SilencePluginBuilder)) {
        warn!("Failed to register plugin: {:?}", e);
    };
    factory
}

struct SilencePluginBuilder;

impl plugin_factory::PluginBuilder for SilencePluginBuilder {
    fn metadata(&self) -> plugin_factory::PluginMetadata {
        plugin_factory::PluginMetadata {
            id: "builtin_silence".to_string(),
            display_name: "Empty".to_string(),
        }
    }

    fn build(&self) -> Box<dyn olivia_core::plugin::PluginInstance> {
        Box::new(olivia_core::example_plugin::Silence)
    }
}
