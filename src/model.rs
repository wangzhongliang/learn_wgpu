use std::ops::Range;

use crate::{vertex::Vertex, texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}
impl Default for ModelVertex {
    fn default() -> Self {
        ModelVertex { position: [0.0; 3], tex_coords: [0.0; 2], normal: [1.0; 3], tangent: [1.0; 3], bitangent: [1.0; 3] }
    }
}
impl Vertex for ModelVertex{
    // const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0=>Float32x3, 1=>Float32x2];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3
                }
            ],
            // attributes: &Self::ATTRIBS
        }
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize
}
pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some(name), 
            layout, 
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ]
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self, 
        mesh: &'a Mesh, 
        material: &'a Material, 
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup
    );
    fn draw_model(
        &mut self, 
        model: &'a Model, 
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup
    );
}

impl<'a, 'b>  DrawModel<'b> for wgpu::RenderPass<'a>
where 'b: 'a {
    fn draw_mesh(
        &mut self, 
        mesh: &'b Mesh, 
        material: &'b Material, 
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_mesh_instanced(
            &mut self,
            mesh: &'b Mesh,
            material: &'b Material,
            instances: Range<u32>,
            camera_bind_group: &'a wgpu::BindGroup,
            light_bind_group: &'b wgpu::BindGroup
        ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances)
    }
    fn draw_model(
        &mut self, 
        model: &'b Model, 
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup
    ) {
        self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }
    fn draw_model_instanced(
            &mut self,
            model: &'b Model,
            instances: Range<u32>,
            camera_bind_group: &'b wgpu::BindGroup,
            light_bind_group: &'b wgpu::BindGroup
        ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), camera_bind_group, light_bind_group);
        }
    }
}