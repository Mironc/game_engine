use std::collections::HashMap;

use ash::vk::Pipeline;
use slotmap::SlotMap;

pub struct ShadersContainer {
    shaders: SlotMap<ShaderId, String>,
}
slotmap::new_key_type! {pub struct ShaderId;}

impl ShadersContainer {
    pub fn new() -> Self {
        Self {
            shaders: SlotMap::default(),
        }
    }

    pub fn insert(&mut self, shader_source: String) -> ShaderId {
        self.shaders.insert(shader_source)
    }
}
pub struct PipelineContainer {
    pipelines: SlotMap<PipelineId, Pipeline>,
}
slotmap::new_key_type! {pub struct PipelineId;}
//TODO:Creation of pipeline obj 