use std::sync::mpsc::channel;

use pi_3d::PluginBundleDefault;
use pi_3d_loader::factory::{gltf_decode, GltfLoader};
use pi_async::rt::AsyncRuntime;
use pi_atom::Atom;
use pi_ecs::prelude::{ResMut, Setup};
use pi_ecs_macros::setup;
use pi_engine_shell::{
    engine_shell::{AppShell, EnginShell},
    frame_time::InterfaceFrameTime,
    object::ObjectID,
    plugin::Plugin,
    run_stage::{ERunStageChap, TSystemStageInfo}, assets::local_load::PluginLocalLoad,
};
use pi_hal::runtime::MULTI_MEDIA_RUNTIME;
use pi_render::rhi::options::RenderOptions;
use pi_scene_context::{
    cameras::interface::InterfaceCamera,
    scene::interface::InterfaceScene,
    transforms::{
        command::{ETransformNodeModifyCommand, SingleTransformNodeModifyCommandList},
        interface::InterfaceTransformNode,
    }, skeleton::PluginSkeleton, renderers::graphic::RendererGraphicDesc, pass::{PassTagOrders, EPassTag}, state::PluginStateToFile,
};
use pi_scene_math::{Matrix, Vector3};

#[derive(Debug, Default)]
pub struct SingleTestData {
    pub transforms: Vec<(ObjectID, bool, f32)>,
}

pub struct SysTest;
impl TSystemStageInfo for SysTest {}
#[setup]
impl SysTest {
    #[system]
    pub fn sys(
        mut list: ResMut<SingleTestData>,
        mut transform_commands: ResMut<SingleTransformNodeModifyCommandList>,
    ) {
        // list.transforms.iter_mut().for_each(|mut item| {
        //     item.1 = item.1 + 16.0;
        //     item.2 = item.2 + 16.0;
        //     item.3 = item.3 + 16.0;
        //     let x0 = item.1 % 4000.0 / 4000.0;
        //     let x = x0 * 3.1415926 * 2.;
        //     let y0 = item.2 % 4000.0 / 4000.0;
        //     let y = y0 * 3.1415926 * 2.;
        //     let z0 = item.3 % 4000.0 / 4000.0;
        //     let z = z0 * 3.1415926 * 2.;
        //     // transform_commands.list.push(TransformNodeCommand::ModifyPosition(item.0, Vector3::new(x.cos() * 3., 0., 0.)));
        //     // transform_commands.list.push(TransformNodeCommand::ModifyScaling(item.0, Vector3::new(x.cos() + 0.5, x.sin() + 0.5, x + 0.5)));
        //     transform_commands.list.push(ETransformNodeModifyCommand::ModifyRotation(item.0, Vector3::new(x, y, z)));
        // });
    }
}

pub struct PluginTest;
impl Plugin for PluginTest {
    fn init(
        &mut self,
        engine: &mut pi_scene_context::engine::Engine,
        stages: &mut pi_scene_context::run_stage::RunStage,
    ) -> Result<(), pi_scene_context::plugin::ErrorPlugin> {
        // PluginQuadBuilder.init(engine, stages);
        PluginSkeleton.init(engine, stages);
        PluginLocalLoad.init(engine, stages);

        let mut world = engine.world_mut().clone();

        SysTest::setup(
            &mut world,
            stages.query_stage::<SysTest>(ERunStageChap::Command),
        );

        let testdata = SingleTestData::default();
        world.insert_resource(testdata);
        
        PluginStateToFile.init(engine, stages);

        Ok(())
    }
}

impl PluginTest {
    fn setup(engine: &EnginShell) {
        engine.frame_time(60);
        // Test Code
        let scene01 = engine.create_scene();
        let camera01 = engine.create_free_camera(scene01);
        // let node01 = engine.create_transform_node(scene01);
        // engine.set_parent(camera01, scene01, Some(node01));
        engine.active_camera(camera01, true);
        engine.transform_position(camera01, Vector3::new(0., 0., -5.));
        engine.free_camera_orth_size(camera01, 6 as f32);
        engine.camera_renderer(camera01, RendererGraphicDesc { pre: Some(Atom::from("Clear")), curr: Atom::from("MainCamera"), next: None, passorders: PassTagOrders::new(vec![EPassTag::Opaque]) });

        println!("============1");
        let (sender, receiver) = channel();
        let _ = MULTI_MEDIA_RUNTIME.spawn(MULTI_MEDIA_RUNTIME.alloc(), async move {
            println!("============2");
            println!("curr dir : {:?}", std::env::current_dir());
            let gltf_loader =
                GltfLoader::from_gltf_async("examples/gltf/SimpleAnim.gltf")
                    .await
                    .unwrap();
                println!("============3");
            let buffer = gltf_loader.load_buffer_async().await;
            println!("============4");
            let _ = sender.send((gltf_loader, buffer));
            println!("============5");
        });

        let (gltf_loader, buffer) = receiver.recv().unwrap();
        println!("============6");
        gltf_decode(&gltf_loader, engine, buffer, scene01);
    }
}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let mut shell = AppShell::new(RenderOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    });
    shell.add_plugin(PluginBundleDefault);
    // shell.add_plugin(PluginSkinBuilder);
    // shell.add_plugin(PluginBones);
    shell.add_plugin(PluginTest);
    shell.ready();
    shell.setup(&PluginTest::setup);
    shell.run();
}
