use std::{collections::HashMap, error::Error};

use ash::vk::{
    Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageSubresourceRange,
    ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo, SampleCountFlags,
};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{self, Allocation, AllocationCreateDesc},
};
use slotmap::{SlotMap, new_key_type};

use crate::device::DeviceContext;

/// Centralized container for managing all that related to textures, but not array of textures
pub struct TextureContainer {
    images: SlotMap<TextureId, Texture>,
    image_views: SlotMap<TextureViewId, TextureView>,
    image_view_to_image: HashMap<TextureViewId, TextureId>,
}
impl TextureContainer {
    /// Creates empty `TextureContainer`
    pub fn new() -> Self {
        Self {
            images: SlotMap::default(),
            image_views: SlotMap::default(),
            image_view_to_image: HashMap::new(),
        }
    }

    /// Creates `Texture` with given `CreateTexture`
    ///
    /// # Errors
    /// returns error if image creation or memory allocation fails
    pub fn create_texture(
        &mut self,
        device: &DeviceContext,
        create: CreateTexture,
    ) -> Result<TextureId, Box<dyn Error>> {
        let image_type = if create.dimensions.depth > 1 {
            ImageType::TYPE_3D
        } else if create.dimensions.height > 1 {
            ImageType::TYPE_2D
        } else {
            ImageType::TYPE_1D
        };
        //TODO:Add more parameters to create texture
        let image_create = ImageCreateInfo::default()
            .extent(create.dimensions)
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
            extent: create.dimensions,
            image_type,
            format: create.image_format,
        };
        Ok(self.images.insert(texture))
    }

    /// Creates `TextureView` with given `CreateTexture`
    pub fn create_texture_view(
        &mut self,
        device: &DeviceContext,
        create: CreateTextureView,
    ) -> Result<TextureViewId, Box<dyn Error>> {
        if let Some(texture_id) = create.texture_id {
            let subresource = ImageSubresourceRange::default()
                .base_mip_level(0)
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(1);
            let image_view_createinfo = ImageViewCreateInfo::default()
                .format(create.view_format)
                .image(self.images[texture_id].image)
                .subresource_range(subresource);
            let texture = self.get_image(texture_id).unwrap();
            let image_view = unsafe { device.create_image_view(&image_view_createinfo, None) }?;
            let texture_view = TextureView{ image_view, extent: texture.dimensions(), format: create.view_format };
            let view_id = self.image_views.insert(texture_view);

            self.image_view_to_image.insert(view_id, texture_id);
            return Ok(view_id);
        }
        Err("No TextureId was provided".into())
    }

    /// Returns a reference to the `Texture` associated with the `TextureId`
    ///
    /// Returns `None` if texture has been destroyed or the `TextureId` is invalid
    pub fn get_image(&self, texture_id: TextureId) -> Option<&Texture> {
        self.images.get(texture_id)
    }

    /// Returns a reference to the `TextureView` associated with the `TextureViewId`
    ///
    /// Returns `None` if texture has been destroyed or the `TextureViewId` is invalid
    pub fn get_image_view(&self, view_id: TextureViewId) -> Option<&TextureView> {
        self.image_views.get(view_id)
    }

    /// **FOR TESTING PURPOSES**
    ///
    /// Creates a dummy `Texture`
    #[cfg(test)]
    pub fn create_texture_null(&mut self) -> TextureId {
        self.images.insert(Texture::default())
    }

    /// **FOR TESTING PURPOSES**
    ///
    /// Creates a dummy `TextureView`
    #[cfg(test)]
    pub fn create_texture_view_null(&mut self) -> TextureViewId {
        self.image_views.insert(TextureView::default())
    }
}

//TODO:Add reference counting for automatic dispose
new_key_type! {
    /// Unique identifier to a `Texture` in a `TextureContainer`
    pub struct TextureId;
}

new_key_type! {
    /// Unique identifier to a `TextureView` in a `TextureContainer`
    pub struct TextureViewId;
}

/// A GPU-driven image resource.
#[derive(Debug, Default)]
pub struct Texture {
    image: Image,
    alloc: Allocation,
    image_type: ImageType,
    extent: Extent3D,
    format: Format,
    //TODO:Add mipmap level count
}
impl Texture {
    /// Returns the raw handle `VKImage`
    pub fn handle(&self) -> Image {
        self.image
    }

    /// Returns the physical dimensions of texture
    pub fn dimensions(&self) -> Extent3D {
        self.extent
    }

    /// Returns the underlying image `Format`
    pub fn format(&self) -> Format {
        self.format
    }

    /// Returns the dimensionality type of the texture (1D, 2D, or 3D).
    pub fn image_type(&self) -> ImageType {
        self.image_type
    }

    /// Returns the reference to the allocation info of texture
    pub fn allocation(&self) -> &Allocation {
        &self.alloc
    }
}
/// Configuration parameters for creating a `Texture`.
#[derive(Debug, Clone, Default)]
pub struct CreateTexture {
    dimensions: Extent3D,
    image_format: Format,
}
impl CreateTexture {
    /// Creates new `CreateTexture` with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the dimensions of the texture
    pub fn dimensions(mut self, width: u32, height: u32, depth: u32) -> Self {
        self.dimensions = Extent3D {
            width,
            height,
            depth,
        };
        self
    }

    /// Sets the image format
    pub fn image_format(mut self, image_format: Format) -> Self {
        self.image_format = image_format;
        self
    }
}

/// A struct that define how renderer will read and write data into `Texture`
#[derive(Debug,Default)]
pub struct TextureView {
    image_view: ImageView,
    extent: Extent3D,
    format: Format,
    //TODO:Add mipmap level
}
/// Configuration parameters for creating a `TextureView`.
#[derive(Debug, Clone, Default)]
pub struct CreateTextureView {
    texture_id: Option<TextureId>,
    view_format: Format,
}
impl CreateTextureView {
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets `TextureId` for the image view
    pub fn texture_id(mut self, texture_id: TextureId) -> Self {
        self.texture_id = Some(texture_id);
        self
    }
}
