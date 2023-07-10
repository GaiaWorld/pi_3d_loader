use crate::interface::GLTFAPI;
use particle::{
    emitter::ishape_emitter_type::{EBoxShapeMode, EShapeEmitterArcMode},
    extend::format_mesh_particle,
    iparticle_system_config::{
        FourGradientInfo, IParticleSystemConfig, IShape, IShapeArc, IShapeArcBurstSpread,
        IShapeArcLoop, IShapeArcPingPong, IShapeArcRandom, IShapeBox, IShapeCircle, IShapeCone,
        IShapeEdge, IShapeHemisphere, IShapeRectangle, IShapeSphere, ITextureSheet, OneParamInfo,
        ParamInfo, ThreeParamInfo,
    },
    mesh_particle_system::MeshParticleSystem,
    modifier::texture_sheet::{AnimationMode, RowMode, TimeMode},
    particle_system_tool::{
        EMeshParticleScaleMode, EMeshParticleSpaceMode, ERenderAlignment, ERenderMode,
    },
};
use pi_engine_shell::prelude::*;
use pi_gltf::json::Value;

#[derive(Component)]
pub struct Particle(pub MeshParticleSystem);

#[derive(Component)]
pub struct MeshParticleMeshID(pub Entity);

impl GLTFAPI<'_, '_> {
    pub fn gltf_extras_particle(&mut self, extras: &Value) -> MeshParticleSystem {
        let config = gltf_format_particle_cfg(extras);

        let mut mp = MeshParticleSystem::new();
        format_mesh_particle(&config, &mut mp);
        mp.build();
        mp.start();

        mp
    }
}

fn gltf_format_particle_cfg(mesh_particle_cfg: &Value) -> IParticleSystemConfig {
    let mut config = IParticleSystemConfig::default();

    if let Some(name) = mesh_particle_cfg.get("name") {
        config.name = name.as_str().unwrap().to_string()
    }

    if let Some(duration) = mesh_particle_cfg.get("duration") {
        config.duration = duration.as_f64().unwrap() as f32;
    }

    if let Some(start_delay) = mesh_particle_cfg.get("startDelay") {
        config.start_delay = start_delay.as_f64().unwrap() as f32;
    }

    if let Some(looping) = mesh_particle_cfg.get("looping") {
        config.looping = looping.as_u64().unwrap() as u32;
    }

    if let Some(prewarm) = mesh_particle_cfg.get("prewarm") {
        let prewarm = prewarm.as_i64().unwrap();
        if prewarm == 0 {
            config.prewarm = false;
        } else {
            config.prewarm = true;
        }
    }

    if let Some(simulation_space_is_world) = mesh_particle_cfg.get("simulationSpaceIsWorld") {
        let simulation_space_is_world = simulation_space_is_world.as_u64().unwrap();
        if simulation_space_is_world == 0 {
            config.simulation_space_is_world = EMeshParticleSpaceMode::Local;
        } else if simulation_space_is_world == 1 {
            config.simulation_space_is_world = EMeshParticleSpaceMode::World;
        }
    }

    if let Some(scaling_mode) = mesh_particle_cfg.get("scalingMode") {
        let scaling_mode = scaling_mode.as_u64().unwrap();
        if scaling_mode == 0 {
            config.scaling_mode = EMeshParticleScaleMode::Hierarchy;
        } else if scaling_mode == 1 {
            config.scaling_mode = EMeshParticleScaleMode::Local;
        } else if scaling_mode == 2 {
            config.scaling_mode = EMeshParticleScaleMode::Shape;
        }
    }

    if let Some(render_alignment) = mesh_particle_cfg.get("renderAlignment") {
        let render_alignment = render_alignment.as_u64().unwrap();
        if render_alignment == 0 {
            config.render_alignment = ERenderAlignment::View;
        } else if render_alignment == 1 {
            config.render_alignment = ERenderAlignment::World;
        } else if render_alignment == 2 {
            config.render_alignment = ERenderAlignment::Velocity;
        } else if render_alignment == 3 {
            config.render_alignment = ERenderAlignment::Facing;
        } else if render_alignment == 4 {
            config.render_alignment = ERenderAlignment::Local;
        }
    }

    if let Some(render_mode) = mesh_particle_cfg.get("renderMode") {
        let render_mode = render_mode.as_u64().unwrap();
        if render_mode == 0 {
            config.render_mode = ERenderMode::Billboard;
        } else if render_mode == 1 {
            config.render_mode = ERenderMode::StretchedBillboard;
        } else if render_mode == 2 {
            config.render_mode = ERenderMode::HorizontalBillboard;
        } else if render_mode == 3 {
            config.render_mode = ERenderMode::VerticalBillboard;
        } else if render_mode == 4 {
            config.render_mode = ERenderMode::Mesh;
        } else {
            config.render_mode = ERenderMode::None;
        }
    }

    if let Some(stretched_length_scale) = mesh_particle_cfg.get("stretchedLengthScale") {
        config.stretched_length_scale = stretched_length_scale.as_f64().unwrap() as f32;
    }

    if let Some(stretched_velocity_scale) = mesh_particle_cfg.get("stretchedVelocityScale") {
        config.stretched_velocity_scale = stretched_velocity_scale.as_f64().unwrap() as f32;
    }

    if let Some(stretched_velocity_scale) = mesh_particle_cfg.get("renderPivot") {
        config.render_pivot = Some([
            stretched_velocity_scale[0].as_f64().unwrap() as f32,
            stretched_velocity_scale[1].as_f64().unwrap() as f32,
            stretched_velocity_scale[2].as_f64().unwrap() as f32,
        ]);
    }

    if let Some(max_particles) = mesh_particle_cfg.get("maxParticles") {
        config.max_particles = max_particles.as_f64().unwrap() as f32;
    }

    if let Some(start_speed) = mesh_particle_cfg.get("startSpeed") {
        config.start_speed = format_one_param_info(start_speed);
    }

    if let Some(lifetime) = mesh_particle_cfg.get("lifetime") {
        config.lifetime = format_one_param_info(lifetime);
    }

    if let Some(start_color) = mesh_particle_cfg.get("startColor") {
        config.start_color = format_four_gradient_info(start_color);
    }

    if let Some(start_size) = mesh_particle_cfg.get("startSize") {
        config.start_size = format_param_info(start_size);
    }

    if let Some(start_rotation) = mesh_particle_cfg.get("startRotation") {
        config.start_rotation = format_param_info(start_rotation);
    }

    if let Some(gravity) = mesh_particle_cfg.get("gravity") {
        config.gravity = format_one_param_info(gravity)
    }

    if let Some(emission) = mesh_particle_cfg.get("emission") {
        let a = emission[0].as_f64().unwrap() as f32;
        let mut v2 = None;
        if let Some(e2) = emission[1].as_array() {
            let mut temp = vec![];
            for e in e2 {
                println!("{:?}", e);
                temp.push([
                    e[0].as_f64().unwrap() as f32,
                    e[1].as_f64().unwrap() as f32,
                    e[2].as_f64().unwrap() as f32,
                    e[3].as_f64().unwrap() as f32,
                ])
            }
            v2 = Some(temp);
        }
        config.emission = (a, v2);
    }

    if let Some(shape) = mesh_particle_cfg.get("shape") {
        config.shape = format_shape(shape);
    }

    if let Some(velocity_over_lifetime) = mesh_particle_cfg.get("velocityOverLifetime") {
        config.velocity_over_lifetime = Some(format_param_info(velocity_over_lifetime));
    }

    if let Some(velocity_over_lifetime_is_local) =
        mesh_particle_cfg.get("velocityOverLifetimeIsLocal")
    {
        config.velocity_over_lifetime_is_local =
            Some(velocity_over_lifetime_is_local.as_i64().unwrap() as u32);
    }

    if let Some(cfg) = mesh_particle_cfg.get("limitVelocityOverLifetime") {
        config.limit_velocity_over_lifetime = Some(format_one_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("limitVelocityOverLifetime") {
        config.limit_velocity_over_lifetime = Some(format_one_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("limitVelocityOverLifetimeDampen") {
        config.limit_velocity_over_lifetime_dampen = Some(cfg.as_f64().unwrap() as f32);
    }

    if let Some(cfg) = mesh_particle_cfg.get("forceOverLifetime") {
        config.force_over_lifetime = Some(format_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("forceSpaceIsLocal") {
        config.force_space_is_local = Some(cfg.as_i64().unwrap() as u32);
    }

    if let Some(cfg) = mesh_particle_cfg.get("colorOverLifetime") {
        config.color_over_lifetime = Some(format_four_gradient_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("colorBySpeed") {
        config.color_by_speed = Some((
            format_four_gradient_info(&cfg[0]),
            cfg[1].as_f64().unwrap() as f32,
            cfg[2].as_f64().unwrap() as f32,
        ));
    }

    if let Some(cfg) = mesh_particle_cfg.get("sizeOverLifetime") {
        config.size_over_lifetime = Some(format_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("sizeBySpeed") {
        config.size_by_speed = Some((
            format_one_param_info(&cfg[0]),
            cfg[1].as_f64().unwrap() as f32,
            cfg[2].as_f64().unwrap() as f32,
        ));
    }

    if let Some(cfg) = mesh_particle_cfg.get("rotationOverLifetime") {
        config.rotation_over_lifetime = Some(format_param_info(&cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("rotationBySpeed") {
        config.rotation_by_speed = Some((
            format_one_param_info(&cfg[0]),
            cfg[1].as_f64().unwrap() as f32,
            cfg[2].as_f64().unwrap() as f32,
        ));
    }

    if let Some(cfg) = mesh_particle_cfg.get("textureSheet") {
        config.texture_sheet = Some(format_texture_sheet(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("texture") {
        config.texture = Some(cfg.as_str().unwrap().to_string())
    }

    if let Some(_cfg) = mesh_particle_cfg.get("trail") {
        todo!()
    }

    if let Some(cfg) = mesh_particle_cfg.get("orbtialVelocity") {
        config.orbtial_velocity = Some(format_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("orbitalOffset") {
        config.orbital_offset = Some(format_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("orbitalRadial") {
        config.orbital_radial = Some(format_one_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("speedModifier") {
        config.speed_modifier = Some(format_one_param_info(cfg));
    }

    if let Some(cfg) = mesh_particle_cfg.get("renderPivot") {
        config.render_pivot = Some([
            cfg[0].as_f64().unwrap() as f32,
            cfg[1].as_f64().unwrap() as f32,
            cfg[2].as_f64().unwrap() as f32,
        ]);
    }

    if let Some(cfg) = mesh_particle_cfg.get("custom1") {
        config.custom1 = Some([
            format_one_param_info(&cfg[0]),
            format_one_param_info(&cfg[1]),
            format_one_param_info(&cfg[2]),
            format_one_param_info(&cfg[3]),
        ]);
    }

    config
}

fn format_one_param_info(config: &Value) -> OneParamInfo {
    match config[1].as_i64().unwrap() {
        1 => OneParamInfo::TInterpolateConstant(config[2].as_f64().unwrap() as f32),
        2 => OneParamInfo::TInterpolateTwoConstants(
            config[2].as_f64().unwrap() as f32,
            config[3].as_f64().unwrap() as f32,
        ),
        4 => {
            let mut res = vec![];
            for v in config[2][0].as_array().unwrap() {
                let mut t = vec![];
                for v_t in v.as_array().unwrap() {
                    t.push(v_t.as_f64().unwrap() as f32);
                }
                res.push(t);
            }
            let s = config[2][1].as_f64().unwrap() as f32;
            OneParamInfo::TInterpolateCurve((res, s))
        }
        8 => todo!(),
        _ => {
            panic!("config of OneParamInfo: {} is exits!!!!", config)
        }
    }
}

fn format_three_param_info(config: &Value) -> ThreeParamInfo {
    match config[1].as_i64().unwrap() {
        1 => ThreeParamInfo::TInterpolateConstant([
            config[2][0].as_f64().unwrap() as f32,
            config[2][1].as_f64().unwrap() as f32,
            config[2][2].as_f64().unwrap() as f32,
        ]),
        2 => ThreeParamInfo::TInterpolateTwoConstants(
            [
                config[2][0].as_f64().unwrap() as f32,
                config[2][1].as_f64().unwrap() as f32,
                config[2][2].as_f64().unwrap() as f32,
            ],
            [
                config[3][0].as_f64().unwrap() as f32,
                config[3][1].as_f64().unwrap() as f32,
                config[3][2].as_f64().unwrap() as f32,
            ],
        ),
        4 | 8 => todo!(),
        _ => {
            panic!("config of ThreeParamInfo: {} is exits!!!!", config)
        }
    }
}

fn format_param_info(config: &Value) -> ParamInfo {
    match config[0].as_i64().unwrap() {
        1 => ParamInfo::OneParamInfo(format_one_param_info(config)),
        3 => ParamInfo::ThreeParamInfo(format_three_param_info(config)),
        _ => {
            panic!("config of ParamInfo: {} is exits!!!!", config)
        }
    }
}

fn format_four_gradient_info(config: &Value) -> FourGradientInfo {
    match config[1].as_i64().unwrap() {
        1 => FourGradientInfo::TInterpolateColor([
            config[2][0].as_f64().unwrap() as f32,
            config[2][1].as_f64().unwrap() as f32,
            config[2][2].as_f64().unwrap() as f32,
            config[2][3].as_f64().unwrap() as f32,
        ]),
        2 => FourGradientInfo::TInterpolateTwoColors(
            [
                config[2][0].as_f64().unwrap() as f32,
                config[2][1].as_f64().unwrap() as f32,
                config[2][2].as_f64().unwrap() as f32,
                config[2][3].as_f64().unwrap() as f32,
            ],
            [
                config[3][0].as_f64().unwrap() as f32,
                config[3][1].as_f64().unwrap() as f32,
                config[3][2].as_f64().unwrap() as f32,
                config[3][3].as_f64().unwrap() as f32,
            ],
        ),
        4 => {
            let mut vec1 = vec![];
            for v in config[2][0].as_array().unwrap() {
                vec1.push([v[0].as_f64().unwrap() as f32, v[1].as_f64().unwrap() as f32]);
            }
            let mut vec2 = vec![];
            for v in config[2][1].as_array().unwrap() {
                vec2.push([v[0].as_f64().unwrap() as f32, v[1].as_f64().unwrap() as f32]);
            }
            let mut vec3 = vec![];
            for v in config[2][2].as_array().unwrap() {
                vec3.push([v[0].as_f64().unwrap() as f32, v[1].as_f64().unwrap() as f32]);
            }
            let mut vec4 = vec![];
            for v in config[2][3].as_array().unwrap() {
                vec4.push([v[0].as_f64().unwrap() as f32, v[1].as_f64().unwrap() as f32]);
            }
            FourGradientInfo::TInterpolateGradient([vec1, vec2, vec3, vec4])
        }
        8 => todo!(),
        16 => FourGradientInfo::TInterpolateRandom,
        _ => {
            panic!("config of FourGradientInfo: {} is exits!!!!", config)
        }
    }
}

fn format_shape(config: &Value) -> IShape {
    let mut radius = 0.0;
    if let Some(v) = config.get("radius") {
        radius = v.as_f64().unwrap() as f32;
    };

    let mut height = 0.0;
    if let Some(v) = config.get("height") {
        height = v.as_f64().unwrap() as f32;
    };

    let mut radius_thickness = 0.0;
    if let Some(v) = config.get("radiusThickness") {
        radius_thickness = v.as_f64().unwrap() as f32;
    };

    let mut arc = IShapeArc::default();
    if let Some(v) = config.get("arc") {
        let value = v["value"].as_f64().unwrap() as f32;
        let spread = v["spread"].as_f64().unwrap() as f32;
        let speed = v["speed"].as_f64().unwrap() as f32;

        arc = match v["mode"].as_i64().unwrap() {
            1 => IShapeArc::IShapeArcRandom(IShapeArcRandom {
                mode: EShapeEmitterArcMode::Random,
                value,
                spread,
                speed,
            }),
            2 => IShapeArc::IShapeArcLoop(IShapeArcLoop {
                mode: EShapeEmitterArcMode::Loop,
                value: v["mode"].as_f64().unwrap() as f32,
                spread: v["spread"].as_f64().unwrap() as f32,
                speed: v["speed"].as_f64().unwrap() as f32,
            }),
            3 => IShapeArc::IShapeArcPingPong(IShapeArcPingPong {
                mode: EShapeEmitterArcMode::PingPong,
                value: v["mode"].as_f64().unwrap() as f32,
                spread: v["spread"].as_f64().unwrap() as f32,
                speed: v["speed"].as_f64().unwrap() as f32,
            }),
            4 => IShapeArc::IShapeArcBurstSpread(IShapeArcBurstSpread {
                mode: EShapeEmitterArcMode::BurstsSpread,
                value: v["mode"].as_f64().unwrap() as f32,
                spread: v["spread"].as_f64().unwrap() as f32,
                speed: v["speed"].as_f64().unwrap() as f32,
            }),
            _ => panic!("arc mode is not exits"),
        }
    };

    let mut scale = None;
    if let Some(v) = config.get("scale") {
        scale = Some([
            v[0].as_f64().unwrap() as f32,
            v[1].as_f64().unwrap() as f32,
            v[2].as_f64().unwrap() as f32,
        ]);
    };

    let mut position = None;
    if let Some(v) = config.get("position") {
        position = Some([
            v[0].as_f64().unwrap() as f32,
            v[1].as_f64().unwrap() as f32,
            v[2].as_f64().unwrap() as f32,
        ]);
    };

    let mut rotation = None;
    if let Some(v) = config.get("rotation") {
        rotation = Some([
            v[0].as_f64().unwrap() as f32,
            v[1].as_f64().unwrap() as f32,
            v[2].as_f64().unwrap() as f32,
        ]);
    };

    let mut align_dir = 0;
    if let Some(v) = config.get("alignDir") {
        align_dir = v.as_i64().unwrap() as u32;
    };

    let mut angle = 0.0;
    if let Some(v) = config.get("angle") {
        angle = v.as_f64().unwrap() as f32;
    };

    let mut randomize = None;
    if let Some(v) = config.get("randomize") {
        randomize = Some([
            v[0].as_f64().unwrap() as f32,
            v[1].as_f64().unwrap() as f32,
            v[2].as_f64().unwrap() as f32,
        ]);
    };

    let mut emit_as_volume = true;
    if let Some(vemit_as_volume) = config.get("emit_as_volume") {
        if vemit_as_volume.as_i64().unwrap() == 0 {
            emit_as_volume = false;
        }
    };

    let mut is_volume = 0;
    if let Some(v) = config.get("is_volume") {
        is_volume = v.as_i64().unwrap() as u32;
    };

    let mut box_emit_mode = None;
    if let Some(v) = config.get("box_emit_mode") {
        match v.as_i64().unwrap() {
            0 => box_emit_mode = Some(EBoxShapeMode::Volume),
            1 => box_emit_mode = Some(EBoxShapeMode::Shell),
            2 => box_emit_mode = Some(EBoxShapeMode::Edge),
            _ => panic!("box_emit_mode is not exits"),
        }
    };

    match config["type"].as_i64().unwrap() {
        0 => IShape::ShapeCone(IShapeCone {
            _type: 0,
            radius,
            angle,
            radius_thickness,
            arc,
            emit_as_volume,
            height,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
        }),
        1 => IShape::ShapeSphere(IShapeSphere {
            _type: 1,
            radius,
            radius_thickness,
            arc,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
        }),
        2 => IShape::ShapeBox(IShapeBox {
            _type: 2,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
            is_volume,
            box_emit_mode,
        }),
        3 => IShape::ShapeCircle(IShapeCircle {
            _type: 3,
            radius,
            radius_thickness,
            arc,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
        }),
        4 => IShape::ShapeHemisphere(IShapeHemisphere {
            _type: 4,
            radius,
            radius_thickness,
            arc,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
        }),
        5 => IShape::ShapeEdge(IShapeEdge {
            _type: 5,
            radius,
            arc,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
        }),
        6 => IShape::ShapeRectangle(IShapeRectangle {
            _type: 6,
            scale,
            position,
            rotation,
            align_dir,
            randomize,
        }),

        _ => {
            panic!("config of FourGradientInfo: {} is exits!!!!", config)
        }
    }
}

fn format_texture_sheet(config: &Value) -> ITextureSheet {
    ITextureSheet {
        frame_over_time: format_one_param_info(&config["frameOverTime"]),
        anim_mode: match config["animMode"].as_i64().unwrap() {
            0 => AnimationMode::WholeSheet,
            1 => AnimationMode::SingleRow,
            _ => panic!("animMode is not exits"),
        },
        custom_row: config["customRow"].as_i64().unwrap() as f32,
        cycles: config["cycles"].as_i64().unwrap() as f32,
        row_mode: match config["rowMode"].as_i64().unwrap() {
            0 => RowMode::Custom,
            1 => RowMode::Random,
            _ => panic!("rowMode is not exits"),
        },
        start_frame: format_one_param_info(&config["startFrame"]),
        tiles_x: config["tilesX"].as_i64().unwrap() as f32,
        tiles_y: config["tilesY"].as_i64().unwrap() as f32,
        time_mode: match config["timeMode"].as_i64().unwrap() {
            0 => TimeMode::Liftime,
            1 => TimeMode::Speed,
            _ => panic!("timeMode is not exits"),
        },
    }
}
