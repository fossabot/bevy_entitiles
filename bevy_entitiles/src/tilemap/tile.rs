use bevy::prelude::{Commands, Component, Entity, UVec2, Vec4};

use crate::MAX_LAYER_COUNT;

use super::map::Tilemap;

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub enum TileType {
    /// The most basic shape.
    #[default]
    Square,
    /// A diamond shape. It's like a square but rotated 45 degrees counterclockwise around the origin.
    /// But the coordinate system is the same as `Square`.
    IsometricDiamond,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TileFlip {
    Horizontal = 0b01,
    Vertical = 0b10,
    Both = 0b11,
}

#[derive(Clone)]
pub struct TileBuilder {
    pub(crate) texture_indices: [i32; MAX_LAYER_COUNT],
    pub(crate) top_layer: usize,
    pub(crate) anim: Option<AnimatedTile>,
    pub(crate) color: Vec4,
}

impl TileBuilder {
    /// Create a new tile builder.
    pub fn new(texture_index: u32) -> Self {
        let mut texture_indices = [-1; MAX_LAYER_COUNT];
        texture_indices[0] = texture_index as i32;
        Self {
            texture_indices,
            anim: None,
            top_layer: 0,
            color: Vec4::ONE,
        }
    }

    #[cfg(feature = "serializing")]
    pub fn from_serialized_tile(serialized_tile: &crate::serializing::SerializedTile) -> Self {
        Self {
            texture_indices: serialized_tile.texture_indices,
            top_layer: serialized_tile.top_layer,
            anim: serialized_tile.anim.clone(),
            color: serialized_tile.color,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_animation(mut self, anim: AnimatedTile) -> Self {
        self.anim = Some(anim);
        self
    }

    pub fn with_layer(mut self, layer: usize, texture_index: u32) -> Self {
        if let Some(anim) = self.anim.as_mut() {
            anim.layer = layer;
        } else if layer >= MAX_LAYER_COUNT {
            self.texture_indices[layer] = texture_index as i32;
        }

        self
    }

    pub(crate) fn build(&self, commands: &mut Commands, index: UVec2, tilemap: &Tilemap) -> Entity {
        let render_chunk_index_2d = index / tilemap.render_chunk_size;
        let render_chunk_index = {
            if tilemap.size.x % tilemap.render_chunk_size == 0 {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size)
                    + render_chunk_index_2d.x
            } else {
                render_chunk_index_2d.y * (tilemap.size.x / tilemap.render_chunk_size + 1)
                    + render_chunk_index_2d.x
            }
        } as usize;
        let mut tile = commands.spawn_empty();
        tile.insert(Tile {
            render_chunk_index,
            tilemap_id: tilemap.id,
            index,
            texture_indices: self.texture_indices,
            top_layer: 0,
            color: self.color,
        });
        if let Some(anim) = &self.anim {
            tile.insert(anim.clone());
        }
        tile.id()
    }
}

#[derive(Component, Clone, Debug)]
pub struct Tile {
    pub tilemap_id: Entity,
    pub render_chunk_index: usize,
    pub index: UVec2,
    pub texture_indices: [i32; MAX_LAYER_COUNT],
    pub top_layer: usize,
    pub color: Vec4,
}

#[derive(Component, Clone)]
#[cfg_attr(feature = "serializing", derive(serde::Serialize, serde::Deserialize))]
pub struct AnimatedTile {
    pub layer: usize,
    pub sequence_index: u32,
    pub fps: f32,
    pub is_loop: bool,
}
