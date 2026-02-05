use std::collections::HashMap;

use ash::vk::Framebuffer as F;

use crate::rendering::texture_container::TextureViewId;

pub struct FramebufferContainer {
    framebuffers: HashMap<Vec<TextureViewId>, F>,
}
impl FramebufferContainer {
    pub fn new() -> Self {
        Self {
            framebuffers: HashMap::new(),
        }
    }
    pub fn add_framebuffer(
        &mut self,
        key: Vec<TextureViewId>,
        framebuffer_creation: FramebufferCreate,
    ) {
        todo!("Framebuffer creation");
        //self.framebuffers.insert(key, )
    }
}

pub struct FramebufferCreate {}
