use crate::interface::GLTFAPI;
use std::path::Path;

use bevy::prelude::Entity;
use pi_atom::Atom;
use pi_engine_shell::prelude::*;
use pi_gltf::{image, iter::Textures, json::Value, Material};
use pi_node_materials::prelude::{
    BlockCutoff, BlockEmissiveTexture, BlockEmissiveTextureUVOffsetSpeed, BlockMainTexture,
    BlockMainTextureUVOffsetSpeed, BlockMaskTexture, BlockMaskTextureUVOffsetSpeed, BlockOpacity,
    BlockOpacityFresnel, BlockOpacityTexture, BlockOpacityTextureUVOffsetSpeed,
};

use pi_scene_context::prelude::*;

use unlit_material::{
    effects::{
        distortion_uv::DistortionUVShader, emissive_fresnel::EmissiveFresnelShader,
        main_opacity::MainOpacityShader, main_opacity_fresnel::MainOpacityFresnelShader,
    },
    shader::UnlitShader,
};

const MAIN_OPACITY: &'static str = "main_opacity";
const DISTORTION_UV: &'static str = "distortionUV";
const TWO_OPACITY_MIX: &'static str = "two_opacity_mix";
const MAIN_OPACITY_OPACITY_FRESNEL: &'static str = "main_opacity_opacity_fresnel";

impl GLTFAPI<'_, '_> {
    pub fn gltf_extras_material(
        &mut self,
        entity: Entity,
        idmat: Entity,
        extras: &Value,
        textures: &Vec<pi_gltf::Texture>,
        root_path: &Path,
    ) -> ObjectID {
        if let Some(distortion_uv) = extras.get(DISTORTION_UV) {
            self.distortion_uv(root_path, entity, idmat, distortion_uv, textures);
        } else if let Some(main_opacity) = extras.get(MAIN_OPACITY) {
            self.main_opacity(root_path, entity, idmat, main_opacity, textures);
        } else if let Some(_two_opacity_mix) = extras.get(TWO_OPACITY_MIX) {
            todo!()
        } else if let Some(main_opacity_opacity_fresnel) = extras.get(MAIN_OPACITY_OPACITY_FRESNEL)
        {
            self.main_opacity_opacity_fresnel(
                root_path,
                entity,
                idmat,
                main_opacity_opacity_fresnel,
                textures,
            )
        }

        self.commands
            .matcmds
            .usemat
            .push(OpsMaterialUse::ops(entity, idmat));

        idmat
    }

    fn distortion_uv(
        &mut self,
        root_path: &Path,
        entity: Entity,
        idmat: Entity,
        distortion_uv: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        ActionMaterial::regist_material_meta(
            &self.commands.matcmds.metas,
            &mut self.commands.matcmds.metas_wait,
            KeyShaderMeta::from(DistortionUVShader::KEY),
            DistortionUVShader::create(&self.commands.nodematblocks),
        );

        self.commands.matcmds.create.push(OpsMaterialCreate::ops(
            idmat,
            DistortionUVShader::KEY,
            EPassTag::Transparent,
        ));

        if let Some(diffuse_texture) = distortion_uv.get("diffuseTexture") {
            println!("diffuse_texture: {:?}", diffuse_texture);
            self.diffuse_texture(root_path, idmat, distortion_uv, diffuse_texture, textures)
        }

        if let Some(mask_texture) = distortion_uv.get("maskTexture") {
            println!("mask_texture: {:?}", mask_texture);
            self.mask_texture(root_path, idmat, mask_texture, textures)
        }

        if let Some(diffuse_color) = distortion_uv.get("diffuseColor") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockMainTexture::KEY_COLOR),
                1.,
                1.,
                1.,
                1.,
            ));
        }

        if let Some(alpha) = distortion_uv.get("alpha") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockOpacity::KEY_ALPHA),
                alpha.as_f64().unwrap() as f32,
            ));
        }

        if let Some(alpha_cut_off) = distortion_uv.get("alphaCutOff") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockCutoff::KEY_VALUE),
                alpha_cut_off.as_f64().unwrap() as f32,
            ));
        }

        if let Some(_distortion_x) = distortion_uv.get("distortionX") {
            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("distortionX"),
            //     distortion_x.as_f64().unwrap() as f32,
            // ));
        }

        if let Some(_distortion_y) = distortion_uv.get("distortionY") {
            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("distortionY"),
            //     distortion_y.as_f64().unwrap() as f32,
            // ));
        }

        if let Some(_distortion_sx) = distortion_uv.get("distortionSX") {
            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("distortionSX"),
            //     distortion_sx.as_f64().unwrap() as f32,
            // ));
        }

        if let Some(_distortion_sy) = distortion_uv.get("distortionSY") {
            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("distortionSY"),
            //     distortion_sy.as_f64().unwrap() as f32,
            // ));
        }

        if let Some(mask_flow_mode) = distortion_uv.get("maskFlowMode") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(DistortionUVShader::KEY_MODE),
                mask_flow_mode.as_f64().unwrap() as f32,
            ));
        }

        if let Some(_vertex_color_factor) = distortion_uv.get("vertexColorFactor") {
            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("vertexColorFactor"),
            //     vertex_color_factor.as_f64().unwrap() as f32,
            // ));
        }

        if let Some(_opacity_from_mask_blue) = distortion_uv.get("opacityFromMaskBlue") {
            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("opacityFromMaskBlue"),
            //     opacity_from_mask_blue.as_f64().unwrap() as f32,
            // ));
        }

        let mut cull_mode = CullMode::Off;
        if let Some(cull) = distortion_uv.get("cull") {
            let cull = cull.as_str().unwrap();
            if cull == "off" {
                cull_mode = CullMode::Off
            } else if cull == "front" {
                cull_mode = CullMode::Front
            }
        }

        // TODO:????
        self.commands
            .meshcmds
            .cullmode
            .push(OpsCullMode::ops(entity, cull_mode));

        let _depth_test = true;

        let mut depth_write = false;
        if let Some(z_write) = distortion_uv.get("zWrite") {
            if z_write.as_bool().unwrap() {
                depth_write = true;
            }
        }
        self.commands
            .meshcmds
            .depth_write
            .push(OpsDepthWrite::ops(entity, depth_write));

        let mut alpha_mode = 0;
        if let Some(t_alpha_mode) = distortion_uv.get("alphaMode") {
            println!("=========== distortion_uv alphaMode ===========");
            let mut blend = ModelBlend::default();
            blend.combine();
            self.commands
                .meshcmds
                .blend
                .push(OpsRenderBlend::ops(entity, blend));
        }

        let mut render_queue = 3000;
        if let Some(t_render_queue) = distortion_uv.get("renderQueue") {
            render_queue = t_render_queue.as_i64().unwrap();
        }
        self.commands
            .meshcmds
            .render_queue
            .push(OpsRenderQueue::ops(entity, 0, render_queue as i32));
    }

    fn main_opacity(
        &mut self,
        root_path: &Path,
        entity: Entity,
        idmat: Entity,
        info: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        ActionMaterial::regist_material_meta(
            &self.commands.matcmds.metas,
            &mut self.commands.matcmds.metas_wait,
            KeyShaderMeta::from(MainOpacityShader::KEY),
            MainOpacityShader::meta(),
        );

        self.commands.matcmds.create.push(OpsMaterialCreate::ops(
            idmat,
            MainOpacityShader::KEY,
            EPassTag::Transparent,
        ));

        if let Some(diffuse_texture) = info.get("diffuseTexture") {
            self.diffuse_texture(root_path, idmat, info, diffuse_texture, textures)
        }

        if let Some(emission_texture) = info.get("emissionTexture") {
            self.emissive_texture(root_path, idmat, info, emission_texture, textures)
        }

        if let Some(opacity_texture) = info.get("opacityTexture") {
            self.opacity_texture(root_path, idmat, info, opacity_texture, textures)
        }

        if let Some(diffuse_color) = info.get("diffuseColor") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockMainTexture::KEY_COLOR),
                1., //diffuse_color[0].as_f64().unwrap() as f32,
                1., //diffuse_color[1].as_f64().unwrap() as f32,
                1., //diffuse_color[2].as_f64().unwrap() as f32,
                1., //diffuse_color[3].as_f64().unwrap() as f32,
            ));
        }

        if let Some(emission_color) = info.get("emissionColor") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockEmissiveTexture::KEY_INFO),
                emission_color[0].as_f64().unwrap() as f32,
                emission_color[1].as_f64().unwrap() as f32,
                emission_color[2].as_f64().unwrap() as f32,
                emission_color[3].as_f64().unwrap() as f32,
            ));
        }

        if let Some(alpha) = info.get("alpha") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockOpacity::KEY_ALPHA),
                alpha.as_f64().unwrap() as f32,
            ));
        }

        if let Some(alpha_cut_off) = info.get("alphaCutOff") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockCutoff::KEY_VALUE),
                alpha_cut_off.as_f64().unwrap() as f32,
            ));
        }

        let mut cull_mode = CullMode::Off;
        if let Some(cull) = info.get("cull") {
            let cull = cull.as_str().unwrap();
            if cull == "off" {
                cull_mode = CullMode::Off
            } else if cull == "front" {
                cull_mode = CullMode::Front
            }
        }

        self.commands
            .meshcmds
            .cullmode
            .push(OpsCullMode::ops(entity, cull_mode));

        // TODO:????
        let _depth_test = true;

        let mut depth_write = false;
        if let Some(z_write) = info.get("zWrite") {
            if z_write.as_bool().unwrap() {
                depth_write = true;
            }
        }
        self.commands
            .meshcmds
            .depth_write
            .push(OpsDepthWrite::ops(entity, depth_write));

        let mut alpha_mode = 0;
        if let Some(t_alpha_mode) = info.get("alphaMode") {
            println!("=========== main_opacity alphaMode ===========");
            let mut blend = ModelBlend::default();
            blend.combine();
            self.commands
                .meshcmds
                .blend
                .push(OpsRenderBlend::ops(entity, blend));
        }

        let mut render_queue = 3000;
        if let Some(t_render_queue) = info.get("renderQueue") {
            render_queue = t_render_queue.as_i64().unwrap();
        }
        self.commands
            .meshcmds
            .render_queue
            .push(OpsRenderQueue::ops(entity, 0, render_queue as i32));
    }

    fn main_opacity_opacity_fresnel(
        &mut self,
        root_path: &Path,
        entity: Entity,
        idmat: Entity,
        info: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        ActionMaterial::regist_material_meta(
            &self.commands.matcmds.metas,
            &mut self.commands.matcmds.metas_wait,
            KeyShaderMeta::from(MainOpacityFresnelShader::KEY),
            MainOpacityFresnelShader::create(&self.commands.nodematblocks),
        );

        self.commands.matcmds.create.push(OpsMaterialCreate::ops(
            idmat,
            MainOpacityFresnelShader::KEY,
            EPassTag::Transparent,
        ));

        if let Some(diffuse_texture) = info.get("diffuseTexture") {
            self.diffuse_texture(root_path, idmat, info, diffuse_texture, textures)
        }

        if let Some(emission_texture) = info.get("emissionTexture") {
            self.emissive_texture(root_path, idmat, info, emission_texture, textures)
        }

        if let Some(opacity_texture) = info.get("opacityTexture") {
            self.opacity_texture(root_path, idmat, info, opacity_texture, textures)
        }

        if let Some(diffuse_color) = info.get("diffuseColor") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from("diffuseColor"),
                1.,//diffuse_color[0].as_f64().unwrap() as f32,
                1.,//diffuse_color[1].as_f64().unwrap() as f32,
                1.,//diffuse_color[2].as_f64().unwrap() as f32,
                1.,//diffuse_color[3].as_f64().unwrap() as f32,
            ));
        }

        if let Some(emission_color) = info.get("emissionColor") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockEmissiveTexture::KEY_INFO),
                emission_color[0].as_f64().unwrap() as f32,
                emission_color[1].as_f64().unwrap() as f32,
                emission_color[2].as_f64().unwrap() as f32,
                emission_color[3].as_f64().unwrap() as f32,
            ));
        }

        if let Some(alpha) = info.get("alpha") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockOpacity::KEY_ALPHA),
                alpha.as_f64().unwrap() as f32,
            ));
        }

        if let Some(alpha_cut_off) = info.get("alphaCutOff") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockCutoff::KEY_VALUE),
                alpha_cut_off.as_f64().unwrap() as f32,
            ));
        }

        if let Some((ofbias, ofpower)) = info.get("OFBias").zip(info.get("OFPower")) {
            self.commands.matcmds.vec2.push(OpsUniformVec2::ops(
                idmat,
                Atom::from(BlockOpacityFresnel::KEY_PARAM),
                ofbias.as_f64().unwrap() as f32,
                ofpower.as_f64().unwrap() as f32,
            ));
        }

        if let Some(of_left) = info.get("OFLeft") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockOpacityFresnel::KEY_LEFT),
                of_left[0].as_f64().unwrap() as f32,
                of_left[1].as_f64().unwrap() as f32,
                of_left[2].as_f64().unwrap() as f32,
                of_left[3].as_f64().unwrap() as f32,
            ));
        }

        if let Some(of_right) = info.get("OFRight") {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockOpacityFresnel::KEY_RIGHT),
                of_right[0].as_f64().unwrap() as f32,
                of_right[1].as_f64().unwrap() as f32,
                of_right[2].as_f64().unwrap() as f32,
                of_right[3].as_f64().unwrap() as f32,
            ));
        }

        let mut cull_mode = CullMode::Off;
        if let Some(cull) = info.get("cull") {
            let cull = cull.as_str().unwrap();
            if cull == "off" {
                cull_mode = CullMode::Off
            } else if cull == "front" {
                cull_mode = CullMode::Front
            }
        }

        self.commands
            .meshcmds
            .cullmode
            .push(OpsCullMode::ops(entity, cull_mode));

        // TODO:????
        let _depth_test = true;

        let mut depth_write = false;
        if let Some(z_write) = info.get("zWrite") {
            if z_write.as_bool().unwrap() {
                depth_write = true;
            }
        }
        self.commands
            .meshcmds
            .depth_write
            .push(OpsDepthWrite::ops(entity, depth_write));

        let mut alpha_mode = 0;
        if let Some(t_alpha_mode) = info.get("alphaMode") {
            println!("=========== main_opacity_opacity_fresnel alphaMode ===========");
            let mut blend = ModelBlend::default();
            blend.combine();
            self.commands
                .meshcmds
                .blend
                .push(OpsRenderBlend::ops(entity, blend));
        }

        let mut render_queue = 3000;
        if let Some(t_render_queue) = info.get("renderQueue") {
            render_queue = t_render_queue.as_i64().unwrap();
        }
        self.commands
            .meshcmds
            .render_queue
            .push(OpsRenderQueue::ops(entity, 0, render_queue as i32));
    }

    fn diffuse_texture(
        &mut self,
        root_path: &Path,
        idmat: Entity,
        info: &Value,
        diffuse_texture: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        let index = diffuse_texture["index"].as_u64().unwrap() as usize;
        match textures[index].source().source() {
            image::Source::View {
                view: _,
                mime_type: _,
            } => todo!(),
            image::Source::Uri { uri, mime_type: _ } => {
                let path = root_path.parent().unwrap().join(uri);
                println!("diffuse_texture: {:?}", path);
                self.commands.matcmds.texture.push(OpsUniformTexture::ops(
                    idmat,
                    UniformTextureWithSamplerParam {
                        slotname: Atom::from(BlockMainTexture::KEY_TEX),
                        filter: true,
                        sample: KeySampler::linear_repeat(),
                        url: EKeyTexture::from(path.to_str().unwrap()),
                    },
                ));
                // self.commands.matcmds.texture.push(OpsUniformTexture::ops(idmat, UniformTextureWithSamplerParam {
                //     slotname: Atom::from(BlockOpacityTexture::KEY_TEX),
                //     filter: true,
                //     sample: KeySampler::linear_repeat(),
                //     url: EKeyTexture::from("assets/images/eff_ui_ll_085.png"),
                // }));
            }
        }
        if let Some(diffuse_level) = info.get("diffuseLevel") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockMainTexture::KEY_TILLOFF),
                diffuse_level.as_f64().unwrap() as f32,
            ));
        }

        if let Some((diffuse_ou, diffuse_ov)) = info.get("diffuseOU").zip(info.get("diffuseOV")) {
            self.commands.matcmds.vec2.push(OpsUniformVec2::ops(
                idmat,
                Atom::from(BlockMainTexture::KEY_TILLOFF),
                1000.0 / diffuse_ou.as_f64().unwrap() as f32,
                1000.0 / diffuse_ov.as_f64().unwrap() as f32,
            ));
        }

        if let Some((scale, offset)) = diffuse_texture
            .get("scale")
            .zip(diffuse_texture.get("offset"))
        {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockMainTextureUVOffsetSpeed::KEY_PARAM),
                scale[0].as_f64().unwrap() as f32,
                scale[1].as_f64().unwrap() as f32,
                offset[0].as_f64().unwrap() as f32,
                offset[1].as_f64().unwrap() as f32,
            ));
        }
    }

    fn emissive_texture(
        &mut self,
        root_path: &Path,
        idmat: Entity,
        info: &Value,
        emissive_texture: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        let index = emissive_texture["index"].as_u64().unwrap() as usize;
        match textures[index].source().source() {
            image::Source::View {
                view: _,
                mime_type: _,
            } => todo!(),
            image::Source::Uri { uri, mime_type: _ } => {
                let path = root_path.parent().unwrap().join(uri);
                self.commands.matcmds.texture.push(OpsUniformTexture::ops(
                    idmat,
                    UniformTextureWithSamplerParam {
                        slotname: Atom::from(BlockEmissiveTexture::KEY_TEX),
                        filter: true,
                        sample: KeySampler::default(),
                        url: EKeyTexture::from(path.to_str().unwrap()),
                    },
                ));
            }
        }

        if let Some(_emissive_map_level) = info.get("emissiveMapLevel") {

            // self.commands.matcmds.float.push(OpsUniformFloat::ops(
            //     idmat,
            //     Atom::from("emissiveMapLevel"),
            //     emissive_map_level.as_f64().unwrap() as f32,
            // ));
        }

        if let Some((emission_ou, emission_ov)) = info.get("emissionOU").zip(info.get("emissionOV"))
        {
            self.commands.matcmds.vec2.push(OpsUniformVec2::ops(
                idmat,
                Atom::from(BlockEmissiveTextureUVOffsetSpeed::KEY_PARAM),
                emission_ou.as_f64().unwrap() as f32,
                emission_ov.as_f64().unwrap() as f32,
            ));
        }

        if let Some((scale, offset)) = emissive_texture
            .get("scale")
            .zip(emissive_texture.get("offset"))
        {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockEmissiveTexture::KEY_TILLOFF),
                scale[0].as_f64().unwrap() as f32,
                scale[1].as_f64().unwrap() as f32,
                offset[0].as_f64().unwrap() as f32,
                offset[1].as_f64().unwrap() as f32,
            ));
        }
    }

    fn opacity_texture(
        &mut self,
        root_path: &Path,
        idmat: Entity,
        info: &Value,
        opacity_texture: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        let index = opacity_texture["index"].as_u64().unwrap() as usize;
        match textures[index].source().source() {
            image::Source::View {
                view: _,
                mime_type: _,
            } => todo!(),
            image::Source::Uri { uri, mime_type: _ } => {
                let path = root_path.parent().unwrap().join(uri);
                self.commands.matcmds.texture.push(OpsUniformTexture::ops(
                    idmat,
                    UniformTextureWithSamplerParam {
                        slotname: Atom::from(BlockOpacityTexture::KEY_TEX),
                        filter: true,
                        sample: KeySampler::default(),
                        url: EKeyTexture::from(path.to_str().unwrap()),
                    },
                ));
            }
        }

        if let Some(opacity_level) = info.get("opacityLevel") {
            self.commands.matcmds.float.push(OpsUniformFloat::ops(
                idmat,
                Atom::from(BlockOpacityTexture::KEY_LEVEL),
                opacity_level.as_f64().unwrap() as f32,
            ));
        }

        if let Some((opacity_ou, opacity_ov)) = info.get("opacityOU").zip(info.get("opacityOV")) {
            self.commands.matcmds.vec2.push(OpsUniformVec2::ops(
                idmat,
                Atom::from(BlockOpacityTexture::KEY_TILLOFF),
                opacity_ou.as_f64().unwrap() as f32,
                opacity_ov.as_f64().unwrap() as f32,
            ));
        }

        if let Some((scale, offset)) = opacity_texture
            .get("scale")
            .zip(opacity_texture.get("offset"))
        {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockOpacityTextureUVOffsetSpeed::KEY_PARAM),
                scale[0].as_f64().unwrap() as f32,
                scale[1].as_f64().unwrap() as f32,
                offset[0].as_f64().unwrap() as f32,
                offset[1].as_f64().unwrap() as f32,
            ));
        }
    }

    fn mask_texture(
        &mut self,
        root_path: &Path,
        idmat: Entity,
        mask_texture: &Value,
        textures: &Vec<pi_gltf::Texture>,
    ) {
        let index = mask_texture["index"].as_u64().unwrap() as usize;
        match textures[index].source().source() {
            image::Source::View {
                view: _,
                mime_type: _,
            } => todo!(),
            image::Source::Uri { uri, mime_type: _ } => {
                let path = root_path.parent().unwrap().join(uri);
                println!("mask_texture path: {:?}", path);
                self.commands.matcmds.texture.push(OpsUniformTexture::ops(
                    idmat,
                    UniformTextureWithSamplerParam {
                        slotname: Atom::from(BlockMaskTexture::KEY_TEX),
                        filter: true,
                        sample: KeySampler::default(),
                        url: EKeyTexture::from(path.to_str().unwrap()),
                    },
                ));
            }
        }

        if let Some((scale, offset)) = mask_texture.get("scale").zip(mask_texture.get("offset")) {
            self.commands.matcmds.vec4.push(OpsUniformVec4::ops(
                idmat,
                Atom::from(BlockMaskTextureUVOffsetSpeed::KEY_PARAM),
                scale[0].as_f64().unwrap() as f32,
                scale[1].as_f64().unwrap() as f32,
                offset[0].as_f64().unwrap() as f32,
                offset[1].as_f64().unwrap() as f32,
            ));
        }
    }
}
