use std::{
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};

use gltf::{Document, Gltf};
use pi_atom::Atom;
use pi_gltf as gltf;

use crate::interface::{
    EGLTFVertexDataKind, GLTFVertexAttribute, GLTFVertexBufferDesc, InterfaceGLTFLoader,
};

pub struct GltfLoader {
    gltf: Gltf,
    path: PathBuf,
}

impl GltfLoader {
    pub async fn from_gltf_async(path: &str) -> Result<Self, String> {
        let data = pi_hal::file::load_from_path(&Atom::from(path))
            .await
            .unwrap();
        match Gltf::from_slice_without_validation(&data) {
            Ok(gltf) => {
                return Ok(Self {
                    gltf,
                    path: PathBuf::from_str(path).unwrap(),
                });
            }
            Err(err) => {
                return Err(format!(
                    "create gltf  failed!! path: {}, reason: {:?}",
                    path, err
                ))
            }
        };
    }

    pub async fn from_cbor(path: &str) -> Result<Self, String> {
        let data = pi_hal::file::load_from_path(&Atom::from(path))
            .await
            .unwrap();

        match serde_cbor::from_slice(&data) {
            Ok(root) => {
                let document = Document::from_json_without_validation(root);
                let gltf = Gltf {
                    blob: Some(data),
                    document,
                };
                return Ok(Self {
                    gltf,
                    path: PathBuf::from_str(path).unwrap(),
                });
            }
            Err(err) => {
                return Err(format!(
                    "create gltf form cbor failed!! path: {}, reason: {:?}",
                    path, err
                ))
            }
        }
    }

    pub async fn init_async(&self, factory: &mut impl InterfaceGLTFLoader) {
        let gltf = &self.gltf;
        let buffer_data = self.load_buffer_async().await;

        let mut materials = vec![];
        for material in self.gltf.materials() {
            materials.push(material);
        }
        let mut node_index = 0;
        for node in gltf.nodes() {
            let entity = match node.transform() {
                gltf::scene::Transform::Matrix { matrix } => {
                    factory.gltf_create_node(None, None, None, None, Some(matrix))
                }
                gltf::scene::Transform::Decomposed {
                    translation,
                    rotation,
                    scale,
                } => factory.gltf_create_node(
                    Some(translation),
                    Some(scale),
                    None,
                    Some(rotation),
                    None,
                ),
            };

            if let Some(mesh) = node.mesh() {
                // TODO: 取 layer 参数
                // factory.gltf_layer_mask(entity, layer);

                // 用name加索引作为uuid
                let name = match mesh.name() {
                    Some(str) => str.to_string(),
                    // TODO 生成独一无二字符
                    None => "".to_string(),
                };

                let mut primitives_index = 0;
                let mut attributes = vec![];

                for primitive in mesh.primitives() {
                    let rbounding_box = primitive.bounding_box();
                    factory.gltf_bounding_info(entity, rbounding_box.min, rbounding_box.max);

                    let index = AtomicUsize::new(0);
                    let reader = primitive.reader(|buffer| {
                        index.store(buffer.index(), Ordering::Relaxed);
                        Some(&buffer_data[buffer.index()].1)
                    });

                    if let Some(positions) = reader
                        .read_positions()
                        .map(|v| v.collect::<Vec<[f32; 3]>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; positions:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&positions).to_vec();

                            factory.gltf_apply_vertices_buffer(
                                entity,
                                EGLTFVertexDataKind::Position,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::VertexFormat::Float32x3,
                            );

                            factory.gltf_create_buffer(id, data);

                            attributes.push(GLTFVertexAttribute {
                                kind: EGLTFVertexDataKind::Position,
                                format: wgpu::VertexFormat::Float32x3,
                            })
                        }
                    }

                    if let Some(indices) = reader
                        .read_indices()
                        .map(|v| v.into_u32().collect::<Vec<u32>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; indices:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&indices).to_vec();

                            factory.gltf_apply_indices_buffer(
                                entity,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::IndexFormat::Uint32,
                            );

                            factory.gltf_create_buffer(id, data);
                        }
                    };

                    if let Some(normals) =
                        reader.read_normals().map(|v| v.collect::<Vec<[f32; 3]>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; normals:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&normals).to_vec();

                            factory.gltf_apply_vertices_buffer(
                                entity,
                                EGLTFVertexDataKind::Normal,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::VertexFormat::Float32x3,
                            );

                            factory.gltf_create_buffer(id, data);

                            attributes.push(GLTFVertexAttribute {
                                kind: EGLTFVertexDataKind::Normal,
                                format: wgpu::VertexFormat::Float32x3,
                            })
                        }
                    }

                    if let Some(tangents) =
                        reader.read_tangents().map(|v| v.collect::<Vec<[f32; 4]>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; tangents:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&tangents).to_vec();

                            factory.gltf_apply_vertices_buffer(
                                entity,
                                EGLTFVertexDataKind::Tangent,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::VertexFormat::Float32x4,
                            );

                            factory.gltf_create_buffer(id, data);

                            attributes.push(GLTFVertexAttribute {
                                kind: EGLTFVertexDataKind::Tangent,
                                format: wgpu::VertexFormat::Float32x4,
                            })
                        }
                    }

                    if let Some(colors) = reader
                        .read_colors(0)
                        .map(|v| v.into_rgba_f32().collect::<Vec<[f32; 4]>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; colors:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&colors).to_vec();

                            factory.gltf_apply_vertices_buffer(
                                entity,
                                EGLTFVertexDataKind::Color4,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::VertexFormat::Float32x4,
                            );

                            factory.gltf_create_buffer(id, data);

                            attributes.push(GLTFVertexAttribute {
                                kind: EGLTFVertexDataKind::Color4,
                                format: wgpu::VertexFormat::Float32x4,
                            })
                        }
                    }

                    if let Some(joints) = reader
                        .read_joints(0)
                        .map(|v| v.into_u16().collect::<Vec<[u16; 4]>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; joints:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&joints).to_vec();

                            factory.gltf_apply_vertices_buffer(
                                entity,
                                EGLTFVertexDataKind::MatricesIndices,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::VertexFormat::Uint16x4,
                            );

                            factory.gltf_create_buffer(id, data);

                            attributes.push(GLTFVertexAttribute {
                                kind: EGLTFVertexDataKind::MatricesIndices,
                                format: wgpu::VertexFormat::Uint16x4,
                            })
                        }
                    }

                    if let Some(weights) = reader
                        .read_weights(0)
                        .map(|v| v.into_f32().collect::<Vec<[f32; 4]>>())
                    {
                        let id = Atom::from(format!(
                            "{:?}; weights:{}{}",
                            buffer_data[index.load(Ordering::Relaxed)].0,
                            node_index,
                            primitives_index
                        ));

                        if !factory.gltf_check_buffer(&id) {
                            let data = bytemuck::cast_slice(&weights).to_vec();

                            factory.gltf_apply_vertices_buffer(
                                entity,
                                EGLTFVertexDataKind::MatricesWeights,
                                id.clone(),
                                0..data.len() as u64,
                                wgpu::VertexFormat::Float32x4,
                            );

                            factory.gltf_create_buffer(id, data);

                            attributes.push(GLTFVertexAttribute {
                                kind: EGLTFVertexDataKind::MatricesWeights,
                                format: wgpu::VertexFormat::Float32x4,
                            })
                        }
                    }

                    for i in 0..8 {
                        if let Some(uvs) = reader
                            .read_tex_coords(i)
                            .map(|v| v.into_f32().collect::<Vec<[f32; 2]>>())
                        {
                            let vertex_data_kind = match i {
                                0 => EGLTFVertexDataKind::UV,
                                1 => EGLTFVertexDataKind::UV2,
                                2 => EGLTFVertexDataKind::UV3,
                                3 => EGLTFVertexDataKind::UV4,
                                4 => EGLTFVertexDataKind::UV5,
                                5 => EGLTFVertexDataKind::UV6,
                                6 => EGLTFVertexDataKind::UV7,
                                7 => EGLTFVertexDataKind::UV8,
                                _ => panic!("uv not surpport overflow 8"),
                            };

                            let id = Atom::from(format!(
                                "{:?}; uv{}:{}{}",
                                i,
                                buffer_data[index.load(Ordering::Relaxed)].0,
                                node_index,
                                primitives_index
                            ));

                            if !factory.gltf_check_buffer(&id) {
                                let data = bytemuck::cast_slice(&uvs).to_vec();

                                factory.gltf_apply_vertices_buffer(
                                    entity,
                                    vertex_data_kind,
                                    id.clone(),
                                    0..data.len() as u64,
                                    wgpu::VertexFormat::Float32x2,
                                );

                                factory.gltf_create_buffer(id, data);

                                attributes.push(GLTFVertexAttribute {
                                    kind: vertex_data_kind,
                                    format: wgpu::VertexFormat::Float32x2,
                                })
                            }
                        }
                    }

                    if let Some(_) = primitive
                        .material()
                        .index()
                        .and_then(|i| materials.get(i).cloned())
                    {}

                    primitives_index += 1;
                }
            }

            if let Some(skin) = node.skin() {
                let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()].1));

                // skin.

                if let Some(bones) = reader.read_inverse_bind_matrices() {
                    let bones = bones.collect::<Vec<[[f32; 4]; 4]>>();
                    factory.gltf_apply_skin(entity, bones);
                }
            }
            node_index += 1;
        }
    }

    async fn load_buffer_async(&self) -> Vec<(String, Vec<u8>)> {
        let mut buffer_data: Vec<(String, Vec<u8>)> = Vec::new();
        for buffer in self.gltf.buffers() {
            println!("source: {:?}", buffer.source());
            match buffer.source() {
                gltf::buffer::Source::Uri(uri) => {
                    if uri.starts_with("data:") {
                        if let Some(index) = uri.find(',') {
                            let base64_buffer = uri.split_at(index + 1).1;
                            println!("base64_buffer: {}", base64_buffer);
                            let buffer = base64::decode(base64_buffer).unwrap();
                            buffer_data.push(("".to_string(), buffer));
                        }
                    }
                }
                gltf::buffer::Source::Bin => {
                    let r = self.gltf.blob.as_deref().unwrap();
                    // TODO: 不會用base64 數據
                    buffer_data.push(("".to_string(), r.into()));
                }
            }
        }

        buffer_data
    }
}



