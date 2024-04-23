use std::marker::PhantomData;

use bevy::{
    asset::{AssetServer, Handle},
    ecs::{system::Resource, world::World},
    prelude::FromWorld,
    render::{
        render_resource::{
            BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Face, FragmentState,
            FrontFace, MultisampleState, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPipelineDescriptor, Shader, ShaderDefVal, ShaderRef, SpecializedRenderPipeline,
            TextureFormat, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use crate::tilemap::map::TilemapType;

use super::{
    binding::TilemapBindGroupLayouts,
    material::{StandardTilemapMaterial, TilemapMaterial},
    TILEMAP_SHADER,
};

#[derive(Resource)]
pub struct EntiTilesPipeline<M: TilemapMaterial> {
    pub view_layout: BindGroupLayout,
    pub uniform_buffers_layout: BindGroupLayout,
    pub storage_buffers_layout: BindGroupLayout,
    pub color_texture_layout: BindGroupLayout,
    pub add_material_layout: BindGroupLayout,
    pub vertex_shader: Handle<Shader>,
    pub fragment_shader: Handle<Shader>,
    pub marker: PhantomData<M>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct EntiTilesPipelineKey {
    pub msaa: u32,
    pub map_type: TilemapType,
    pub without_texture: bool,
}

impl<M: TilemapMaterial> FromWorld for EntiTilesPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let layouts = world.resource::<TilemapBindGroupLayouts>();
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();

        Self {
            view_layout: layouts.view_layout.clone(),
            uniform_buffers_layout: layouts.tilemap_uniforms_layout.clone(),
            storage_buffers_layout: layouts.tilemap_storage_layout.clone(),
            color_texture_layout: layouts.color_texture_layout.clone(),
            add_material_layout: M::bind_group_layout(render_device),
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => TILEMAP_SHADER,
                ShaderRef::Handle(h) => h,
                ShaderRef::Path(p) => asset_server.load(p),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => TILEMAP_SHADER,
                ShaderRef::Handle(h) => h,
                ShaderRef::Path(p) => asset_server.load(p),
            },
            marker: PhantomData::default(),
        }
    }
}

impl<M: TilemapMaterial> SpecializedRenderPipeline for EntiTilesPipeline<M> {
    type Key = EntiTilesPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs: Vec<ShaderDefVal> = vec![];
        shader_defs.push(
            {
                match key.map_type {
                    TilemapType::Square => "SQUARE",
                    TilemapType::Isometric => "ISOMETRIC",
                    TilemapType::Hexagonal(_) => "HEXAGONAL",
                }
            }
            .into(),
        );
        #[cfg(feature = "atlas")]
        shader_defs.push("ATLAS".into());

        let mut vtx_fmt = vec![
            // position
            VertexFormat::Float32x3,
            // index + anim_start + anim_len
            VertexFormat::Sint32x4,
            // color
            VertexFormat::Float32x4,
        ];

        if key.without_texture {
            shader_defs.push("WITHOUT_TEXTURE".into());
        } else {
            // texture_indices
            vtx_fmt.push(VertexFormat::Sint32x4);
            // flip
            vtx_fmt.push(VertexFormat::Uint32x4);
        }

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vtx_fmt);

        let mut layout = vec![
            // group(0)
            self.view_layout.clone(),
            // group(1)
            self.uniform_buffers_layout.clone(),
        ];

        if !key.without_texture {
            // group(2)
            layout.push(self.color_texture_layout.clone());
            // group(3)
            layout.push(self.storage_buffers_layout.clone());
        }

        if std::any::TypeId::of::<M>() != std::any::TypeId::of::<StandardTilemapMaterial>() {
            // group(4)
            layout.push(self.add_material_layout.clone());
        }

        let mut desc = RenderPipelineDescriptor {
            label: Some("tilemap_pipeline".into()),
            layout,
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.vertex_shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: "tilemap_vertex".into(),
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: "tilemap_fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        M::specialize(&mut desc);

        desc
    }
}
