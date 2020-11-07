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
}

pub trait PluginBuilder: Send {
    fn metadata(&self) -> PluginMetadata;
    fn build(&self) -> Box<dyn PluginInstance>;
}

#[derive(Clone, Eq, PartialEq, Debug, serde::Serialize)]
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
