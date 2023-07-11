use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};
use gltf::{Document, Gltf};
use pi_atom::Atom;
use pi_engine_shell::prelude::*;
use pi_gltf as gltf;
use pi_render::rhi::{IndexFormat, VertexFormat};
use pi_scene_context::prelude::*;

use crate::{
    extras::particle::{MeshParticleMeshID, Particle},
    interface::{GLTFCommands, GLTFAPI},
};

pub struct GltfLoader {
    gltf: Gltf,
    _path: PathBuf,
}

impl GltfLoader {
    pub async fn from_gltf_async(path: &str) -> Result<Self, String> {
        let data = pi_hal::file::load_from_url(&Atom::from(path))
            .await
            .unwrap();
        match Gltf::from_slice_without_validation(&data) {
            Ok(gltf) => {
                return Ok(Self {
                    gltf,
                    _path: PathBuf::from_str(path).unwrap(),
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
        let data = pi_hal::file::load_from_url(&Atom::from(path))
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
                    _path: PathBuf::from_str(path).unwrap(),
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

    pub async fn load_buffer_async(&self) -> Vec<(String, Vec<u8>)> {
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
                    } else {
                        let path = self._path.parent().unwrap().join(uri);
                        // println!("path: {:?}", path);
                        let data = pi_hal::file::load_from_url(&Atom::from(path.to_str().unwrap()))
                            .await
                            .unwrap();
                        buffer_data.push((uri.to_string(), data));
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

pub struct OpsGLTFLoaded(pub Entity, pub GltfLoader, pub Vec<(String, Vec<u8>)>);
impl OpsGLTFLoaded {
    pub fn ops(scene: Entity, loaded: GltfLoader, buffer: Vec<(String, Vec<u8>)>) -> Self {
        Self(scene, loaded, buffer)
    }
}
pub type ActionListGLTFLoaded = ActionList<OpsGLTFLoaded>;

pub fn sys_gltf_decode(
    mut loadeds: ResMut<ActionListGLTFLoaded>,
    mut cmd: GLTFCommands,
    mut commands: Commands,
) {
    loadeds
        .drain()
        .drain(..)
        .for_each(|OpsGLTFLoaded(scene_id, loader, buffer_data)| {
            let mut factory = GLTFAPI {
                scene_id,
                commands: &mut cmd,
            };
            let gltf = &loader.gltf;
            // let images = gltf.images();
            let root_path = loader._path;

            let mut materials = vec![];
            for material in gltf.materials() {
                materials.push(material);
            }
            let textures = gltf.textures().collect::<Vec<pi_gltf::Texture>>();

            let mut node_map = HashMap::new();
            let mut node_index = 0;

            let root = commands.spawn_empty().id();
            factory
                .commands
                .transformcmds
                .tree
                .push(OpsTransformNodeParent::ops(root, scene_id));

            // factory.gltf_transform(root, None, None, None, None, None, );
            for node in gltf.nodes() {
                let node_entity = commands.spawn_empty().id();
                factory
                    .commands
                    .transformcmds
                    .tree
                    .push(OpsTransformNodeParent::ops(node_entity, root));
                factory
                    .commands
                    .transformcmds
                    .create
                    .push(OpsTransformNode::ops(
                        scene_id,
                        node_entity,
                        String::from("name"),
                    ));

                // factory.gltf_create_skin(bone_root, bones);

                let mut t_translation = None;
                let mut t_scaling = None;
                let t_rotation = None;
                let mut t_rotation_quternion = None;
                let mut t_matrix = None;
                match node.transform() {
                    gltf::scene::Transform::Matrix { matrix } => {
                        t_matrix = Some(matrix);
                        // println!("====== node matrix: {:?}", matrix);
                        // factory.gltf_transform(entity, None, None, None, None, Some(matrix), )
                    }
                    gltf::scene::Transform::Decomposed {
                        translation,
                        rotation,
                        scale,
                    } => {
                        println!(
                            "====== node translation: {:?}, rotation: {:?}, scale: {:?}",
                            translation, rotation, scale
                        );

                        t_translation = Some(translation);
                        t_scaling = Some(scale);
                        t_rotation_quternion = Some(rotation);
                    }
                };
                factory.gltf_transform(
                    node_entity,
                    t_translation,
                    t_scaling,
                    t_rotation,
                    t_rotation_quternion,
                    t_matrix,
                );

                println!("node.index(): {}", node.index());
                node_map.insert(node.index(), node_entity.clone());

                if let Some(mesh) = node.mesh() {
                    // TODO: 取 layer 参数
                    // factory.gltf_layer_mask(entity, layer);

                    // 用name加索引作为uuid
                    let _name = match mesh.name() {
                        Some(str) => str.to_string(),
                        // TODO 生成独一无二字符
                        None => "".to_string(),
                    };

                    for primitive in mesh.primitives() {
                        let mesh_entity = commands.spawn_empty().id();
                        factory.commands.meshcmds.create.push(OpsMeshCreation::ops(
                            scene_id,
                            mesh_entity,
                            String::from("name"),
                        ));

                        let mut primitives_index = 0;
                        let mut vertex_buffer_desc = vec![];
                        let mut indices_desc = None;

                        // let rbounding_box = primitive.bounding_box();
                        // factory.gltf_bounding_info(entity.clone(), rbounding_box.min, rbounding_box.max);

                        let index = AtomicUsize::new(0);
                        let reader = primitive.reader(|buffer| {
                            index.store(buffer.index(), Ordering::Relaxed);
                            Some(&buffer_data[buffer.index()].1)
                        });

                        if let Some(indices) = reader
                            .read_indices()
                            .map(|v| v.into_u32().collect::<Vec<u32>>())
                        {
                            // let indices = indices.iter().map(|v| *v as u16).collect::<Vec<u16>>();
                            println!("indices: {:?}", indices);
                            let id = format!(
                                "{:?}; indices:{}{}",
                                buffer_data[index.load(Ordering::Relaxed)].0,
                                node_index,
                                primitives_index
                            );

                            let data = bytemuck::cast_slice(&indices).to_vec();

                            factory.gltf_create_indices_buffer(id.as_str(), data);

                            indices_desc = Some(IndicesBufferDesc {
                                format: IndexFormat::Uint32,
                                buffer_range: None,
                                buffer: id.into(),
                            });
                        };

                        primitive.attributes().for_each(|v| match v.0 {
                            gltf::Semantic::Positions => {
                                if let Some(positions) = reader
                                    .read_positions()
                                    .map(|v| v.collect::<Vec<[f32; 3]>>())
                                {
                                    println!("positions: {:?}", positions);
                                    let id = format!(
                                        "{:?}; positions:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&positions).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: EVertexDataKind::Position,
                                            format: VertexFormat::Float32x3,
                                        }],
                                    ));
                                }
                            }
                            gltf::Semantic::Normals => {
                                if let Some(normals) =
                                    reader.read_normals().map(|v| v.collect::<Vec<[f32; 3]>>())
                                {
                                    println!("normals: {:?}", normals);
                                    let id = format!(
                                        "{:?}; normals:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&normals).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: EVertexDataKind::Normal,
                                            format: VertexFormat::Float32x3,
                                        }],
                                    ));
                                }
                            }
                            gltf::Semantic::Tangents => {
                                if let Some(tangents) =
                                    reader.read_tangents().map(|v| v.collect::<Vec<[f32; 4]>>())
                                {
                                    println!("tangents: {:?}", tangents);
                                    let id = format!(
                                        "{:?}; tangents:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&tangents).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: EVertexDataKind::Tangent,
                                            format: VertexFormat::Float32x4,
                                        }],
                                    ));
                                }
                            }
                            gltf::Semantic::Colors(set) => {
                                if let Some(colors) = reader
                                    .read_colors(set)
                                    .map(|v| v.into_rgba_f32().collect::<Vec<[f32; 4]>>())
                                {
                                    println!("colors: {:?}", colors);
                                    let id = format!(
                                        "{:?}; colors:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&colors).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: EVertexDataKind::Color4,
                                            format: VertexFormat::Float32x4,
                                        }],
                                    ));
                                }
                            }
                            gltf::Semantic::TexCoords(set) => {
                                if let Some(uvs) = reader
                                    .read_tex_coords(set)
                                    .map(|v| v.into_f32().collect::<Vec<[f32; 2]>>())
                                {
                                    println!("uvs{}: {:?}", set, uvs);
                                    let vertex_data_kind = match set {
                                        0 => EVertexDataKind::UV,
                                        1 => EVertexDataKind::UV2,
                                        2 => EVertexDataKind::UV3,
                                        3 => EVertexDataKind::UV4,
                                        4 => EVertexDataKind::UV5,
                                        5 => EVertexDataKind::UV6,
                                        _ => panic!("uv not surpport overflow 8"),
                                    };

                                    let id = format!(
                                        "{}; uv{}:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        set,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&uvs).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: vertex_data_kind,
                                            format: VertexFormat::Float32x2,
                                        }],
                                    ));
                                }
                            }
                            gltf::Semantic::Joints(set) => {
                                if let Some(joints) = reader
                                    .read_joints(set)
                                    .map(|v| v.into_u16().collect::<Vec<[u16; 4]>>())
                                {
                                    println!("joints: {:?}", joints);

                                    let (vertex_data_kind, format) = match set {
                                        0 => (
                                            EVertexDataKind::MatricesIndices,
                                            VertexFormat::Uint16x4,
                                        ),
                                        1 => (
                                            EVertexDataKind::MatricesIndices1,
                                            VertexFormat::Uint16x4,
                                        ),
                                        2 => (
                                            EVertexDataKind::MatricesIndices2,
                                            VertexFormat::Uint16x4,
                                        ),
                                        3 => (
                                            EVertexDataKind::MatricesIndices3,
                                            VertexFormat::Uint16x4,
                                        ),
                                        _ => panic!("uv not surpport overflow 4"),
                                    };

                                    let id = format!(
                                        "{:?}; joints{}:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        set,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&joints).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: vertex_data_kind,
                                            format,
                                        }],
                                    ));
                                }
                            }
                            gltf::Semantic::Weights(set) => {
                                if let Some(joints) = reader
                                    .read_weights(set)
                                    .map(|v| v.into_f32().collect::<Vec<[f32; 4]>>())
                                {
                                    println!("joints: {:?}", joints);

                                    let (vertex_data_kind, format) = match set {
                                        0 => (
                                            EVertexDataKind::MatricesWeights,
                                            VertexFormat::Float32x4,
                                        ),
                                        1 => (
                                            EVertexDataKind::MatricesWeights1,
                                            VertexFormat::Float32,
                                        ),
                                        2 => (
                                            EVertexDataKind::MatricesWeights2,
                                            VertexFormat::Float32x2,
                                        ),
                                        3 => (
                                            EVertexDataKind::MatricesWeights3,
                                            VertexFormat::Float32x3,
                                        ),
                                        _ => panic!("uv not surpport overflow 4"),
                                    };

                                    let id = format!(
                                        "{:?}; weights{}:{}{}",
                                        buffer_data[index.load(Ordering::Relaxed)].0,
                                        set,
                                        node_index,
                                        primitives_index
                                    );

                                    let data = bytemuck::cast_slice(&joints).to_vec();
                                    factory.gltf_create_buffer(id.as_str(), data);

                                    vertex_buffer_desc.push(VertexBufferDesc::vertices(
                                        id.into(),
                                        None,
                                        vec![VertexAttribute {
                                            kind: vertex_data_kind,
                                            format: format,
                                        }],
                                    ));
                                }
                            }
                            _ => {}
                        });

                        if let Some(material) = primitive
                            .material()
                            .index()
                            .and_then(|i| materials.get(i).cloned())
                        {
                            if let Some(extras) = material.extras() {
                                println!("material extras: {:?}", extras);
                                let idmat = commands.spawn_empty().id();
                                factory.gltf_extras_material(
                                    mesh_entity,
                                    idmat,
                                    extras,
                                    &textures,
                                    &root_path,
                                );

                                // if let Some(info) = material.emissive_texture() {
                                //     let tex = info.texture();
                                //     match tex.source().source() {
                                //         image::Source::View {
                                //             view: _,
                                //             mime_type: _,
                                //         } => todo!(),
                                //         image::Source::Uri { uri, mime_type: _ } => {
                                //             factory.commands.matcmds.texture.push(OpsUniformTexture::ops(
                                //                 idmat,
                                //                 UniformTextureWithSamplerParam {
                                //                     slotname: Atom::from(BlockEmissiveTexture::KEY_TEX),
                                //                     filter: true,
                                //                     sample: KeySampler::default(),
                                //                     url: EKeyTexture::from(uri),
                                //                 },
                                //             ));
                                //         }
                                //     }
                                // }
                            }
                        } else {
                            factory.gltf_default_material(
                                mesh_entity.clone(),
                                commands.spawn_empty().id(),
                            );
                        }

                        if let Some(extras) = node.extras() {
                            if let Some(mesh_particle_cfg) = extras.get("meshParticle") {
                                let mp = factory.gltf_extras_particle(mesh_particle_cfg);
                                let mp_entity = node_entity;
                                commands
                                    .entity(mp_entity)
                                    .insert(Particle(mp))
                                    .insert(MeshParticleMeshID(mesh_entity));
                                vertex_buffer_desc.push(VertexBufferDesc::instance_world_matrix());
                                vertex_buffer_desc.push(VertexBufferDesc::instance_color());
                                vertex_buffer_desc.push(VertexBufferDesc::instance_tilloff());

                                factory
                                    .commands
                                    .transformcmds
                                    .tree
                                    .push(OpsTransformNodeParent::ops(mesh_entity, scene_id));
                            } else {
                                factory
                                    .commands
                                    .transformcmds
                                    .tree
                                    .push(OpsTransformNodeParent::ops(mesh_entity, node_entity));
                            }
                        } else {
                            factory
                                .commands
                                .transformcmds
                                .tree
                                .push(OpsTransformNodeParent::ops(mesh_entity, node_entity));
                        }

                        println!("vertex_buffer_desc: {:?}", vertex_buffer_desc);
                        let id_geo = commands.spawn_empty().id();
                        factory.gltf_geometry(
                            mesh_entity.clone(),
                            id_geo,
                            vertex_buffer_desc,
                            indices_desc,
                        );

                        primitives_index += 1;
                    }
                }
                if let Some(_skin) = node.skin() {
                    // TODO: 添加姿态
                    // let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()].1));

                    // if let Some(bones) = reader.read_inverse_bind_matrices() {
                    //     let bones = bones.collect::<Vec<[[f32; 4]; 4]>>();
                    //     let bones = factory.gltf_create_skin(scene_id.clone(), entity.clone(), bones);
                    //     factory.gltf_apply_skin(entity.clone(), bones);
                    // }
                }

                node_index += 1;
            }
            for node in gltf.nodes() {
                if let Some(skin) = node.skin() {
                    // let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()].1));
                    if let Some(entity) = node_map.get(&node.index()) {
                        let mut bones = vec![];
                        skin.joints().for_each(|v| {
                            if let Some(idx) = node_map.get(&v.index()) {
                                bones.push(*idx)
                            }
                        });
                        let root_bone = bones[0].clone();
                        // bones.push(root_bone);
                        // bones.push(root_bone);
                        println!("Skin: {:?}, {:?}", entity, bones);
                        let skin = commands.spawn_empty().id();
                        let skin = factory.gltf_create_skin(root_bone, bones, skin);
                        factory.gltf_apply_skin(*entity, skin);
                    }
                }
                node.children().for_each(|child| {
                    if let Some(entity) = node_map.get(&node.index()) {
                        if let Some(child) = node_map.get(&child.index()) {
                            // println!("{}'s parent: {}", v.index(), node.index());
                            factory
                                .commands
                                .transformcmds
                                .tree
                                .push(OpsTransformNodeParent::ops(*child, *entity));
                        }
                    }
                });
            }

            let mut animation_index = 0;
            for animation in gltf.animations() {
                let key_animegroup = Atom::from(format!("anim{}", animation_index,));
                let id_group = factory.gltf_create_animation_group(root, &key_animegroup);

                let mut channel_index = 0;

                for channel in animation.channels() {
                    let node = channel.target().node();
                    println!("animation!!! node.index(): {}", node.index());

                    if let Some(node_id) = node_map.get(&node.index()) {
                        let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()].1));

                        if let Some(inputs) = reader.read_inputs() {
                            if let Some(outputs) = reader.read_outputs() {
                                let key_curve = Atom::from(format!(
                                    "anim{}channel{}",
                                    animation_index, channel_index
                                ));

                                let assets_curve = factory
                                    .gltf_create_assets_curve(key_curve, channel, inputs, outputs);
                                factory.gltf_create_target_animation(
                                    assets_curve,
                                    scene_id.clone(),
                                    node_id.clone(),
                                    id_group,
                                );
                            }
                        }
                    }
                    channel_index += 1;
                }

                factory.gltf_start_animation_group(root, id_group);
                animation_index += 1;
            }
        });
}
