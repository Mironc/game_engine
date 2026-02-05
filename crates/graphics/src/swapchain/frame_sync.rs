use std::sync::Arc;

use ash::{
    Device,
    vk::{Fence, FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo},
};

use crate::device::DeviceContext;
/// This struct is responsible for frame synchronization.
/// There's one framesync for every FrameInFlight
#[derive(Debug, Clone)]
pub struct FrameSync {
    frame_is_done: Fence,
    render_finished: Semaphore,
    image_available: Semaphore,
}
impl FrameSync {
    pub fn new(device: &Arc<DeviceContext>) -> Self {
        let create_fence = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let create_semaphore = SemaphoreCreateInfo::default();
        unsafe {
            let frame_is_done = device
                .create_fence(&create_fence, None)
                .expect("Couldn't create fence");
            let image_available = device
                .create_semaphore(&create_semaphore, None)
                .expect("Couldn't create semaphore");
            let render_finished = device
                .create_semaphore(&create_semaphore, None)
                .expect("Couldn't create semaphore");
            Self {
                frame_is_done,
                render_finished,
                image_available,
            }
        }
    }
    pub fn frame_done(&self) -> Fence {
        self.frame_is_done
    }
    ///Blocking fn until frame is frame is done
    pub fn wait_until_frame_done(&self, context: &DeviceContext) {
        unsafe {
            context
                .wait_for_fences(&[self.frame_is_done], true, u64::MAX)
                .expect("Error occured while waiting for next frame")
        };
    }
    ///Blocking fn waits until frame is frame is done
    pub fn reset_frame_done_fence(&self, context: &DeviceContext) {
        unsafe {
            context
                .reset_fences(&[self.frame_is_done])
                .expect("Couldn't reset frame done fence")
        }
    }
    ///Returns semaphore that is responsible for signaling when image is ready for present call
    pub fn image_available(&self) -> Semaphore {
        self.image_available
    }
    ///Returns semaphore that is responsible for signaling when image was presented
    pub fn render_finished(&self) -> Semaphore {
        self.render_finished
    }

    pub fn destroy(&mut self, device: &Arc<DeviceContext>) {
        unsafe {
            device
                .device_wait_idle()
                .expect("Something went wrong while waiting for gpu idle")
        };
        unsafe {
            device.destroy_semaphore(self.image_available, None);
            device.destroy_semaphore(self.render_finished, None);
            device.destroy_fence(self.frame_is_done, None);
        }
    }
}
impl Drop for FrameSync {
    fn drop(&mut self) {}
}
