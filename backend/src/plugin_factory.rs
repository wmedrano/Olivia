use olivia_core::plugin::PluginInstance;
use std::collections::HashMap;

#[derive(Default)]
pub struct PluginFactory {
    builders: HashMap<String, (PluginMetadata, Box<dyn PluginBuilder>)>,
}

impl PluginFactory {
    pub fn new() -> PluginFactory {
        PluginFactory {
            builders: HashMap::new(),
        }
    }

    pub fn register<B: 'static + PluginBuilder>(
        &mut self,
        builder: B,
    ) -> Result<(), PluginRegistrationError> {
        let metadata = builder.metadata();
        if let Err(e) = metadata.validate() {
            return Err(PluginRegistrationError::InvalidMetadata(metadata, e));
        }
        if self.builders.contains_key(&metadata.id) {
            return Err(PluginRegistrationError::PluginAlreadyRegistered(
                metadata.id.clone(),
                metadata,
            ));
        }
        info!("Registered plugin {}: {:?}.", metadata.id, metadata);
        self.builders
            .insert(metadata.id.clone(), (metadata, Box::new(builder)));
        Ok(())
    }

    pub fn metadata(&self) -> impl Iterator<Item = &'_ PluginMetadata> {
        self.builders.values().map(|(m, _)| m)
    }

    pub fn build(&self, plugin_id: &str) -> Result<Box<dyn PluginInstance>, PluginBuilderError> {
        match self.builders.get(plugin_id) {
            Some((_, builder)) => builder.build(),
            None => Err(PluginBuilderError::PluginDoesNotExist(
                plugin_id.to_string(),
            )),
        }
    }
}

pub trait PluginBuilder: Send {
    fn metadata(&self) -> PluginMetadata;
    fn build(&self) -> Result<Box<dyn PluginInstance>, PluginBuilderError>;
}

#[derive(Clone, Eq, PartialEq, Debug, serde::Serialize)]
pub struct PluginMetadata {
    pub id: String,
    pub display_name: String,
}

impl PluginMetadata {
    pub fn validate(&self) -> Result<(), MetadataError> {
        // TODO(wmedrano): Do proper checks once LV2 plugins are registered
        // with proper IDs.
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MetadataError {}

impl std::error::Error for MetadataError {}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PluginRegistrationError {
    PluginAlreadyRegistered(String, PluginMetadata),
    InvalidMetadata(PluginMetadata, MetadataError),
}

impl std::error::Error for PluginRegistrationError {}

impl std::fmt::Display for PluginRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PluginBuilderError {
    PluginDoesNotExist(String),
    GenericError(&'static str),
}

impl std::error::Error for PluginBuilderError {}

impl std::fmt::Display for PluginBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
