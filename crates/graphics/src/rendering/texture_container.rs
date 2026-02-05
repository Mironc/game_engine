use std::{collections::HashMap, error::Error};

use ash::vk::{
    self, DeviceMemory, Extent3D, Format, Image, ImageCreateFlags, ImageCreateInfo, ImageLayout,
    ImageTiling, ImageType, ImageUsageFlags, ImageView, MemoryAllocateInfo, SampleCountFlags,
};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{self, Allocation, AllocationCreateDesc},
};
use slotmap::{SlotMap, new_key_type};

use crate::device::DeviceContext;

pub struct TextureContainer {
    images: SlotMap<TextureId, Texture>,
    image_views: SlotMap<TextureViewId, ImageView>,
    image_view_to_image: HashMap<TextureViewId, TextureId>,
}
impl TextureContainer {
    ///Creates new `TextureContainer`
    pub fn new() -> Self {
        Self {
            images: SlotMap::default(),
            image_views: SlotMap::default(),
            image_view_to_image: HashMap::new(),
        }
    }
    ///Creates `Texture` with given `CreateTexture` parameters
    pub fn create_texture(
        &mut self,
        device: &DeviceContext,
        create: CreateTexture,
    ) -> Result<TextureId, Box<dyn Error>> {
        let image_type = if create.extent.depth > 1 {
            ImageType::TYPE_3D
        } else if create.extent.height > 1 {
            ImageType::TYPE_2D
        } else {
            ImageType::TYPE_1D
        };
        //TODO:Add more parameters to create texture
        let image_create = ImageCreateInfo::default()
            .extent(create.extent)
            .image_type(image_type)
            .initial_layout(ImageLayout::UNDEFINED)
            .format(create.image_format)
            .mip_levels(1)
            .array_layers(1)
            .usage(
                ImageUsageFlags::TRANSFER_SRC
                    | ImageUsageFlags::TRANSFER_DST
                    | ImageUsageFlags::SAMPLED
                    | ImageUsageFlags::COLOR_ATTACHMENT,
            )
            .samples(SampleCountFlags::TYPE_1)
            .tiling(ImageTiling::OPTIMAL);
        let image = unsafe { device.create_image(&image_create, None)? };
        let image_mem_req = unsafe { device.get_image_memory_requirements(image) };

        let alloc = device.allocator().allocate(&AllocationCreateDesc {
            name: "Texture",
            requirements: image_mem_req,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: vulkan::AllocationScheme::DedicatedImage(image.clone()),
        })?;
        unsafe { device.bind_image_memory(image, alloc.memory(), alloc.offset()) }.unwrap();
        let texture = Texture {
            alloc,
            image,
            extent: create.extent,
            image_type,
            format: create.image_format,
        };
        Ok(self.images.insert(texture))
    }
    ///Creates `TextureView` with given `CreateTexture` parameters
    pub fn create_texture_view(&mut self, create: CreateTextureView) -> TextureViewId {
        self.image_views.insert(ImageView::null())
    }
    ///!! FOR TESTING PURPOSES!!
    ///
    /// Creates dummy `Texture`
    #[cfg(test)]
    pub fn create_texture_null(&mut self) -> TextureId {
        self.images.insert(Texture::default())
    }
    ///!! FOR TESTING PURPOSES !!
    ///
    /// Creates dummy `TextureView`
    #[cfg(test)]
    pub fn create_texture_view_null(&mut self) -> TextureViewId {
        self.image_views.insert(ImageView::null())
    }
}

new_key_type! {pub struct TextureId;}

new_key_type! {pub struct TextureViewId;}

///
#[derive(Debug, Default)]
pub struct Texture {
    image: Image,
    alloc: Allocation,
    image_type: ImageType,
    extent: Extent3D,
    format: Format,
}
///
#[derive(Debug, Clone, Default)]
pub struct CreateTexture {
    extent: Extent3D,
    image_format: Format,
}
impl CreateTexture {
    pub fn new() -> Self {
        Self::default()
    }

    ///Sets dimensions of the texture
    pub fn dimensions(mut self, width: u32, height: u32, depth: u32) -> Self {
        self.extent = Extent3D {
            width,
            height,
            depth,
        };
        self
    }

    ///Sets image format
    pub fn image_format(mut self, image_format: Format) -> Self {
        self.image_format = image_format;
        self
    }
}
///TODO:Fill in
pub struct CreateTextureView {
    texture_id: TextureId,
}
