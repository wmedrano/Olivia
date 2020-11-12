use crate::plugin_factory::{PluginBuilder, PluginBuilderError::GenericError, PluginMetadata};

pub struct LV2PluginBuilder {
    common_nodes: std::sync::Arc<CommonNodes>,
    plugin: lilv::Plugin,
    metadata: PluginMetadata,
}

impl PluginBuilder for LV2PluginBuilder {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn build(
        &self,
    ) -> Result<
        Box<dyn olivia_core::plugin::PluginInstance>,
        crate::plugin_factory::PluginBuilderError,
    > {
        let mut instance = unsafe {
            self.plugin
                .instantiate(44100.0, std::ptr::null())
                .ok_or(GenericError("Failed to instantiate LV2 plugin."))?
        };

        let mut audio_output_ports = self
            .plugin
            .ports()
            .enumerate()
            .filter(|(_, p)| self.common_nodes.is_audio(p) && self.common_nodes.is_output(p))
            .map(|(port_index, _)| port_index);

        instance.activate();
        Ok(Box::new(LV2PluginInstance {
            instance,
            out_left_port: audio_output_ports.next(),
            out_right_port: audio_output_ports.next(),
            other_output_ports: audio_output_ports.collect(),
        }))
    }
}

#[derive(Debug)]
pub struct LV2PluginInstance {
    instance: lilv::Instance,
    out_left_port: Option<usize>,
    out_right_port: Option<usize>,
    other_output_ports: Vec<usize>,
}

impl olivia_core::plugin::PluginInstance for LV2PluginInstance {
    fn process(
        &mut self,
        _midi: &[olivia_core::TimedMidi],
        out_left: &mut [f32],
        out_right: &mut [f32],
    ) {
        let samples = out_left.len().min(out_right.len());
        if let Some(p) = self.out_left_port {
            unsafe { self.instance.connect_port(p, out_left.as_mut_ptr()) };
        }
        if let Some(p) = self.out_right_port {
            unsafe { self.instance.connect_port(p, out_right.as_mut_ptr()) };
        }
        for p in self.other_output_ports.iter() {
            unsafe {
                self.instance
                    .connect_port(*p, std::ptr::null_mut() as *mut f32)
            };
        }
        self.instance.run(samples);
    }
}

pub fn load_plugins() -> Vec<LV2PluginBuilder> {
    let w = lilv::World::with_load_all();
    let mut plugin_builders = Vec::new();
    let common_nodes = std::sync::Arc::new(CommonNodes {
        output_port: w.new_uri("http://lv2plug.in/ns/lv2core#OutputPort"),
        audio_port: w.new_uri("http://lv2plug.in/ns/lv2core#AudioPort"),
    });
    for plugin in w.all_plugins().iter() {
        if plugin.uri().as_uri().is_none() {
            warn!("Could not get uri from {:?}.", plugin.uri().turtle_token());
            continue;
        }
        let metadata = PluginMetadata {
            // TODO(wmedrano): Fix the URI.
            id: format!("lv2_{}", plugin.uri().as_uri().unwrap()),
            display_name: plugin
                .name()
                .as_str()
                .unwrap_or(plugin.uri().as_uri().unwrap())
                .to_string(),
        };
        let builder = LV2PluginBuilder {
            common_nodes: common_nodes.clone(),
            metadata,
            plugin,
        };
        plugin_builders.push(builder);
    }
    plugin_builders
}

struct CommonNodes {
    output_port: lilv::Node,
    audio_port: lilv::Node,
}

impl CommonNodes {
    fn is_audio(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.audio_port)
    }

    fn is_output(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.output_port)
    }
}
