use std::{ops::Range, path::Path};

use bevy::prelude::Entity;
use default_render::SingleIDBaseDefaultMaterial;

use pi_animation::{animation::AnimationInfo, animation_group::AnimationGroupID};
use pi_atom::Atom;
use pi_curves::curve::{
    frame::FrameDataValue,
    frame_curve::{frames::interplate_frame_values_step, FrameCurve},
};

use pi_engine_shell::prelude::*;
use pi_gltf::{
    accessor::Iter,
    animation::{util::ReadOutputs, Channel, Interpolation},
};

use pi_node_materials::NodeMaterialBlocks;
use pi_render::rhi::{BufferAddress, VertexFormat};
use pi_scene_context::prelude::*;
use pi_scene_math::{
    coordiante_system::CoordinateSytem3, vector::TToolMatrix, Matrix, Quaternion, Rotation3,
    Vector3,
};
use unlit_material::shader::UnlitShader;

pub enum FrameCurveType {
    Scaling(FrameCurve<LocalScaling>),
    Rotation(FrameCurve<LocalEulerAngles>),
    Position(FrameCurve<LocalPosition>),
}

pub enum AssetFrameCurveType {
    Scaling(AssetTypeFrameCurve<LocalScaling>),
    Rotation(AssetTypeFrameCurve<LocalEulerAngles>),
    Position(AssetTypeFrameCurve<LocalPosition>),
}

#[derive(SystemParam)]
pub struct GLTFCommands<'w> {
    pub scenecmds: ResMut<'w, ActionListSceneCreate>,
    pub cameracmds: ActionSetCamera<'w>,
    pub fps: ResMut<'w, SingleFrameTimeCommand>,
    pub final_render: ResMut<'w, WindowRenderer>,
    pub renderercmds: ActionSetRenderer<'w>,
    pub transformcmds: ActionSetTransform<'w>,
    pub transformanime: ActionSetTransformNodeAnime<'w>,
    pub meshcmds: ActionSetMesh<'w>,
    pub skincmds: ActionSetSkeleton<'w>,
    pub matcmds: ActionSetMaterial<'w>,
    pub animegroupcmd: ActionSetAnimationGroup<'w>,
    pub asset_mgr: Res<'w, ShareAssetMgr<EVertexBufferRange>>,
    pub data_map: ResMut<'w, VertexBufferDataMap3D>,
    pub geometrycreate: ResMut<'w, ActionListGeometryCreate>,
    pub defaultmat: Res<'w, SingleIDBaseDefaultMaterial>,
    pub nodematblocks: Res<'w, NodeMaterialBlocks>,
}

pub struct GLTFAPI<'a, 'b> {
    pub scene_id: Entity,
    pub commands: &'b mut GLTFCommands<'a>,
}

impl<'a, 'b> GLTFAPI<'a, 'b> {
    // type FrameData;

    pub fn gltf_transform(
        &mut self,
        entity: Entity,
        translation: Option<[f32; 3]>,
        scaling: Option<[f32; 3]>,
        rotation: Option<[f32; 3]>,
        rotation_quaterion: Option<[f32; 4]>,
        matrix: Option<[[f32; 4]; 4]>,
    ) {
        self.commands
            .transformcmds
            .tree
            .push(OpsTransformNodeParent::ops(entity, self.scene_id));

        if let Some(pos) = translation {
            self.commands
                .transformcmds
                .localpos
                .push(OpsTransformNodeLocalPosition::ops(
                    entity, pos[0], pos[1], pos[2],
                ));
        }

        if let Some(scaling) = scaling {
            self.commands
                .transformcmds
                .localscl
                .push(OpsTransformNodeLocalScaling::ops(
                    entity, scaling[0], scaling[1], scaling[2],
                ));
        }

        if let Some(rotation) = rotation {
            self.commands
                .transformcmds
                .localrot
                .push(OpsTransformNodeLocalEuler::ops(
                    entity,
                    rotation[0],
                    rotation[1],
                    rotation[2],
                ));
        }

        if let Some(rotation_quaterion) = rotation_quaterion {
            // TODO: need to check
            let rotation = Quaternion::from_quaternion(nalgebra::Quaternion::new(
                rotation_quaterion[0],
                rotation_quaterion[1],
                rotation_quaterion[2],
                rotation_quaterion[3],
            ))
            .euler_angles();
            self.commands
                .transformcmds
                .localrot
                .push(OpsTransformNodeLocalEuler::ops(
                    entity, rotation.0, rotation.1, rotation.2,
                ));
        }

        if let Some(m) = matrix {
            let matrix = Matrix::new(
                m[0][0], m[0][1], m[0][2], m[0][3], m[1][0], m[1][1], m[1][2], m[1][3], m[2][0],
                m[2][1], m[2][2], m[2][3], m[3][0], m[3][1], m[3][2], m[3][3],
            );
            let mut postion = Vector3::new(0., 0., 0.);
            let mut rotation = Rotation3::identity();
            let mut scaling = Vector3::new(1., 1., 1.);
            CoordinateSytem3::matrix4_decompose_rotation(
                &matrix,
                Some(&mut scaling),
                Some(&mut rotation),
                Some(&mut postion),
            );

            self.commands
                .transformcmds
                .localpos
                .push(OpsTransformNodeLocalPosition::ops(
                    entity, postion[0], postion[1], postion[2],
                ));

            self.commands
                .transformcmds
                .localscl
                .push(OpsTransformNodeLocalScaling::ops(
                    entity, scaling[0], scaling[1], scaling[2],
                ));

            let euler_angles = rotation.euler_angles();
            self.commands
                .transformcmds
                .localrot
                .push(OpsTransformNodeLocalEuler::ops(
                    entity,
                    euler_angles.0,
                    euler_angles.1,
                    euler_angles.2,
                ));
        }
    }

    pub fn gltf_layer_mask(&self, entity: ObjectID, layer: u32) {
        // self.layer_mask(entity, LayerMask(layer));
    }

    pub fn gltf_bounding_info(&self, entity: ObjectID, min: [f32; 3], max: [f32; 3]) {
        // self.add_of_oct_tree(
        //     entity,
        //     BoundingInfo::new(
        //         Vector3::new(min[0], min[1], min[2]),
        //         Vector3::new(max[0], max[1], max[2]),
        //     ),
        // );
    }

    pub fn gltf_create_buffer(&mut self, buffer_id: &str, data: Vec<u8>) {
        if !ActionVertexBuffer::check(&self.commands.asset_mgr, KeyVertexBuffer::from(buffer_id)) {
            ActionVertexBuffer::create(
                &mut self.commands.data_map,
                KeyVertexBuffer::from(buffer_id),
                data,
            );
        }
    }

    pub fn gltf_create_indices_buffer(&mut self, buffer_id: &str, data: Vec<u8>) {
        if !ActionVertexBuffer::check(&self.commands.asset_mgr, KeyVertexBuffer::from(buffer_id)) {
            ActionVertexBuffer::create_indices(
                &mut self.commands.data_map,
                KeyVertexBuffer::from(buffer_id),
                data,
            );
        }
    }

    pub fn gltf_geometry(
        &mut self,
        entity: ObjectID,
        id_geo: Entity,
        descs: Vec<VertexBufferDesc>,
        indices: Option<IndicesBufferDesc>,
    ) {
        // self.use_geometry(entity, descs, indices);
        self.commands
            .geometrycreate
            .push(OpsGeomeryCreate::ops(entity, id_geo, descs, indices));
    }

    pub fn gltf_apply_vertices_buffer(
        &self,
        _entity: ObjectID,
        _kind: EVertexDataKind,
        _buffer_id: Atom,
        _range: std::ops::Range<BufferAddress>,
        _format: VertexFormat,
    ) {
        todo!()
    }

    pub fn gltf_emissive_texture(&mut self, materialid: ObjectID, path: Atom) {
        self.commands.matcmds.texture.push(OpsUniformTexture::ops(
            materialid,
            UniformTextureWithSamplerParam {
                slotname: Atom::from("_MainTex"),
                filter: true,
                sample: KeySampler::default(),
                url: EKeyTexture::from(path.as_str()),
            },
        ));
    }

    pub fn gltf_default_material(&mut self, entity: ObjectID, idmat: Entity) {
        // self.commands.matcmds.create.push(OpsMaterialCreate::ops(
        //     idmat,
        //     DefaultShader::KEY,
        //     EPassTag::Opaque,
        // ));
        self.commands.matcmds.usemat.push(OpsMaterialUse::ops(
            entity,
            self.commands.defaultmat.0.clone(),
        ));
    }

    pub fn gltf_use_material(&mut self, entity: ObjectID, materialid: ObjectID) {
        // self.use_material(entity, materialid);
        self.commands
            .matcmds
            .usemat
            .push(OpsMaterialUse::ops(entity, materialid));
    }

    pub fn gltf_create_unlit_material(&mut self, idmat: Entity) -> ObjectID {
        self.commands.matcmds.create.push(OpsMaterialCreate::ops(
            idmat,
            UnlitShader::KEY,
            EPassTag::Opaque,
        ));
        idmat
    }

    pub fn gltf_create_skin(
        &mut self,
        bone_root: ObjectID,
        bones: Vec<ObjectID>,
        skeleton: Entity,
    ) -> ObjectID {
        // self.create_skeleton_ubo(ESkinBonesPerVertex::Four, bone_root, bones)
        self.commands
            .skincmds
            .skin_create
            .push(OpsSkinCreation::ops(
                skeleton,
                ESkinBonesPerVertex::Four,
                bone_root,
                &bones,
            ));
        skeleton
    }

    pub fn gltf_apply_skin(&mut self, mesh: ObjectID, skin: ObjectID) {
        // self.use_skeleton(mesh, skin);
        self.commands
            .skincmds
            .skin_use
            .push(OpsSkinUse::ops(mesh, skin));
    }

    pub fn gltf_creat_anim_curve(&self, key: &Atom, curve: FrameCurveType) -> AssetFrameCurveType {
        // self.creat_anim_curve(key, curve)
        match curve {
            FrameCurveType::Scaling(curve) => {
                let assets_curve = self
                    .commands
                    .transformanime
                    .scaling
                    .curves
                    .insert(key.clone(), TypeFrameCurve(curve))
                    .unwrap_or_else(|_| panic!("{:?} is already exist", key));
                AssetFrameCurveType::Scaling(AssetTypeFrameCurve::from(assets_curve))
            }
            FrameCurveType::Rotation(curve) => {
                let assets_curve = self
                    .commands
                    .transformanime
                    .euler
                    .curves
                    .insert(key.clone(), TypeFrameCurve(curve))
                    .unwrap_or_else(|_| panic!("{:?} is already exist", key));
                AssetFrameCurveType::Rotation(AssetTypeFrameCurve::from(assets_curve))
            }
            FrameCurveType::Position(curve) => {
                let assets_curve = self
                    .commands
                    .transformanime
                    .position
                    .curves
                    .insert(key.clone(), TypeFrameCurve(curve))
                    .unwrap_or_else(|_| panic!("{:?} is already exist", key));
                AssetFrameCurveType::Position(AssetTypeFrameCurve::from(assets_curve))
            }
        }
    }

    pub fn gltf_check_anim_curve(&self, key: &Atom) -> Option<AssetFrameCurveType> {
        // self.check_anim_curve(key)
        if let Some(curve) = self.commands.transformanime.scaling.curves.get(&key) {
            return Some(AssetFrameCurveType::Scaling(AssetTypeFrameCurve::from(
                curve,
            )));
        }
        if let Some(curve) = self.commands.transformanime.euler.curves.get(&key) {
            return Some(AssetFrameCurveType::Rotation(AssetTypeFrameCurve::from(
                curve,
            )));
        }
        if let Some(curve) = self.commands.transformanime.position.curves.get(&key) {
            return Some(AssetFrameCurveType::Position(AssetTypeFrameCurve::from(
                curve,
            )));
        }
        None
    }

    pub fn gltf_create_animation_group(
        &mut self,
        id_obj: ObjectID,
        key_animegroup: &Atom,
    ) -> AnimationGroupID {
        // let _ = self.create_animation_group(id_obj, key_animegroup);
        let id_group = self
            .commands
            .animegroupcmd
            .scene_ctxs
            .create_group(self.scene_id)
            .unwrap();
        self.commands
            .animegroupcmd
            .global
            .record_group(id_obj, id_group);
        // todo!()

        id_group
    }

    pub fn gltf_create_target_animation(
        &mut self,
        asset_curve: AssetFrameCurveType,
        id_scene: ObjectID,
        id_target: ObjectID,
        key_animegroup: AnimationGroupID,
    ) {
        let animation: AnimationInfo = match asset_curve {
            AssetFrameCurveType::Scaling(curve) => self
                .commands
                .transformanime
                .scaling
                .ctx
                .create_animation(0, curve),
            AssetFrameCurveType::Rotation(curve) => self
                .commands
                .transformanime
                .euler
                .ctx
                .create_animation(0, curve),
            AssetFrameCurveType::Position(curve) => self
                .commands
                .transformanime
                .position
                .ctx
                .create_animation(0, curve),
        };

        self.commands.animegroupcmd.scene_ctxs.add_target_anime(
            id_scene,
            id_target,
            key_animegroup,
            animation,
        );
    }

    pub fn gltf_start_animation_group(&mut self, id_scene: ObjectID, group: AnimationGroupID) {
        self.commands.animegroupcmd.scene_ctxs.start_with_progress(
            id_scene,
            group,
            AnimationGroupParam::default(),
        );
    }

    pub fn gltf_create_assets_curve(
        &mut self,
        key_curve: Atom,
        channel: Channel,
        inputs: Iter<f32>,
        outputs: ReadOutputs,
    ) -> AssetFrameCurveType {
        if let Some(curve) = self.gltf_check_anim_curve(&key_curve) {
            curve
        } else {
            let interpolation = channel.sampler().interpolation();

            match outputs {
                ReadOutputs::Translations(mut t) => {
                    let mut curve = create_vurve(&interpolation);
                    if interpolation == Interpolation::CubicSpline {
                        for input in inputs {
                            let input_tangent = t.next().unwrap();
                            let keyframe = t.next().unwrap();
                            let output_tangent = t.next().unwrap();

                            curve.curve_cubic_splice_frame(
                                (input * 1000.0) as u16,
                                LocalPosition(Vector3::new(keyframe[0], keyframe[1], keyframe[2])),
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
                    } else {
                        for (input, t) in inputs.zip(t) {
                            curve.curve_frame_values_frame(
                                (input * 1000.0) as u16,
                                LocalPosition(Vector3::new(t[0], t[1], t[2])),
                            );
                        }
                    };

                    self.gltf_creat_anim_curve(&key_curve, FrameCurveType::Position(curve))
                }
                ReadOutputs::Rotations(r) => {
                    let mut curve = create_vurve(&interpolation);
                    let mut rotations = r.into_f32();

                    if interpolation == Interpolation::CubicSpline {
                        for input in inputs {
                            let input_tangent = rotations.next().unwrap();
                            let input_tangent =
                                Quaternion::from_quaternion(nalgebra::Quaternion::new(
                                    input_tangent[0],
                                    input_tangent[1],
                                    input_tangent[2],
                                    input_tangent[3],
                                ))
                                .euler_angles();

                            let keyframe = rotations.next().unwrap();
                            let keyframe = Quaternion::from_quaternion(nalgebra::Quaternion::new(
                                keyframe[0],
                                keyframe[1],
                                keyframe[2],
                                keyframe[3],
                            ))
                            .euler_angles();

                            let output_tanget = rotations.next().unwrap();
                            let output_target =
                                Quaternion::from_quaternion(nalgebra::Quaternion::new(
                                    output_tanget[0],
                                    output_tanget[1],
                                    output_tanget[2],
                                    output_tanget[3],
                                ))
                                .euler_angles();

                            curve.curve_cubic_splice_frame(
                                (input * 1000.0) as u16,
                                LocalEulerAngles(Vector3::new(
                                    input_tangent.0,
                                    input_tangent.1,
                                    input_tangent.2,
                                )),
                                LocalEulerAngles(Vector3::new(keyframe.0, keyframe.1, keyframe.2)),
                                LocalEulerAngles(Vector3::new(
                                    output_target.0,
                                    output_target.1,
                                    output_target.2,
                                )),
                            );
                        }
                    } else {
                        for (input, rotation) in inputs.zip(rotations) {
                            let euler_angles =
                                Quaternion::from_quaternion(nalgebra::Quaternion::new(
                                    rotation[0],
                                    rotation[1],
                                    rotation[2],
                                    rotation[3],
                                ))
                                .euler_angles();

                            curve.curve_frame_values_frame(
                                (input * 1000.0) as u16,
                                LocalEulerAngles(Vector3::new(
                                    euler_angles.0,
                                    euler_angles.1,
                                    euler_angles.2,
                                )),
                            );
                        }
                    };

                    self.gltf_creat_anim_curve(&key_curve, FrameCurveType::Rotation(curve))
                }
                ReadOutputs::Scales(mut s) => {
                    let mut curve = create_vurve(&interpolation);

                    if interpolation == Interpolation::CubicSpline {
                        for input in inputs {
                            let input_tangent = s.next().unwrap();
                            let keyframe = s.next().unwrap();
                            let output_tangent = s.next().unwrap();

                            curve.curve_cubic_splice_frame(
                                (input * 1000.0) as u16,
                                LocalScaling(Vector3::new(keyframe[0], keyframe[1], keyframe[2])),
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
                    } else {
                        for (input, scale) in inputs.zip(s) {
                            curve.curve_frame_values_frame(
                                (input * 1000.0) as u16,
                                LocalScaling(Vector3::new(scale[0], scale[1], scale[2])),
                            );
                        }
                    };

                    self.gltf_creat_anim_curve(&key_curve, FrameCurveType::Scaling(curve))
                }
                ReadOutputs::MorphTargetWeights(_) => panic!("MorphTargetWeights is not supported"),
            }
        }
    }
}

fn create_vurve<T: FrameDataValue>(interpolation: &Interpolation) -> FrameCurve<T> {
    if interpolation == &Interpolation::CubicSpline {
        return FrameCurve::curve_cubic_spline(1000);
    } else {
        let mut curve = FrameCurve::curve_frame_values(1000);
        if interpolation == &Interpolation::Step {
            curve.call = interplate_frame_values_step;
        }
        return curve;
    }
}
