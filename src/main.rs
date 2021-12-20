use std::time::{Duration, Instant};

use bevy::{
    prelude::*,
    render::{RenderApp, RenderStage},
};
use bevy_mod_picking::{
    DebugCursorPickingPlugin, DebugEventsPickingPlugin, DefaultPickingPlugins, PickableBundle,
    PickingCameraBundle,
};

fn main() {
    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        vsync: true,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(DefaultPickingPlugins)
    .add_plugin(DebugCursorPickingPlugin)
    .add_plugin(DebugEventsPickingPlugin)
    .add_startup_system(setup);

    let render_app = app.sub_app(RenderApp);
    render_app
        .insert_resource(FrameTimer::new(60, Duration::from_micros(500)))
        .add_system_to_stage(RenderStage::Render, framerate_exact_limiter)
        .add_system_to_stage(RenderStage::Cleanup, framerate_limit_forward_estimator);

    app.run();
}

#[derive(Debug)]
struct FrameTimer {
    enabled: bool,
    framerate_target: u64,
    frame_start: Instant,
    render_start: Instant,
    exact_sleep: Duration,
    safety_margin: Duration,
}
impl FrameTimer {
    fn new(framerate_limit: u64, margin: Duration) -> Self {
        FrameTimer {
            enabled: true,
            frame_start: Instant::now(),
            render_start: Instant::now(),
            exact_sleep: Duration::from_millis(0),
            framerate_target: framerate_limit,
            safety_margin: margin,
        }
    }
}

/// How long we *think* we should sleep before starting to render the next frame
fn framerate_limit_forward_estimator(mut timer: ResMut<FrameTimer>) {
    let render_end = Instant::now();
    let target_frametime = Duration::from_micros(1_000_000 / timer.framerate_target);
    let last_frametime = render_end.duration_since(timer.frame_start);
    let last_render_time = last_frametime - timer.exact_sleep;
    let estimated_cpu_time_needed = last_render_time + timer.safety_margin;
    let estimated_sleep_time = target_frametime - target_frametime.min(estimated_cpu_time_needed);
    if timer.enabled {
        spin_sleep::sleep(estimated_sleep_time);
    }
    timer.frame_start = Instant::now();
}

fn framerate_exact_limiter(mut timer: ResMut<FrameTimer>) {
    let system_start = Instant::now();
    let target_frametime = Duration::from_micros(1_000_000 / timer.framerate_target);
    let sleep_needed =
        target_frametime - target_frametime.min(system_start.duration_since(timer.render_start));
    if timer.enabled {
        spin_sleep::sleep(sleep_needed);
    }
    timer.render_start = Instant::now();
    timer.exact_sleep = timer.render_start.duration_since(system_start);
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 25.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default());
    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default());
    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
}
