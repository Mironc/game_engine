use ash::vk::Image;

use crate::swapchain::frame_sync::FrameSync;

#[derive(Debug)]
pub struct FrameData {
    fif_id: usize,
    sync: FrameSync,
    image_id: u32,
    image: Image,
}
impl FrameData {
    pub fn new(fif_id: usize, sync: FrameSync, image_id: u32, image: Image) -> Self {
        Self {
            fif_id,
            sync,
            image_id,
            image,
        }
    }

    pub fn fif_id(&self) -> usize {
        self.fif_id
    }

    pub fn sync(&self) -> &FrameSync {
        &self.sync
    }

    pub fn image_id(&self) -> u32 {
        self.image_id
    }

    pub fn image(&self) -> &Image {
        &self.image
    }
}
