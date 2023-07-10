use std::sync::{mpsc::channel, Arc};

use default_render::{shader::DefaultShader, SingleIDBaseDefaultMaterial};
use nalgebra::Quaternion;
use pi_3d::PluginBundleDefault;
use pi_3d_loader::{
    factory::{GltfLoader, ActionListGLTFLoaded, sys_gltf_decode, OpsGLTFLoaded},
};
use pi_animation::animation::AnimationInfo;
// use pi_ecs::prelude::Component;
use pi_async::rt::AsyncRuntime;
use pi_bevy_ecs_extend::system_param::layer_dirty::ComponentEvent;
use pi_bevy_render_plugin::PiRenderPlugin;
use pi_render::rhi::{BufferAddress, VertexFormat};

use pi_curves::curve::{frame::FrameDataValue, frame_curve::FrameCurve};
use pi_engine_shell::{frame_time::PluginFrameTime, prelude::*};
use pi_hal::{init_load_cb, on_load, runtime::MULTI_MEDIA_RUNTIME};
use pi_mesh_builder::{cube::*, quad::PluginQuadBuilder};
use pi_node_materials::{NodeMaterialBlocks, PluginNodeMaterial};
use pi_scene_context::{prelude::*, skeleton::PluginSkeleton};

use pi_3d_loader::interface::AssetFrameCurveType;
use pi_3d_loader::interface::FrameCurveType;
use pi_scene_math::{
    coordiante_system::CoordinateSytem3, vector::TToolMatrix, Matrix, Quaternion as MQuaternion,
    Rotation3, Vector3,
};
use unlit_material::{shader::UnlitShader, PluginUnlitMaterial};

pub struct PluginLocalLoad;
impl Plugin for PluginLocalLoad {
    fn build(&self, app: &mut App) {
        init_load_cb(Arc::new(|path: String| {
            MULTI_MEDIA_RUNTIME
                .spawn(MULTI_MEDIA_RUNTIME.alloc(), async move {
                    log::warn!("Load {}", path);
                    if let Ok(r) = std::fs::read(path.clone()) {
                        on_load(&path, r);
                    } else {
                        log::error!("Load Error: {:?}", path);
                    }
                    // let r = std::fs::read(path.clone()).unwrap();
                })
                .unwrap();
        }));
    }
}

#[derive(Debug, Default)]
pub struct SingleTestData {
    pub transforms: Vec<(ObjectID, bool, f32)>,
}

#[derive(SystemParam)]
struct GLTFCommands<'w, 's> {
    commands: Commands<'w, 's>,
    scenecmds: ResMut<'w, ActionListSceneCreate>,
    // treecmds: ResMut<'w, ActionListTransformNodeParent>,
    cameracmds: ActionSetCamera<'w>,
    fps: ResMut<'w, SingleFrameTimeCommand>,
    final_render: ResMut<'w, WindowRenderer>,
    renderercmds: ActionSetRenderer<'w>,
    transformcmds: ActionSetTransform<'w>,
    transformanime: ActionSetTransformNodeAnime<'w>,
    meshcmds: ActionSetMesh<'w>,
    skincmds: ActionSetSkeleton<'w>,
    matcmds: ActionSetMaterial<'w>,
    animegroupcmd: ActionSetAnimationGroup<'w>,
    // meshcreate: ResMut<'w, ActionListMeshCreate>,
    // localpositioncmds: ResMut<'w, ActionListTransformNodeLocalPosition>,
    // scaling_cmds: ResMut<'w, ActionListTransformNodeLocalScaling>,
    // rotation_cmds: ResMut<'w, ActionListTransformNodeLocalEuler>,
    asset_mgr: Res<'w, ShareAssetMgr<EVertexBufferRange>>,
    data_map: ResMut<'w, VertexBufferDataMap3D>,
    geometrycreate: ResMut<'w, ActionListGeometryCreate>,
}

fn setup(

    mut gltf_commands: GLTFCommands,
    mut gltfloaded: ResMut<ActionListGLTFLoaded>,
    mut defaultmat: Res<SingleIDBaseDefaultMaterial>,
) {
    gltf_commands.final_render.cleardepth = 0.0;
    // engine.frame_time(60);
    gltf_commands.fps.frame_ms = 16;
    // Test Code
    // let scene01 = engine.create_scene();
    let scene = gltf_commands.commands.spawn_empty().id();
    gltf_commands.animegroupcmd.scene_ctxs.init_scene(scene);

    gltf_commands
        .scenecmds
        .push(OpsSceneCreation::ops(scene, ScenePassRenderCfg::default()));

    // let camera01 = engine.create_free_camera(scene01);
    let camera01 = gltf_commands.commands.spawn_empty().id();
    gltf_commands
        .transformcmds
        .tree
        .push(OpsTransformNodeParent::ops(camera01, scene));
    gltf_commands.cameracmds.create.push(OpsCameraCreation::ops(
        scene,
        camera01,
        String::from("TestCamera"),
        true,
    ));

    // engine.active_camera(camera01, true);
    gltf_commands
        .cameracmds
        .active
        .push(OpsCameraActive::ops(camera01, true));

    // engine.transform_position(camera01, Vector3::new(0., 0., -5.));
    gltf_commands
        .transformcmds
        .localpos
        .push(OpsTransformNodeLocalPosition::ops(camera01, 0., 0., -5.));

    // engine.free_camera_orth_size(camera01, 6 as f32);
    gltf_commands
        .cameracmds
        .size
        .push(OpsCameraOrthSize::ops(camera01, 6 as f32));

    // engine.camera_renderer(
    //     camera01,
    //     RendererGraphicDesc {
    //         pre: Some(Atom::from("Clear")),
    //         curr: Atom::from("MainCamera"),
    //         next: None,
    //         passorders: PassTagOrders::new(vec![EPassTag::Opaque]),
    //     },
    // );
    let desc = RendererGraphicDesc {
        pre: Some(gltf_commands.final_render.clear_entity),
        curr: String::from("MainCamera"),
        next: None,
        passorders: PassTagOrders::new(vec![EPassTag::Opaque]),
    };
    let id_renderer = gltf_commands.commands.spawn_empty().id();
    gltf_commands
        .renderercmds
        .create
        .push(OpsRendererCreate::ops(id_renderer, desc.curr.clone()));
    gltf_commands
        .renderercmds
        .connect
        .push(OpsRendererConnect::ops(
            gltf_commands.final_render.clear_entity,
            id_renderer,
        ));
    gltf_commands
        .renderercmds
        .connect
        .push(OpsRendererConnect::ops(
            id_renderer,
            gltf_commands.final_render.render_entity,
        ));
    gltf_commands
        .cameracmds
        .render
        .push(OpsCameraRendererInit::ops(
            camera01,
            id_renderer,
            desc.curr,
            desc.passorders,
            ColorFormat::Rgba8Unorm,
            DepthStencilFormat::None,
        ));

    println!("============1");
    let (sender, receiver) = channel();
    let _ = MULTI_MEDIA_RUNTIME.spawn(MULTI_MEDIA_RUNTIME.alloc(), async move {
        println!("============2");
        let gltf_loader = GltfLoader::from_gltf_async("assets/gltf/eff_ui_leijie.gltf")
            .await
            .unwrap();
        println!("============3");
        let buffer = gltf_loader.load_buffer_async().await;
        println!("============4");
        let _ = sender.send((gltf_loader, buffer));
        println!("============5");
    });
    // let shell = GLTFShell {
    //     scene_id: scene,
    //     commands: gltf_commands,
    // };

    let (gltf_loader, buffer) = receiver.recv().unwrap();
    // println!("============6");
    // gltf_decode(&gltf_loader, shell, buffer);
    gltfloaded.push(OpsGLTFLoaded::ops(scene, gltf_loader, buffer));

    // let cube = gltf_commands.commands.spawn_empty().id(); gltf_commands.transformcmds.tree.push(OpsTransformNodeParent::ops(cube, scene));
    // gltf_commands.meshcmds.create.push(OpsMeshCreation::ops(scene, cube, String::from("TestCube")));
    // let id_geo = gltf_commands.commands.spawn_empty().id();
    // gltf_commands.geometrycreate.push(OpsGeomeryCreate::ops(cube, id_geo, CubeBuilder::attrs_meta(), Some(CubeBuilder::indices_meta())));
    // gltf_commands.matcmds.usemat.push(OpsMaterialUse::ops(cube, defaultmat.0));
}

struct GLTFShell<'w, 's> {
    scene_id: ObjectID,
    pub commands: GLTFCommands<'w, 's>,
}


// pub struct SysTest;
// impl TSystemStageInfo for SysTest {}

// #[setup]
// impl SysTest {
//     #[system]
//     pub fn sys(
//         mut list: ResMut<SingleTestData>,
//         mut transform_commands: ResMut<SingleTransformNodeModifyCommandList>,
//     ) {
//         // list.transforms.iter_mut().for_each(|mut item| {
//         //     item.1 = item.1 + 16.0;
//         //     item.2 = item.2 + 16.0;
//         //     item.3 = item.3 + 16.0;
//         //     let x0 = item.1 % 4000.0 / 4000.0;
//         //     let x = x0 * 3.1415926 * 2.;
//         //     let y0 = item.2 % 4000.0 / 4000.0;
//         //     let y = y0 * 3.1415926 * 2.;
//         //     let z0 = item.3 % 4000.0 / 4000.0;
//         //     let z = z0 * 3.1415926 * 2.;
//         //     // transform_commands.list.push(TransformNodeCommand::ModifyPosition(item.0, Vector3::new(x.cos() * 3., 0., 0.)));
//         //     // transform_commands.list.push(TransformNodeCommand::ModifyScaling(item.0, Vector3::new(x.cos() + 0.5, x.sin() + 0.5, x + 0.5)));
//         //     transform_commands.list.push(ETransformNodeModifyCommand::ModifyRotation(item.0, Vector3::new(x, y, z)));
//         // });
//     }
// }

pub trait AddEvent {
    // 添加事件， 该实现每帧清理一次
    fn add_frame_event<T: Event>(&mut self) -> &mut Self;
}

impl AddEvent for App {
    fn add_frame_event<T: Event>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<Events<T>>() {
            self.init_resource::<Events<T>>()
                .add_system(Events::<T>::update_system);
        }
        self
    }
}

pub type ActionListTestData = ActionList<(ObjectID, f32, f32, f32)>;

pub struct PluginTest;
impl Plugin for PluginTest {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActionListTestData::default());
        app.add_frame_event::<ComponentEvent<Changed<Layer>>>();

        app.insert_resource(ActionListGLTFLoaded::default());
        app.add_system(
            sys_gltf_decode.in_set(ERunStageChap::Command)
        );

        // PluginQuadBuilder.init(engine, stages);
        PluginSkeleton.build(app);
        // PluginLocalLoad.build(app);
        PluginStateToFile.build(app);
    }
}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let mut app = App::default();

    let mut window_plugin = WindowPlugin::default();
    if let Some(primary_window) = &mut window_plugin.primary_window {
        primary_window.resolution.set_physical_resolution(800, 600);
    }

    app.add_plugin(InputPlugin::default());
    app.add_plugin(window_plugin);
    app.add_plugin(AccessibilityPlugin);
    app.add_plugin(bevy::winit::WinitPlugin::default());
    // .add_plugin(WorldInspectorPlugin::new())
    app.add_plugin(pi_bevy_asset::PiAssetPlugin::default());
    app.add_plugin(PiRenderPlugin::default());
    app.add_plugin(PluginLocalLoad);
    app.add_plugin(PluginTest);
    app.add_plugin(PluginFrameTime);
    app.add_plugin(PluginWindowRender);
    app.add_plugins(PluginBundleDefault);
    app.add_plugin(PluginCubeBuilder);
    app.add_plugin(PluginQuadBuilder);
    app.add_plugin(PluginStateToFile);
    app.add_plugin(PluginNodeMaterial);
    app.add_plugin(PluginUnlitMaterial);

    app.world
        .get_resource_mut::<WindowRenderer>()
        .unwrap()
        .active = true;
    // app.add_system(sys_demo_particle.in_set(ERunStageChap::CalcRenderMatrix));

    app.add_startup_system(setup);
    // bevy_mod_debugdump::print_main_schedule(&mut app);

    app.run()
}
