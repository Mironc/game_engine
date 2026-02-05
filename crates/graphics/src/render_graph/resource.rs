use crate::rendering::texture_container::{TextureId, TextureViewId};

///This struct is used to unify all ids 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceId {
    Texture(TextureViewId),
}
