use crate::plugin_factory::{PluginBuilder, PluginBuilderError, PluginMetadata};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::RwLock;

pub fn load_plugins() -> Vec<Lv2PluginBuilder> {
    let w = lilv::World::with_load_all();
    let mut plugin_builders = Vec::new();
    let lv2_resources = Arc::new(Lv2Resources {
        audio_port: w.new_uri("http://lv2plug.in/ns/lv2core#AudioPort"),
        atom_port: w.new_uri("http://lv2plug.in/ns/ext/atom#AtomPort"),
        control_port: w.new_uri("http://lv2plug.in/ns/lv2core#ControlPort"),
        input_port: w.new_uri("http://lv2plug.in/ns/lv2core#InputPort"),
        output_port: w.new_uri("http://lv2plug.in/ns/lv2core#OutputPort"),
        urid_map: UridMapFeature::default(),
    });
    let supported_features = [w.new_uri(UridMapFeature::URI)];
    for plugin in w.all_plugins().iter() {
        if plugin.uri().as_uri().is_none() {
            warn!("Could not get uri from {:?}.", plugin.uri().turtle_token());
            continue;
        }
        let mut supported = true;
        if let Some(required_features) = plugin.required_features() {
            for feature in required_features.iter() {
                if supported_features.iter().find(|f| *f == &feature).is_none() {
                    supported = false;
                    error!(
                        "LV2 plugin {:?} requires feature {:?}.",
                        plugin.uri(),
                        feature
                    );
                }
            }
        }
        if let Some(optional_features) = plugin.optional_features() {
            for feature in optional_features.iter() {
                if supported_features.iter().find(|f| *f == &feature).is_none() {
                    warn!(
                        "LV2 plugin {:?} has optional feature {:?}.",
                        plugin.uri(),
                        feature
                    );
                }
            }
        }
        if !supported {
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
        let builder = Lv2PluginBuilder {
            lv2_resources: lv2_resources.clone(),
            metadata,
            plugin,
        };
        plugin_builders.push(builder);
    }
    plugin_builders
}

type PortIndex = usize;

pub struct Lv2PluginBuilder {
    lv2_resources: std::sync::Arc<Lv2Resources>,
    plugin: lilv::Plugin,
    metadata: PluginMetadata,
}

impl PluginBuilder for Lv2PluginBuilder {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn build(
        &self,
    ) -> Result<
        Box<dyn olivia_core::plugin::PluginInstance>,
        crate::plugin_factory::PluginBuilderError,
    > {
        let features: Vec<*const lv2_raw::LV2Feature> = vec![
            self.lv2_resources.urid_map.as_lv2_feature(),
            std::ptr::null(),
        ];
        let mut instance = unsafe {
            self.plugin.instantiate(44100.0, features.as_ptr()).ok_or(
                PluginBuilderError::GenericError("Failed to instantiate LV2 plugin."),
            )?
        };

        let mut audio_output_ports = self
            .plugin
            .ports()
            .enumerate()
            .filter(|(_, p)| self.lv2_resources.is_audio(p) && self.lv2_resources.is_output(p))
            .map(|(port_index, _)| port_index);
        let mut sequence_input_ports = self
            .plugin
            .ports()
            .enumerate()
            .filter(|(_, p)| self.lv2_resources.is_atom(p) && self.lv2_resources.is_input(p))
            .map(|(port_index, _)| port_index);
        let mut control_inputs: Vec<_> = self
            .plugin
            .ports()
            .enumerate()
            .filter(|(_, p)| self.lv2_resources.is_control(p) && self.lv2_resources.is_input(p))
            .map(|(i, p)| (i, p.range().default))
            .collect();
        let midi_uri_cstr = std::ffi::CStr::from_bytes_with_nul(lilv_sys::LILV_URI_MIDI_EVENT)
            .map_err(|e| {
                error!("Could not build midi URI CStr: {:?}", e);
                PluginBuilderError::GenericError("could not build midi URI CStr")
            })?;
        let midi_uri = self.lv2_resources.urid_map.map(&midi_uri_cstr);

        for i in 0..self.plugin.num_ports() {
            unsafe { instance.connect_port(i, std::ptr::null_mut() as *mut f32) };
        }
        for (i, v) in control_inputs.iter_mut() {
            unsafe { instance.connect_port(*i, v) };
        }

        Ok(Box::new(LV2PluginInstance {
            instance,
            midi_buffer: Lv2AtomSequence::new(4096),
            midi_uri,
            midi_port: sequence_input_ports.next(),
            control_inputs,
            out_left_port: audio_output_ports.next(),
            out_right_port: audio_output_ports.next(),
            other_output_ports: audio_output_ports.collect(),
        }))
    }
}

#[derive(Debug)]
pub struct LV2PluginInstance {
    instance: lilv::Instance,
    midi_buffer: Lv2AtomSequence,
    midi_uri: u32,
    midi_port: Option<usize>,
    control_inputs: Vec<(PortIndex, f32)>,
    out_left_port: Option<usize>,
    out_right_port: Option<usize>,
    other_output_ports: Vec<usize>,
}

impl olivia_core::plugin::PluginInstance for LV2PluginInstance {
    fn process(
        &mut self,
        midi: &[olivia_core::TimedMidi],
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
        if let Some(port_index) = self.midi_port {
            self.midi_buffer.clear();
            for timed_midi in midi.iter() {
                let mut event = Lv2AtomEvent::new(timed_midi.frame as i64, self.midi_uri);
                if let Ok(size) = timed_midi.message.copy_to_slice(&mut event.buffer) {
                    event.set_size(size);
                    self.midi_buffer.append_event(&event);
                }
            }
            unsafe {
                self.instance
                    .connect_port(port_index, self.midi_buffer.as_mut_ptr())
            };
        }
        info!("About to run.");
        self.instance.run(samples);
    }
}

struct Lv2Resources {
    audio_port: lilv::Node,
    atom_port: lilv::Node,
    control_port: lilv::Node,
    input_port: lilv::Node,
    output_port: lilv::Node,
    urid_map: UridMapFeature<'static>,
}

impl Lv2Resources {
    fn is_audio(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.audio_port)
    }

    fn is_atom(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.atom_port)
    }

    fn is_control(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.control_port)
    }

    fn is_output(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.output_port)
    }

    fn is_input(&self, port: &lilv::Port) -> bool {
        port.classes().contains(&self.input_port)
    }
}

/// The underlying buffer backing the data for an atom event.
type Lv2AtomEventBuffer = [u8; 16];

/// An single atom event.
#[repr(packed)]
struct Lv2AtomEvent {
    header: lv2_raw::LV2AtomEvent,
    pub buffer: Lv2AtomEventBuffer,
}

impl Lv2AtomEvent {
    /// Create a new atom event with the given time and type. The event can be filled in by setting
    /// the bytes in buffer and calling `set_size`.
    fn new(time_in_frames: i64, my_type: u32) -> Lv2AtomEvent {
        let mut event: Lv2AtomEvent = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        event.header.time_in_frames = time_in_frames;
        event.header.body.mytype = my_type;
        event.header.body.size = 0;
        event
    }

    /// Set the size of the atom. Must be less than or equal to the size of the buffer.
    fn set_size(&mut self, size: usize) {
        debug_assert!(size < self.buffer.len(), "{} < {}", size, self.buffer.len());
        self.header.body.size = size as u32;
    }

    /// Return a pointer to the header of the atom.
    #[allow(safe_packed_borrows)]
    fn as_ptr(&self) -> *const lv2_raw::LV2AtomEvent {
        &self.header
    }
}

/// An atom sequence.
struct Lv2AtomSequence {
    buffer: Vec<lv2_raw::LV2AtomSequence>,
}

impl Lv2AtomSequence {
    /// Create a new sequence that can hold about desired_capacity bytes.
    fn new(desired_capacity: usize) -> Lv2AtomSequence {
        let len = desired_capacity / std::mem::size_of::<lv2_raw::LV2AtomSequence>();
        let mut buffer = Vec::with_capacity(len);
        buffer.resize_with(len, || lv2_raw::LV2AtomSequence {
            atom: lv2_raw::LV2Atom { size: 0, mytype: 0 },
            body: lv2_raw::LV2AtomSequenceBody { unit: 0, pad: 0 },
        });
        let mut seq = Lv2AtomSequence { buffer };
        seq.clear();
        seq
    }

    /// Clear all events in the sequence.
    #[inline(always)]
    fn clear(&mut self) {
        unsafe { lv2_raw::atomutils::lv2_atom_sequence_clear(self.as_mut_ptr()) }
    }

    /// Append an event to the sequence. If there is no capacity for it, then it will not be
    /// appended.
    #[inline(always)]
    fn append_event(&mut self, event: &Lv2AtomEvent) {
        unsafe {
            lv2_raw::atomutils::lv2_atom_sequence_append_event(
                self.as_mut_ptr(),
                self.capacity() as u32,
                event.as_ptr(),
            )
        };
    }

    /// Return a mutable pointer to the underlying data.
    fn as_mut_ptr(&mut self) -> *mut lv2_raw::LV2AtomSequence {
        self.buffer.as_mut_ptr()
    }

    /// Get the capacity of the sequence.
    fn capacity(&self) -> usize {
        let slice: &[lv2_raw::LV2AtomSequence] = &self.buffer;
        std::mem::size_of_val(slice)
    }
}

impl std::fmt::Debug for Lv2AtomSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let capacity = self.capacity();
        f.debug_struct("Lv2AtomSequence")
            .field("capacity", &capacity)
            .finish()
    }
}

/// An implementation of LV2 URID map.
enum UridMapFeatureImpl<'a> {
    /// A native Rust implementation.
    Native(Box<UridMapFeatureNativeImpl>),
    /// An abstract implementation exposed through the LV2_URID_Map handle and function pointer.
    Abstract(&'a lv2_raw::LV2UridMap),
}

impl<'a> UridMapFeatureImpl<'a> {
    fn map(&self, uri: &CStr) -> u32 {
        match self {
            UridMapFeatureImpl::Native(f) => f.map(uri),
            UridMapFeatureImpl::Abstract(f) => {
                let handle = f.handle;
                (f.map)(handle, uri.as_ptr())
            }
        }
    }
}

/// Provides the urid map feature for LV2. See documentation for urid map at
/// http://lv2plug.in/ns/ext/urid/#map.
// The fields are actually referenced as void ptrs within feature and data.
#[allow(dead_code)]
struct UridMapFeature<'a> {
    feature: lv2_raw::LV2Feature,
    data: Option<Box<lv2_raw::LV2UridMap>>,
    urid_map_impl: UridMapFeatureImpl<'a>,
}

unsafe impl Send for UridMapFeature<'static> {}
unsafe impl Sync for UridMapFeature<'static> {}

impl Default for UridMapFeature<'static> {
    /// Create the default instance for UridMapFeature with no registered URIs. URIs will register
    /// themselves with the `get` method.
    fn default() -> UridMapFeature<'static> {
        let mut urid_map_impl: Box<UridMapFeatureNativeImpl> = Box::default();
        let mut data = Box::new(lv2_raw::LV2UridMap {
            handle: urid_map_impl.as_mut() as *mut UridMapFeatureNativeImpl
                as *mut std::ffi::c_void,
            map: urid_map_feature_native_impl_map,
        });
        UridMapFeature {
            feature: lv2_raw::LV2Feature {
                uri: UridMapFeature::URI.as_ptr() as *const ::std::os::raw::c_char,
                data: data.as_mut() as *mut lv2_raw::LV2UridMap as *mut std::ffi::c_void,
            },
            data: Some(data),
            urid_map_impl: UridMapFeatureImpl::Native(urid_map_impl),
        }
    }
}

extern "C" fn urid_map_feature_native_impl_map(
    handle: *mut std::ffi::c_void,    /*Type is UridMapFeatureNativeImpl*/
    uri: *const std::os::raw::c_char, /*CStr*/
) -> u32 {
    let self_ptr = handle as *const UridMapFeatureNativeImpl;
    unsafe {
        match self_ptr.as_ref() {
            Some(self_ref) => self_ref.map(CStr::from_ptr(uri)),
            None => {
                error!("URID Map had null handle for UridMapFeatureNativeImpl.");
                0
            }
        }
    }
}

impl<'a> UridMapFeature<'a> {
    /// The URI for the urid map LV2 feature.
    const URI: &'static str = "http://lv2plug.in/ns/ext/urid#map\0";

    /// Get the urid map as an LV2_feature.
    fn as_lv2_feature(&self) -> &lv2_raw::LV2Feature {
        &self.feature
    }

    /// Get the id for the given uri. If the uri does not have an ID, it will be registered
    /// with a new one.
    ///
    /// Note: This method makes uses of mutexes and heap based maps; do not run in a realtime
    /// context. If needed, cache the returned IDs.
    fn map(&self, uri: &CStr) -> u32 {
        self.urid_map_impl.map(uri)
    }
}

impl<'a> From<&'a lv2_raw::LV2UridMap> for UridMapFeature<'a> {
    fn from(map: &'a lv2_raw::LV2UridMap) -> UridMapFeature<'a> {
        UridMapFeature {
            feature: lv2_raw::LV2Feature {
                uri: lv2_raw::LV2_URID__MAP.as_ptr() as *const ::std::os::raw::c_char,
                data: map as *const lv2_raw::LV2UridMap as *mut std::ffi::c_void,
            },
            data: None, /*The data is borrowed from map*/
            urid_map_impl: UridMapFeatureImpl::Abstract(map),
        }
    }
}

impl<'a> TryFrom<&'a lv2_raw::LV2Feature> for UridMapFeature<'a> {
    type Error = PluginBuilderError;

    /// Convert the feature into a UridMapFeature. If the LV2 feature is not a URID map feature,
    /// then an error is returned.
    fn try_from(
        feature: &'a lv2_raw::LV2Feature,
    ) -> Result<UridMapFeature<'a>, PluginBuilderError> {
        let feature_uri = unsafe { CStr::from_ptr(feature.uri) };
        if feature_uri.to_bytes() == lv2_raw::LV2_URID__MAP.as_bytes() {
            let urid_map_ptr = feature.data as *const lv2_raw::LV2UridMap;
            match unsafe { urid_map_ptr.as_ref() } {
                Some(r) => Ok(UridMapFeature::from(r)),
                None => Err(PluginBuilderError::GenericError("feature data is null")),
            }
        } else {
            Err(PluginBuilderError::GenericError("feature is not URID map"))
        }
    }
}

/// Implementation for uri map LV2 feature.
struct UridMapFeatureNativeImpl {
    map: RwLock<HashMap<CString, u32>>,
    next_id: AtomicU32,
}

impl Default for UridMapFeatureNativeImpl {
    /// Create a new UridMapFeatureNativeImpl. With no registered URIs.
    fn default() -> UridMapFeatureNativeImpl {
        UridMapFeatureNativeImpl {
            map: RwLock::default(),
            next_id: AtomicU32::new(1),
        }
    }
}

impl UridMapFeatureNativeImpl {
    /// Get the ID for the given uri. If the URI is not registered, then it will be registered
    /// with a new unique ID. This function makes use of a heap based hash map and mutex so it is
    /// not suitable for realtime execution. Results for important URIs should be cached.
    fn map(&self, uri: &CStr) -> u32 {
        if let Some(id) = self.map.read().unwrap().get(uri).copied() {
            return id;
        };
        let mut map = self.map.write().unwrap();
        // We check if the ID is present again in case it was inserted in the time between
        // releasing the read lock and regaining the write lock.
        if let Some(id) = map.get(uri).copied() {
            return id;
        }
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        info!("Mapped URI {:?} to {}.", uri, id);
        map.insert(CString::from(uri), id);
        id
    }
}
