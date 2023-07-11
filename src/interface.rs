use bevy::prelude::Entity;
use default_render::SingleIDBaseDefaultMaterial;

use pi_atom::Atom;

use pi_engine_shell::prelude::*;

use pi_node_materials::NodeMaterialBlocks;
use pi_render::rhi::{BufferAddress, VertexFormat};
use pi_scene_context::prelude::*;
use pi_scene_math::{
    coordiante_system::CoordinateSytem3, vector::TToolMatrix, Matrix, Quaternion, Rotation3,
    Vector3,
};
use unlit_material::shader::UnlitShader;

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
}
