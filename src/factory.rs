use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicUsize, Ordering},
};

use gltf::{Document, Gltf};
use nalgebra::Quaternion;
use pi_atom::Atom;
use pi_curves::curve::frame_curve::{frames::interplate_frame_values_step, FrameCurve};

use pi_engine_shell::object::ObjectID;
use pi_gltf as gltf;
use pi_render::renderer::{
    attributes::{EVertexDataKind, VertexAttribute},
    indices::IndicesBufferDesc,
    vertex_buffer_desc::VertexBufferDesc,
};
use pi_scene_context::transforms::transform_node::{
    LocalPosition, LocalRotationQuaternion, LocalScaling,
};
use pi_scene_math::{Quaternion as MQuaternion, Vector3};

use crate::interface::InterfaceGLTFLoader;

pub struct GltfLoader {
    gltf: Gltf,
    _path: PathBuf,
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

pub fn gltf_decode(
    loader: &GltfLoader,
    factory: &impl InterfaceGLTFLoader<ObjectID>,
    buffer_data: Vec<(String, Vec<u8>)>,
    scene_id: ObjectID,
) {
    let gltf = &loader.gltf;

    let mut materials = vec![];
    for material in gltf.materials() {
        materials.push(material);
    }

    let mut node_map = HashMap::new();
    let mut node_index = 0;

    let root = factory.gltf_create_node(None, None, None, None, None, scene_id.clone());

    for node in gltf.nodes() {
        let entity = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => {
                println!("====== node matrix: {:?}", matrix);
                factory.gltf_create_node(None, None, None, None, Some(matrix), scene_id.clone())
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
                factory.gltf_create_node(
                    Some(translation),
                    Some(scale),
                    None,
                    Some(rotation),
                    None,
                    scene_id.clone(),
                )
            }
        };
        println!("node.index(): {}", node.index());
        node_map.insert(node.index(), entity.clone());

        if let Some(mesh) = node.mesh() {
            // TODO: 取 layer 参数
            // factory.gltf_layer_mask(entity, layer);

            // 用name加索引作为uuid
            let _name = match mesh.name() {
                Some(str) => str.to_string(),
                // TODO 生成独一无二字符
                None => "".to_string(),
            };

            let mut primitives_index = 0;
            let mut vertex_buffer_desc = vec![];
            let mut indices_desc = None;

            for primitive in mesh.primitives() {
                let rbounding_box = primitive.bounding_box();
                factory.gltf_bounding_info(entity.clone(), rbounding_box.min, rbounding_box.max);

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

                    factory.gltf_create_buffer(id.as_str(), data);

                    indices_desc = Some(IndicesBufferDesc {
                        format: wgpu::IndexFormat::Uint32,
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
                                    format: wgpu::VertexFormat::Float32x3,
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
                                    format: wgpu::VertexFormat::Float32x3,
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
                                    format: wgpu::VertexFormat::Float32x4,
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
                                    format: wgpu::VertexFormat::Float32x4,
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
                                    format: wgpu::VertexFormat::Float32x2,
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
                                    wgpu::VertexFormat::Uint16x4,
                                ),
                                1 => (
                                    EVertexDataKind::MatricesIndices1,
                                    wgpu::VertexFormat::Uint16x4,
                                ),
                                2 => (
                                    EVertexDataKind::MatricesIndices2,
                                    wgpu::VertexFormat::Uint16x4,
                                ),
                                3 => (
                                    EVertexDataKind::MatricesIndices3,
                                    wgpu::VertexFormat::Uint16x4,
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
                                    wgpu::VertexFormat::Float32x4,
                                ),
                                1 => (
                                    EVertexDataKind::MatricesWeights1,
                                    wgpu::VertexFormat::Float32,
                                ),
                                2 => (
                                    EVertexDataKind::MatricesWeights2,
                                    wgpu::VertexFormat::Float32x2,
                                ),
                                3 => (
                                    EVertexDataKind::MatricesWeights3,
                                    wgpu::VertexFormat::Float32x3,
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
                });

                if let Some(material) = primitive
                    .material()
                    .index()
                    .and_then(|i| materials.get(i).cloned())
                {
                    let materialid = factory.gltf_create_unlit_material();
                    if let Some(info) = material.emissive_texture() {
                        let tex = info.texture();
                        match tex.source().source() {
                            gltf::image::Source::View {
                                view: _,
                                mime_type: _,
                            } => todo!(),
                            gltf::image::Source::Uri { uri, mime_type: _ } => {
                                factory.gltf_emissive_texture(materialid.clone(), Atom::from(uri))
                            }
                        }
                    }

                    factory.gltf_use_material(entity.clone(), materialid)
                } else {
                    factory.gltf_default_material(entity.clone());
                }

                primitives_index += 1;
            }
            println!("vertex_buffer_desc: {:?}", vertex_buffer_desc);
            factory.gltf_geometry(entity.clone(), vertex_buffer_desc, indices_desc);
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
                bones.push(root_bone);
                bones.push(root_bone);
                println!("Skin: {:?}, {:?}", entity, bones);
                let skin = factory.gltf_create_skin(root_bone, bones);
                factory.gltf_apply_skin(*entity, skin);
            }
        }
    }

    let mut animation_index = 0;
    for animation in gltf.animations() {
        let key_animegroup = Atom::from(format!("anim{}", animation_index,));
        factory.gltf_create_animation_group(root, &key_animegroup);

        let mut channel_index = 0;

        for channel in animation.channels() {
            let node = channel.target().node();
            println!("animation!!! node.index(): {}", node.index());

            if let Some(node_id) = node_map.get(&node.index()) {
                let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()].1));

                if let Some(mut inputs) = reader.read_inputs() {
                    if let Some(outputs) = reader.read_outputs() {
                        let key_curve =
                            Atom::from(format!("anim{}channel{}", animation_index, channel_index,));
                        match outputs {
                            gltf::animation::util::ReadOutputs::Translations(mut translations) => {
                                match channel.sampler().interpolation() {
                                    gltf::animation::Interpolation::Linear => {
                                        let asset_curve = if let Some(curve) = factory
                                            .gltf_check_anim_curve::<LocalPosition>(&key_curve)
                                        {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_frame_values(1000);

                                            for (input, translation) in inputs.zip(translations) {
                                                curve.curve_frame_values_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalPosition(Vector3::new(
                                                        translation[0],
                                                        translation[1],
                                                        translation[2],
                                                    )),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            node_id.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                    gltf::animation::Interpolation::Step => {
                                        // panic!("Not supported yet Step mode!!!")

                                        let asset_curve = if let Some(curve) = factory
                                            .gltf_check_anim_curve::<LocalPosition>(&key_curve)
                                        {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_frame_values(1000);
                                            curve.call = interplate_frame_values_step;

                                            for (input, translation) in inputs.zip(translations) {
                                                curve.curve_frame_values_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalPosition(Vector3::new(
                                                        translation[0],
                                                        translation[1],
                                                        translation[2],
                                                    )),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            node_id.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                    gltf::animation::Interpolation::CubicSpline => {
                                        let asset_curve = if let Some(curve) = factory
                                            .gltf_check_anim_curve::<LocalPosition>(&key_curve)
                                        {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_cubic_spline(1000);

                                            for i in 0..inputs.len() {
                                                let input = inputs.nth(i).unwrap();

                                                let input_tangent =
                                                    translations.nth(i * 3 + 0).unwrap();
                                                let keyframe = translations.nth(i * 3 + 1).unwrap();
                                                let output_tangent =
                                                    translations.nth(i * 3 + 2).unwrap();

                                                curve.curve_cubic_splice_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalPosition(Vector3::new(
                                                        keyframe[0],
                                                        keyframe[1],
                                                        keyframe[2],
                                                    )),
                                                    LocalPosition(Vector3::new(
                                                        input_tangent[0],
                                                        input_tangent[1],
                                                        input_tangent[2],
                                                    )),
                                                    LocalPosition(Vector3::new(
                                                        output_tangent[0],
                                                        output_tangent[1],
                                                        output_tangent[2],
                                                    )),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            node_id.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                }
                            }
                            gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                                match channel.sampler().interpolation() {
                                    gltf::animation::Interpolation::Linear => {
                                        let asset_curve = if let Some(curve) =
                                            factory
                                                .gltf_check_anim_curve::<LocalRotationQuaternion>(
                                                    &key_curve,
                                                ) {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_frame_values(1000);

                                            for (input, rotation) in
                                                inputs.zip(rotations.into_f32())
                                            {
                                                println!("========= Rotations Linear!!! time {}, keyframe: {:?}", input, rotation);
                                                curve.curve_frame_values_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalRotationQuaternion(
                                                        MQuaternion::from_quaternion(
                                                            Quaternion::new(
                                                                rotation[0],
                                                                rotation[1],
                                                                rotation[2],
                                                                rotation[3],
                                                            ),
                                                        ),
                                                    ),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            root.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                    gltf::animation::Interpolation::Step => {
                                        let asset_curve = if let Some(curve) =
                                            factory
                                                .gltf_check_anim_curve::<LocalRotationQuaternion>(
                                                    &key_curve,
                                                ) {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_frame_values(1000);
                                            curve.call = interplate_frame_values_step;

                                            for (input, rotation) in
                                                inputs.zip(rotations.into_f32())
                                            {
                                                curve.curve_frame_values_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalRotationQuaternion(
                                                        MQuaternion::from_quaternion(
                                                            Quaternion::new(
                                                                rotation[0],
                                                                rotation[1],
                                                                rotation[2],
                                                                rotation[3],
                                                            ),
                                                        ),
                                                    ),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            root.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                    gltf::animation::Interpolation::CubicSpline => {
                                        let asset_curve = if let Some(curve) =
                                            factory
                                                .gltf_check_anim_curve::<LocalRotationQuaternion>(
                                                    &key_curve,
                                                ) {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_cubic_spline(1000);
                                            let mut rotations = rotations.into_f32();
                                            for i in 0..inputs.len() {
                                                let input = inputs.nth(i).unwrap();

                                                let input_tangent =
                                                    rotations.nth(i * 3 + 0).unwrap();
                                                let keyframe = rotations.nth(i * 3 + 1).unwrap();
                                                let output_tangent =
                                                    rotations.nth(i * 3 + 2).unwrap();

                                                curve.curve_cubic_splice_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalRotationQuaternion(
                                                        MQuaternion::from_quaternion(
                                                            Quaternion::new(
                                                                keyframe[0],
                                                                keyframe[1],
                                                                keyframe[2],
                                                                keyframe[3],
                                                            ),
                                                        ),
                                                    ),
                                                    LocalRotationQuaternion(
                                                        MQuaternion::from_quaternion(
                                                            Quaternion::new(
                                                                input_tangent[0],
                                                                input_tangent[1],
                                                                input_tangent[2],
                                                                input_tangent[3],
                                                            ),
                                                        ),
                                                    ),
                                                    LocalRotationQuaternion(
                                                        MQuaternion::from_quaternion(
                                                            Quaternion::new(
                                                                output_tangent[0],
                                                                output_tangent[1],
                                                                output_tangent[2],
                                                                output_tangent[3],
                                                            ),
                                                        ),
                                                    ),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            root.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                }
                            }
                            gltf::animation::util::ReadOutputs::Scales(mut scales) => {
                                match channel.sampler().interpolation() {
                                    gltf::animation::Interpolation::Linear => {
                                        let asset_curve = if let Some(curve) = factory
                                            .gltf_check_anim_curve::<LocalScaling>(&key_curve)
                                        {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_frame_values(1000);

                                            for (input, scale) in inputs.zip(scales) {
                                                curve.curve_frame_values_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalScaling(Vector3::new(
                                                        scale[0], scale[1], scale[2],
                                                    )),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            root.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                    gltf::animation::Interpolation::Step => {
                                        let asset_curve = if let Some(curve) = factory
                                            .gltf_check_anim_curve::<LocalScaling>(&key_curve)
                                        {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_frame_values(1000);
                                            curve.call = interplate_frame_values_step;

                                            for (input, scale) in inputs.zip(scales) {
                                                curve.curve_frame_values_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalScaling(Vector3::new(
                                                        scale[0], scale[1], scale[2],
                                                    )),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            root.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                    gltf::animation::Interpolation::CubicSpline => {
                                        let asset_curve = if let Some(curve) = factory
                                            .gltf_check_anim_curve::<LocalScaling>(&key_curve)
                                        {
                                            curve
                                        } else {
                                            let mut curve = FrameCurve::curve_cubic_spline(1000);
                                            // let mut rotations = rotations.into_f32();
                                            for i in 0..inputs.len() {
                                                let input = inputs.nth(i).unwrap();

                                                let input_tangent = scales.nth(i * 3 + 0).unwrap();
                                                let keyframe = scales.nth(i * 3 + 1).unwrap();
                                                let output_tangent = scales.nth(i * 3 + 2).unwrap();

                                                curve.curve_cubic_splice_frame(
                                                    (input * 1000.0) as u16,
                                                    LocalScaling(Vector3::new(
                                                        keyframe[0],
                                                        keyframe[1],
                                                        keyframe[2],
                                                    )),
                                                    LocalScaling(Vector3::new(
                                                        input_tangent[0],
                                                        input_tangent[1],
                                                        input_tangent[2],
                                                    )),
                                                    LocalScaling(Vector3::new(
                                                        output_tangent[0],
                                                        output_tangent[1],
                                                        output_tangent[2],
                                                    )),
                                                );
                                            }

                                            factory.gltf_creat_anim_curve(&key_curve, curve)
                                        };

                                        factory.gltf_create_target_animation(
                                            asset_curve,
                                            root.clone(),
                                            node_id.clone(),
                                            &key_animegroup,
                                        );
                                    }
                                }
                            }
                            gltf::animation::util::ReadOutputs::MorphTargetWeights(_m) => {
                                todo!()
                            }
                        }
                    }
                }
            }
            channel_index += 1;
        }

        factory.gltf_start_animation_group(root, &key_animegroup);
        animation_index += 1;
    }
}
