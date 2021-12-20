use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
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
        vsync: true, // Disabled for this demo to remove vsync as a source of input latency
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(DefaultPickingPlugins)
    .add_plugin(DebugCursorPickingPlugin)
    .add_plugin(DebugEventsPickingPlugin)
    .add_startup_system(setup);
    //.add_startup_system(setup_ui);

    let render_app = app.sub_app(RenderApp);
    render_app
        .insert_resource(FrameTimer::default())
        .add_system_to_stage(RenderStage::Render, framerate_exact_limiter)
        .add_system_to_stage(RenderStage::Cleanup, framerate_limit_forward_estimator);
    //.add_system_to_stage(RenderStage::Cleanup, limiter_state);
    app.run();
}

#[derive(SystemLabel, Clone, Hash, Debug, Eq, PartialEq)]
enum FrameLimit {
    Estimate,
    Exact,
}

#[derive(Debug)]
struct FrameTimer {
    enabled: bool,
    frame_start: std::time::Instant,
    exact_start: std::time::Instant,
    render_start: std::time::Instant,
    exact_sleep: std::time::Duration,
    framerate_target: u64,
    safety_margin: std::time::Duration,
}
impl Default for FrameTimer {
    fn default() -> Self {
        FrameTimer {
            enabled: true,
            frame_start: std::time::Instant::now(),
            render_start: std::time::Instant::now(),
            exact_start: std::time::Instant::now(),
            exact_sleep: std::time::Duration::from_millis(0),
            framerate_target: 60,
            safety_margin: std::time::Duration::from_micros(500),
        }
    }
}

/// How long we *think* we should sleep before starting to render the next frame
fn framerate_limit_forward_estimator(mut timer: ResMut<FrameTimer>) {
    let render_end = std::time::Instant::now();
    let target_frametime = std::time::Duration::from_micros(1_000_000 / timer.framerate_target);
    let last_frametime = render_end.duration_since(timer.frame_start);
    let last_render_time = last_frametime - timer.exact_sleep;
    let estimated_cpu_time_needed = last_render_time + timer.safety_margin;
    let estimated_sleep_time = target_frametime - target_frametime.min(estimated_cpu_time_needed);
    if timer.enabled {
        spin_sleep::sleep(estimated_sleep_time);
    }
    timer.frame_start = std::time::Instant::now();
    /*
    info!(
        "FT: {:>5.2}RT: {:>5.2}ms, EST: {:>5.2}ms, EXT: {:>5.2}ms",
        last_frametime.as_micros() as f32 / 1000.,
        last_render_time.as_micros() as f32 / 1000.,
        estimated_sleep_time.as_micros() as f32 / 1000.,
        timer.exact_sleep.as_micros() as f32 / 1000.,
    );
    */
}

fn framerate_exact_limiter(mut timer: ResMut<FrameTimer>) {
    let system_start = std::time::Instant::now();
    let target_frametime = std::time::Duration::from_micros(1_000_000 / timer.framerate_target);
    let sleep_needed =
        target_frametime - target_frametime.min(system_start.duration_since(timer.exact_start));
    if timer.enabled {
        spin_sleep::sleep(sleep_needed);
    }
    timer.exact_start = std::time::Instant::now();
    timer.exact_sleep = timer.exact_start.duration_since(system_start);
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

/*fn limiter_state(
    mut timer: ResMut<FrameTimer>,
    mut status_query: Query<&mut Text, With<StatusText>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mut text = if let Ok(text) = status_query.get_single_mut() {
        text
    } else {
        return;
    };
    if keyboard.just_pressed(KeyCode::Space) {
        if timer.enabled {
            timer.enabled = false;
            text.sections[0].value = "OFF".to_string();
            text.sections[0].style.color = Color::RED;
        } else {
            timer.enabled = true;
            text.sections[0].value = "ON".to_string();
            text.sections[0].style.color = Color::GREEN;
        }
    }
}*/

#[derive(Component)]
struct StatusText;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|ui| {
            ui.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Press spacebar to toggle".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            });
            ui.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "OFF".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 30.0,
                            color: Color::RED,
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(StatusText);
        });
}
