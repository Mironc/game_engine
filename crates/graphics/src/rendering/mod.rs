use ash::vk::Pipeline;

use crate::rendering::{
    framebuffer_container::FramebufferContainer, pipeline_container::PipelineContainer,
    texture_container::TextureContainer,
};
pub mod framebuffer_container;
pub mod pipeline_container;
pub mod render_pass;
pub mod texture_container;

pub struct RendererContext {
    pipeline_c: PipelineContainer,
    framebuffer_c: FramebufferContainer,
    texture_c: TextureContainer,
}
impl RendererContext {}
