/// # 创建动画数据
///   * FrameCurve
/// ## Linear
///   * curve_frame_values
///   * curve_frame_values_frame
/// ## Step
///   *
/// ## CubicSpline
///   * curve_minmax_curve
///   * curve_minmax_curve_frame
/// # 创建动画组
///   * 动画组里边存放了多个 目标属性动画
///   * 通过操作 动画组 播放/暂停/终止
/// # 创建属性动画
///   * 对哪个属性使用哪个动画数据
/// # 创建目标属性动画
///   * 对哪个目标 使用 哪个属性动画
use crate::interface::GLTFAPI;

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

use pi_scene_context::prelude::*;
use pi_scene_math::{Quaternion, Vector3};

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

impl GLTFAPI<'_, '_> {
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
