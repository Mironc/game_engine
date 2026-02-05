use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandPool, CommandPoolCreateInfo,
    CommandPoolResetFlags,
};

use super::queue_family::QueueFamily;
use crate::device::DeviceContext;

/// Command pool that is bond to certain thread
///
///
#[derive(Debug, Clone)]
pub struct LocalCommandPool {
    command_pool: CommandPool,
    command_buffers: Vec<CommandBuffer>,
    used: usize,
}
impl LocalCommandPool {
    pub fn new(logical_device: &DeviceContext, queue_family: QueueFamily) -> Self {
        let commandpool_createinfo =
            CommandPoolCreateInfo::default().queue_family_index(queue_family.id() as u32);
        let command_pool =
            unsafe { logical_device.create_command_pool(&commandpool_createinfo, None) }
                .expect("Couldn't create command pool");
        Self {
            command_pool,
            command_buffers: Vec::new(),
            used: 0,
        }
    }
    ///Gives clean command buffer  
    pub fn get_buffer(&mut self, logical_device: &DeviceContext) -> CommandBuffer {
        if self.used + 1 < self.command_buffers.len() {
            self.used += 1;
            return self.command_buffers[self.used];
        } else {
            self.used += 1;
            let allocate_info = CommandBufferAllocateInfo::default()
                .command_pool(self.command_pool)
                .command_buffer_count(1);
            let command_buffer = unsafe {
                logical_device
                    .allocate_command_buffers(&allocate_info)
                    .expect("Couldn't allocate command buffer")
            }
            .first()
            .unwrap()
            .to_owned();
            self.command_buffers.push(command_buffer);
            command_buffer
        }
    }
    ///Returns all buffers that was written
    pub fn get_written_buffers(&self) -> &[CommandBuffer] {
        &self.command_buffers[0..self.used]
    }
    ///Zeroes all buffers
    ///
    ///Intended to use only after all work for current frame is done
    pub fn reset(&mut self, logical_device: &DeviceContext) {
        let pool_reset = CommandPoolResetFlags::empty();
        unsafe {
            logical_device
                .reset_command_pool(self.command_pool, pool_reset)
                .expect("Couldn't free command pool")
        };
        self.used = 0;
    }
}
