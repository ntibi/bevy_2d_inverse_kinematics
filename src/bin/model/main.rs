use bevy::{input::mouse::MouseWheel, prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fabrik::ik::IKPlugin;
use model::RiggedModelPlugin;

mod model;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(IKPlugin)
        .add_plugins(RiggedModelPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (angle, zoom, translate))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // 6 world units per pixel of window height.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn zoom(//camera: Single<&mut OrthographicProjection, With<Camera>>,
    //mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    //let mut projection = camera.into_inner();
    //projection.scale += -mouse_wheel_input.delta.y * 0.1;
}

fn angle(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut evr_scroll: EventReader<MouseWheel>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = camera_query.single_mut();

    for scroll in evr_scroll.read() {
        transform.rotate_y(scroll.x * time.delta_secs() * 0.3);
        transform.rotate_x(scroll.y * time.delta_secs() * 0.3);
    }

    if keyboard_input.pressed(KeyCode::Backspace) {
        transform.rotation = Quat::IDENTITY;
    }
}

fn translate(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = camera_query.single_mut();

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        transform.translation = transform.translation + transform.up() * time.delta_secs() * 5.;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        transform.translation = transform.translation + transform.down() * time.delta_secs() * 5.;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        transform.translation = transform.translation + transform.left() * time.delta_secs() * 5.;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        transform.translation = transform.translation + transform.right() * time.delta_secs() * 5.;
    }
}
