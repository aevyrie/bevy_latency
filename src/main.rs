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
        .insert_resource(FrameTimer {
            enabled: true,
            start: std::time::Instant::now(),
            framerate_target: 60,
            frametime_buffer: std::time::Duration::from_millis(3),
        })
        .add_system_to_stage(RenderStage::Cleanup, framerate_limiter);
    //.add_system_to_stage(RenderStage::Cleanup, limiter_state);
    app.run();
}

#[derive(Debug)]
struct FrameTimer {
    enabled: bool,
    start: std::time::Instant,
    framerate_target: u64,
    frametime_buffer: std::time::Duration,
}

fn framerate_limiter(mut timer: ResMut<FrameTimer>) {
    let sys_start = std::time::Instant::now();
    let frame_time = sys_start.duration_since(timer.start);
    let target_frametime = std::time::Duration::from_micros(1_000_000 / timer.framerate_target);
    let padded_frame_time = frame_time + timer.frametime_buffer;
    let sleep_time = target_frametime - padded_frame_time.min(target_frametime);
    // info!(
    // "Frame Time: {:>6.2}ms, Target: {:>6.2}ms, Sleeping: {:>6.2}ms",
    // (frame_time.as_micros() as f32) / 1000.0,
    // (target_frametime.as_micros() as f32) / 1000.0,
    // (sleep_time.as_micros() as f32) / 1000.0
    // );
    if timer.enabled {
        while std::time::Instant::now().duration_since(sys_start) < sleep_time {
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
    }
    timer.start = std::time::Instant::now();
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
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
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
