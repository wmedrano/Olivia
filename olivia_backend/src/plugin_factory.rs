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

    pub fn register(&mut self, b: Box<dyn PluginBuilder>) -> Result<(), PluginRegistrationError> {
        let metadata = b.metadata();
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
        self.builders.insert(metadata.id.clone(), (metadata, b));
        Ok(())
    }

    pub fn build_plugin(&self, id: &str) -> Option<Box<dyn PluginInstance>> {
        self.builders.get(id).map(|(_, b)| b.build())
    }

    pub fn build_track(
        &self,
        plugin_id: &str,
        buffer_size: usize,
    ) -> Option<olivia_core::processor::Track> {
        let volume = 1.0;
        self.build_plugin(plugin_id)
            .map(|p| olivia_core::processor::Track::new(p, buffer_size, volume))
    }
}

pub trait PluginBuilder {
    fn metadata(&self) -> PluginMetadata;
    fn build(&self) -> Box<dyn PluginInstance>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PluginMetadata {
    pub id: String,
    pub display_name: String,
}

impl PluginMetadata {
    pub fn validate(&self) -> Result<(), MetadataError> {
        if self.id.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(MetadataError::IdContainsUpperCase(self.id.clone()));
        }
        if !self
            .id
            .chars()
            .any(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(MetadataError::IdContainsNonAlphaNumericOrUnderscore(
                self.id.clone(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MetadataError {
    IdContainsUpperCase(String),
    IdContainsNonAlphaNumericOrUnderscore(String),
}

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
