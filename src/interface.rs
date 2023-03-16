use std::ops::Range;

use default_render::interface::InterfaceDefaultMaterial;
use pi_animation::{amount::AnimationAmountCalc, loop_mode::ELoopMode};

use pi_atom::Atom;
use pi_curves::curve::{frame::FrameDataValue, frame_curve::FrameCurve};
use pi_ecs::prelude::Component;
use pi_engine_shell::{
    engine_shell::EnginShell,
    object::{InterfaceObject, ObjectID},
};
use pi_render::{
    render_3d::shader::{
        skin_code::ESkinBonesPerVertex, uniform_texture::UniformTextureWithSamplerParam,
    },
    renderer::{
        attributes::EVertexDataKind, indices::IndicesBufferDesc, sampler::KeySampler,
        vertex_buffer::KeyVertexBuffer, vertex_buffer_desc::VertexBufferDesc,
    },
};
use pi_scene_context::{
    animation::{
        base::AssetTypeFrameCurve,
        interface::{InterfaceAnimationGroup, InterfaceAnimeAsset},
    },
    cullings::bounding::BoundingInfo,
    geometry::TInterfaceGeomtery,
    layer_mask::{interface::InterfaceLayerMask, LayerMask},
    meshes::interface::InterfaceMesh,
    pass::EPassTag,
    scene::interface::InterfaceScene,
    skeleton::interface::TInterfaceSkeleton,
    transforms::interface::InterfaceTransformNode,
};
use pi_scene_context::{
    cullings::oct_tree::InterfaceOctTree, materials::interface::InterfaceMaterial,
};
use pi_scene_math::{
    coordiante_system::CoordinateSytem3, vector::TToolMatrix, Matrix, Quaternion, Rotation3,
    Vector3,
};

use unlit_material::interface::InterfaceUnlitMaterial;
pub trait InterfaceGLTFLoader<T> {
    /// 创建 节点 - Node
    /// * `scaling` - [f32, f32, f32] - scale
    /// * `rotation` - [f32, f32, f32] - rotation (Euler Angle)
    /// * `rotation_quaterion` - [f32, f32, f32, f32] - rotation (Quaterion)
    /// * `matrix` - [f32; 16] - matrix
    fn gltf_create_node(
        &self,
        translation: Option<[f32; 3]>,
        scaling: Option<[f32; 3]>,
        rotation: Option<[f32; 3]>,
        rotation_quaterion: Option<[f32; 4]>,
        matrix: Option<[[f32; 4]; 4]>,
        scene_id: T,
    ) -> T;

    /// 赋予节点 层级信息 - node.
    /// TODO
    fn gltf_layer_mask(&self, entity: T, layer: u32);

    /// 赋予节点 包围盒信息 -
    /// * `center` `extend` - boundingbox
    fn gltf_bounding_info(&self, entity: T, min: [f32; 3], max: [f32; 3]);

    /// 创建Buffer
    fn gltf_create_buffer(&self, buffer_id: &str, data: Vec<u8>);

    // fn gltf_create_indices_buffer(&self, buffer_id: &str, data: Vec<u8>);

    /// 设置目标的网格描述
    fn gltf_geometry(
        &self,
        entity: T,
        desc: Vec<VertexBufferDesc>,
        indices: Option<IndicesBufferDesc>,
    );

    /// 使用 Verteices Buffer
    fn gltf_apply_vertices_buffer(
        &self,
        entity: T,
        kind: EVertexDataKind,
        buffer_id: Atom,
        range: Range<wgpu::BufferAddress>,
        format: wgpu::VertexFormat,
    );

    // /// 使用 Indeices Buffer
    // fn gltf_apply_indices_buffer(
    //     &self,
    //     entity: T,
    //     indices_id: Atom,
    //     range: Range<wgpu::BufferAddress>,
    //     format: wgpu::IndexFormat,
    // );

    /// 创建 纹理
    fn gltf_emissive_texture(&self, materialid: T, path: Atom);

    /// 创建 基础材质
    /// 1. 测试
    /// 2. 没有实际支持的时候直接使用为DefaultMaterial
    fn gltf_default_material(&self, entity: T);

    /// 绑定材质
    fn gltf_use_material(&self, entity: T, materialid: T);

    fn gltf_create_unlit_material(&self) -> T;

    // fn gltf_create_skin(&self, scene_id: T, bone_root: T, bones: Vec<[[f32; 4]; 4]>) -> Vec<T>;

    fn gltf_create_skin(&self, bone_root: T, bones: Vec<T>) -> T;
    fn gltf_apply_skin(&self, mesh: T, skin: T);

    fn gltf_creat_anim_curve<D: FrameDataValue + Component>(
        &self,
        key: &Atom,
        curve: FrameCurve<D>,
    ) -> AssetTypeFrameCurve<D>;

    fn gltf_check_anim_curve<D: FrameDataValue + Component>(
        &self,
        key: &Atom,
    ) -> Option<AssetTypeFrameCurve<D>>;

    fn gltf_create_animation_group(&self, id_obj: T, key_animegroup: &Atom);

    fn gltf_create_target_animation<D: FrameDataValue + Component>(
        &self,
        asset_curve: AssetTypeFrameCurve<D>,
        id_obj: T,
        id_target: T,
        key_animegroup: &Atom,
    );

    fn gltf_start_animation_group(&self, id_obj: T, key_animegroup: &Atom);
}

impl InterfaceGLTFLoader<ObjectID> for EnginShell {
    // type FrameData;

    fn gltf_create_node(
        &self,
        translation: Option<[f32; 3]>,
        scaling: Option<[f32; 3]>,
        rotation: Option<[f32; 3]>,
        rotation_quaterion: Option<[f32; 4]>,
        matrix: Option<[[f32; 4]; 4]>,
        scene_id: ObjectID,
    ) -> ObjectID {
        let entity = self.new_object();
        self.add_to_scene(entity, scene_id)
            .as_transform_node(entity)
            .transform_parent(entity, scene_id)
            .as_mesh(entity);

        if let Some(pos) = translation {
            self.transform_position(entity, Vector3::new(pos[0], pos[1], pos[2]));
        }

        if let Some(scaling) = scaling {
            self.transform_scaling(entity, Vector3::new(scaling[0], scaling[1], scaling[2]));
        }

        if let Some(rotation) = rotation {
            self.transform_rotation_euler(
                entity,
                Vector3::new(rotation[0], rotation[1], rotation[2]),
            );
        }

        if let Some(rotation) = rotation_quaterion {
            self.transform_rotation_quaternion(
                entity,
                Quaternion::new_eps(
                    Vector3::new(rotation[0], rotation[1], rotation[2]),
                    rotation[3],
                ),
            );
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

            self.transform_position(entity, postion);

            self.transform_scaling(entity, Vector3::new(scaling[0], scaling[1], scaling[2]));

            let euler_angles = rotation.euler_angles();
            self.transform_rotation_euler(
                entity,
                Vector3::new(euler_angles.0, euler_angles.1, euler_angles.2),
            );
        }

        self.layer_mask(entity, LayerMask::default());

        entity
    }

    fn gltf_layer_mask(&self, entity: ObjectID, layer: u32) {
        self.layer_mask(entity, LayerMask(layer));
    }

    fn gltf_bounding_info(&self, entity: ObjectID, min: [f32; 3], max: [f32; 3]) {
        self.add_of_oct_tree(
            entity,
            BoundingInfo::new(
                Vector3::new(min[0], min[1], min[2]),
                Vector3::new(max[0], max[1], max[2]),
            ),
        );
    }

    fn gltf_create_buffer(&self, buffer_id: &str, data: Vec<u8>) {
        self.create_vertex_buffer(KeyVertexBuffer::from(buffer_id), data);
    }

    fn gltf_geometry(
        &self,
        entity: ObjectID,
        descs: Vec<VertexBufferDesc>,
        indices: Option<IndicesBufferDesc>,
    ) {
        self.use_geometry(entity, descs, indices);
    }

    fn gltf_apply_vertices_buffer(
        &self,
        _entity: ObjectID,
        _kind: EVertexDataKind,
        _buffer_id: Atom,
        _range: std::ops::Range<wgpu::BufferAddress>,
        _format: wgpu::VertexFormat,
    ) {
        todo!()
    }

    fn gltf_emissive_texture(&self, materialid: ObjectID, path: Atom) {
        self.emissive_texture(
            materialid,
            UniformTextureWithSamplerParam {
                slotname: Atom::from("_MainTex"),
                filter: true,
                sample: KeySampler::default(),
                url: path,
            },
        );
    }

    fn gltf_default_material(&self, entity: ObjectID) {
        self.use_default_material(entity);
    }

    fn gltf_use_material(&self, entity: ObjectID, materialid: ObjectID) {
        self.use_material(entity, materialid);
    }

    fn gltf_create_unlit_material(&self) -> ObjectID {
        self.create_default_material(EPassTag::Opaque)
    }

    fn gltf_create_skin(&self, bone_root: ObjectID, bones: Vec<ObjectID>) -> ObjectID {
        self.create_skeleton_ubo(ESkinBonesPerVertex::Four, bone_root, bones)
    }

    fn gltf_apply_skin(&self, mesh: ObjectID, skin: ObjectID) {
        self.use_skeleton(mesh, skin);
    }

    fn gltf_creat_anim_curve<D: FrameDataValue + Component>(
        &self,
        key: &Atom,
        curve: FrameCurve<D>,
    ) -> AssetTypeFrameCurve<D> {
        self.creat_anim_curve(key, curve)
    }

    fn gltf_check_anim_curve<D: FrameDataValue + Component>(
        &self,
        key: &Atom,
    ) -> Option<AssetTypeFrameCurve<D>> {
        self.check_anim_curve(key)
    }

    fn gltf_create_animation_group(&self, id_obj: ObjectID, key_animegroup: &Atom) {
        let _ = self.create_animation_group(id_obj, key_animegroup);
    }

    fn gltf_create_target_animation<D: FrameDataValue + Component>(
        &self,
        asset_curve: AssetTypeFrameCurve<D>,
        id_obj: ObjectID,
        id_target: ObjectID,
        key_animegroup: &Atom,
    ) {
        let animation = self.create_animation(asset_curve);
        let _ = self.create_target_animation(id_obj, id_target, key_animegroup, animation);
    }

    fn gltf_start_animation_group(&self, id_obj: ObjectID, key_animegroup: &Atom) {
        let _ = self.start_animation_group(
            id_obj,
            key_animegroup,
            1.0,
            ELoopMode::Positive(None),
            0.,
            1000.,
            60,
            AnimationAmountCalc::default(),
        );
    }
}
