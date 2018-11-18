use amethyst::{
    assets::{AssetStorage, Loader},
    prelude::*,
    renderer::{
        MaterialTextureSet, PngFormat, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, Texture,
        TextureMetadata,
    },
};
use std::collections::HashMap;

#[derive(Default)]
pub struct LoadedTextures {
    pub textures: HashMap<String, SpriteSheetHandle>,
}

pub fn sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_id = super::load::texture(world, png_path);

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    let handle = loader.load(
        ron_path,
        SpriteSheetFormat,
        texture_id,
        (),
        &sprite_sheet_store,
    );

    let mut my = world.write_resource::<LoadedTextures>();
    let old_val = my.textures.insert(png_path.into(), handle.clone());
    assert!(old_val.is_none());

    handle
}

/// Loads texture into world and returns texture id.
pub fn texture(world: &mut World, png_path: &str) -> u64 {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };

    let mut material_texture_set = world.write_resource::<MaterialTextureSet>();
    let texture_id = material_texture_set.len() as u64;
    material_texture_set.insert(texture_id, texture_handle);

    texture_id
}
