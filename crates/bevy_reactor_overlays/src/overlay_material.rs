use bevy::{
    asset::Asset,
    color::LinearRgba,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        alpha::AlphaMode,
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, CompareFunction, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

/// Material for overlays
#[derive(Debug, Clone, AsBindGroup, Asset, TypePath, Default)]
pub struct OverlayMaterial {
    #[uniform(1)]
    pub(crate) color: LinearRgba,
}

#[allow(unused_variables)]
impl Material for OverlayMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_reactor_overlays/overlay.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_reactor_overlays/overlay.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(ref mut depth_stencil) = descriptor.depth_stencil {
            depth_stencil.depth_write_enabled = true;
            depth_stencil.depth_compare = CompareFunction::GreaterEqual;
        }
        Ok(())
    }
}

/// Material for occluded overlays
#[derive(Debug, Clone, AsBindGroup, Asset, TypePath, Default)]
pub struct UnderlayMaterial {
    #[uniform(1)]
    pub(crate) color: LinearRgba,
}

#[allow(unused_variables)]
impl Material for UnderlayMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_reactor_overlays/overlay.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_reactor_overlays/overlay.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(ref mut depth_stencil) = descriptor.depth_stencil {
            depth_stencil.depth_write_enabled = true;
            depth_stencil.depth_compare = CompareFunction::Less;
        }
        Ok(())
    }
}
