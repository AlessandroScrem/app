use crate::assets::texture_asset::ColorSpace;

// Include del file generato da build.rs
include!(concat!(env!("OUT_DIR"), "/static_textures.rs"));

pub struct StaticTexture {
    pub width: u32,
    pub height: u32,
    pub pixels: &'static [u8],
    pub format: ColorSpace,
}

pub const WHITE_STATIC_TEXTURE: StaticTexture = StaticTexture {
    width: WHITE_TEXTURE_WIDTH,
    height: WHITE_TEXTURE_HEIGHT,
    format: ColorSpace::Rgba8,
    pixels: WHITE_TEXTURE,
};

pub const BLACK_STATIC_TEXTURE: StaticTexture = StaticTexture {
    width: BLACK_TEXTURE_WIDTH,
    height: BLACK_TEXTURE_HEIGHT,
    format: ColorSpace::Rgba8,
    pixels: BLACK_TEXTURE,
};

pub const NORMAL_STATIC_TEXTURE: StaticTexture = StaticTexture {
    width: NORMAL_TEXTURE_WIDTH,
    height: NORMAL_TEXTURE_HEIGHT,
    format: ColorSpace::Rgba8,
    pixels: NORMAL_TEXTURE,
};

pub const LIGHTBULB_STATIC_TEXTURE: StaticTexture = StaticTexture {
    width: LIGHTBULB_TEXTURE_WIDTH,
    height: LIGHTBULB_TEXTURE_HEIGHT,
    format: ColorSpace::Srgba8,
    pixels: LIGHTBULB_TEXTURE,
};
