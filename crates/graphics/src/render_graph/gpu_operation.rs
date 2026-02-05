use crate::{render_graph::resource::ResourceId, rendering::texture_container::TextureViewId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    DrawCall(ResourceId, ResourceId),
    CopyCall(ResourceId, ResourceId),
    Present(ResourceId),
}

impl Operation {
    pub fn read_resources(&self) -> Vec<ResourceId> {
        [match self {
            Operation::DrawCall(resource_id, _resource_id1) => *resource_id,
            Operation::CopyCall(resource_id, _resource_id1) => *resource_id,
            Operation::Present(resource_id) => *resource_id,
        }]
        .to_vec()
    }
    pub fn write_resources(&self) -> Vec<ResourceId> {
        [match self {
            Operation::DrawCall(_resource_id, resource_id1) => *resource_id1,
            Operation::CopyCall(_resource_id, resource_id1) => *resource_id1,
            Operation::Present(_) => return [].to_vec(),
        }]
        .to_vec()
    }
}
