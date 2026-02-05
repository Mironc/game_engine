use ash::vk::{Image, ImageView};
use slotmap::{SlotMap, new_key_type};

pub struct TextureContainer {
    images: SlotMap<TextureId, Image>,
    image_views: SlotMap<TextureViewId, ImageView>,
}
impl TextureContainer {
    pub fn new() -> Self {
        Self {
            images: SlotMap::default(),
            image_views: SlotMap::default(),
        }
    }
    pub fn create_texture(&mut self, create: CreateTexture) -> TextureId {
        self.images.insert(Image::null())
    }
    pub fn create_texture_view(&mut self, create: CreateTextureView) -> TextureViewId {
        self.image_views.insert(ImageView::null())
    }
}
new_key_type! {pub struct TextureId;}
new_key_type! {pub struct TextureViewId;}

///TODO:Fill in
pub struct CreateTexture {}
///TODO:Fill in
pub struct CreateTextureView {}
