use ash::vk::{AccessFlags, CommandBuffer, ImageLayout, PipelineStageFlags};
use encase::DynamicUniformBuffer;

use crate::{
    device::DeviceContext,
    render_graph::{
        operations::draw_call::DrawCall,
        render_graph::{ResourceAccess, ResourceState, ResourceUsage},
        resource::ResourceId,
    },
    rendering::{
        buffer_container::{
            GeneralBufferId, UniformBufferId, UniformData, VertexBufferId, VertexData,
        },
        renderer_bundle::RendererBundle,
        texture_container::TextureViewId,
    },
    swapchain::FrameData,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    DrawCall(DrawCall),
    WriteBuffer(WriteBufferOp),
    Present(FrameData),
}
impl Operation {
    pub fn resource_state(&self, bundle: &mut RendererBundle) -> Option<Vec<ResourceState>> {
        match self {
            Operation::DrawCall(draw_call) => match draw_call {
                DrawCall::Direct { draw_param } => draw_param.resource_state(bundle),
            },
            Operation::WriteBuffer(write_buffer_op) => {
                let res = ResourceId::Buffer(write_buffer_op.buff);
                Some(
                    [ResourceState::new(
                        res,
                        ResourceUsage::Buffer(
                            PipelineStageFlags::HOST,
                            write_buffer_op.offset_bytes,
                            write_buffer_op.data.len() as u64,
                            AccessFlags::HOST_WRITE,
                            ResourceAccess::Write,
                        ),
                    )]
                    .to_vec(),
                )
            }
            Operation::Present(frame) => {
                let res = ResourceId::Texture(bundle.texture_container.insert_framedata(frame).0);
                Some(
                    [ResourceState::new(
                        res,
                        ResourceUsage::Texture(
                            ImageLayout::PRESENT_SRC_KHR,
                            PipelineStageFlags::BOTTOM_OF_PIPE,
                            AccessFlags::empty(),
                            ResourceAccess::Read,
                        ),
                    )]
                    .to_vec(),
                )
            }
        }
    }
    pub fn execute(
        &self,
        device: &DeviceContext,
        command_buffer: CommandBuffer,
        bundle: &RendererBundle,
    ) {
        match self {
            Operation::DrawCall(draw_call) => {
                draw_call.execute(bundle, command_buffer, device);
            }
            Operation::WriteBuffer(write_buffer_op) => {
                write_buffer_op.execute(bundle, device);
            }
            Operation::Present(_) => {
                //nothing cuz we need only to sync image
                ()
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteBufferOp {
    buff: GeneralBufferId,
    data: Vec<u8>,
    //len_bytes: u64,
    offset_bytes: u64,
}
impl WriteBufferOp {
    pub fn vertex_buffer<V: VertexData>(
        buff: VertexBufferId<V>,
        data: Vec<V>,
        offset: u64,
    ) -> Option<Self> {
        // Writes to outside of binded memory -> return None
        if offset + (data.len() as u64) > buff.len() {
            return None;
        }

        let v_size = std::mem::size_of::<V>();
        let byte_len = data.len() * v_size;
        let offset_bytes = offset * v_size as u64;

        let mut as_u8 = Vec::with_capacity(byte_len);

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr() as *const u8, as_u8.as_mut_ptr(), byte_len);
            as_u8.set_len(byte_len);
        }
        Some(Self {
            buff: *buff,
            data: as_u8,
            offset_bytes,
        })
    }
    pub fn uniform_buffer<U: UniformData + encase::internal::WriteInto>(
        buff: UniformBufferId<U>,
        mut data: Vec<U>,
        offset: u64,
    ) -> Option<Self> {
        // Writes to outside of binded memory -> return None
        if offset + (data.len() as u64) > buff.len() {
            return None;
        }

        let rhs: u64 = U::min_size().into();
        let offset_bytes = offset * rhs;
        let mut buf = Vec::new();
        let mut writer = DynamicUniformBuffer::new(&mut buf);
        for t in data.iter() {
            writer.write(&t).unwrap();
        }
        Some(Self {
            buff: *buff,
            data: buf,
            offset_bytes,
        })
    }
    //pub fn resource_state(&self) -> ResourceState {}
    pub fn execute(&self, bundle: &RendererBundle, device: &DeviceContext) {
        let buff = bundle
            .buffer_container
            .get_general_buffer(self.buff)
            .unwrap();
        let allocation = buff.alloc();
        let size = (std::mem::size_of_val(&self.data) * self.data.len()) as u64;

        if let Some(ptr) = allocation.mapped_ptr() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.data.as_ptr() as *const u8,
                    ptr.as_ptr() as *mut u8,
                    size as usize,
                );
            }
        } else {
            log::error!("Buffer does not support mapping");
        }
        unsafe {
            device
                .flush_mapped_memory_ranges(&[ash::vk::MappedMemoryRange::default()
                    .memory(allocation.memory())
                    .offset(allocation.offset())
                    .size(allocation.size())])
                .unwrap()
        };
    }
}
