use super::storage_api::*;

use ::desync::*;

use futures::prelude::*;
use futures::future;

use std::sync::*;
use std::collections::{HashMap};

///
/// Representation of a layer in memory
///
struct InMemoryLayerStorage {
    /// The properties for this layer
    properties: String
}

///
/// Representation of an animation in-memory
///
struct InMemoryStorageCore {
    /// The properties for the animation
    animation_properties: Option<String>,

    /// The edit log
    edit_log: Vec<String>,

    /// The definitions for each element
    elements: HashMap<i64, String>,

    /// The layers
    layers: HashMap<u64, InMemoryLayerStorage>
}

///
/// Provides an implementation of the storage API that stores its data in memory
///
pub struct InMemoryStorage {
    /// Where the data is stored for this object 
    storage: Arc<Desync<InMemoryStorageCore>>
}

impl InMemoryStorage {
    ///
    /// Creates a new in-memory storage for an animation
    ///
    pub fn new() -> InMemoryStorage {
        // Create the core
        let core = InMemoryStorageCore {
            animation_properties:   None,
            edit_log:               vec![],
            elements:               HashMap::new(),
            layers:                 HashMap::new()
        };

        // And the storage
        InMemoryStorage {
            storage: Arc::new(Desync::new(core))
        }
    }

    ///
    /// Returns the responses for a stream of commands
    ///
    pub fn get_responses<CommandStream: 'static+Send+Unpin+Stream<Item=Vec<StorageCommand>>>(&self, commands: CommandStream) -> impl Send+Unpin+Stream<Item=Vec<StorageResponse>> {
        pipe(Arc::clone(&self.storage), commands, |storage, commands| {
            future::ready(storage.run_commands(commands)).boxed()
        })
    }
}

impl InMemoryStorageCore {
    ///
    /// Runs a series of storage commands on this store
    ///
    pub fn run_commands(&mut self, commands: Vec<StorageCommand>) -> Vec<StorageResponse> {
        let mut response = vec![];

        for command in commands.into_iter() {
            use self::StorageCommand::*;

            match command {
                WriteAnimationProperties(props)                     => { self.animation_properties = Some(props); response.push(StorageResponse::Updated); }
                ReadAnimationProperties                             => { response.push(self.animation_properties.as_ref().map(|props| StorageResponse::AnimationProperties(props.clone())).unwrap_or(StorageResponse::NotFound)); }
                WriteEdit(edit)                                     => { self.edit_log.push(edit); response.push(StorageResponse::Updated); }
                ReadHighestUnusedElementId                          => { response.push(StorageResponse::HighestUnusedElementId(self.elements.keys().cloned().max().unwrap_or(-1)+1)); }
                ReadEditLogLength                                   => { response.push(StorageResponse::NumberOfEdits(self.edit_log.len())); }
                ReadEdits(edit_range)                               => { response.extend(edit_range.into_iter().map(|index| StorageResponse::Edit(index, self.edit_log[index].clone()))); }
                WriteElement(element_id, value)                     => { self.elements.insert(element_id, value); response.push(StorageResponse::Updated); }
                ReadElement(element_id)                             => { response.push(self.elements.get(&element_id).map(|element| StorageResponse::Element(element_id, element.clone())).unwrap_or(StorageResponse::NotFound)); }
                DeleteElement(element_id)                           => { self.elements.remove(&element_id); response.push(StorageResponse::Updated); }
                AddLayer(layer_id, properties)                      => { self.layers.insert(layer_id, InMemoryLayerStorage::new(properties)); response.push(StorageResponse::Updated); }
                DeleteLayer(layer_id)                               => { if self.layers.remove(&layer_id).is_some() { response.push(StorageResponse::Updated); } else { response.push(StorageResponse::NotFound); } }

                ReadLayers                                          => { 
                    for (layer_id, storage) in self.layers.iter() {
                        response.push(StorageResponse::LayerProperties(*layer_id, storage.properties.clone()));
                    }
                }
                
                WriteLayerProperties(layer_id, properties)          => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.properties = properties;
                        response.push(StorageResponse::Updated);
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadLayerProperties(layer_id)                       => {
                    if let Some(layer) = self.layers.get(&layer_id) {
                        response.push(StorageResponse::LayerProperties(layer_id, layer.properties.clone()));
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                AddKeyFrame(layer_id, when)                         => { }
                DeleteKeyFrame(layer_id, when)                      => { }
                ReadKeyFrames(layer_id, period)                     => { }
                AttachElementToLayer(layer_id, element_id, when)    => { }
                ReadElementAttachments(element_id)                  => { }
                DetachElementFromLayer(element_id)                  => { }
                ReadElementsForKeyFrame(layer_id, when)             => { }
            }
        }

        response
    }
}

impl InMemoryLayerStorage {
    ///
    /// Creates a new in-memory layer storage object
    ///
    pub fn new(properties: String) -> InMemoryLayerStorage {
        InMemoryLayerStorage {
            properties
        }
    }
}
