use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fabrik::ik::IKPlugin;
use model::RiggedModelPlugin;

mod model;

// IK example with a gltf model
// the IK chain data is specified in the code
// left click to make the effectors follow the cursor
// it only has one leg because i'm lazy

const ZOOM_SPEED: f32 = 0.1;
const CAMERA_SPEED: f32 = 5.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(IKPlugin)
        .add_plugins(RiggedModelPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom, translate))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight { ..default() },
        Transform::from_translation(Vec3::new(10., 10., 10.)),
    ));
}

fn zoom(
    camera: Single<&mut Projection, With<Camera>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    match *camera.into_inner() {
        Projection::Orthographic(ref mut orthographic) => {
            let delta_zoom = -mouse_wheel_input.delta.y * ZOOM_SPEED;
            let multiplicative_zoom = 1. + delta_zoom;

            orthographic.scale = orthographic.scale * multiplicative_zoom;
        }
        Projection::Perspective(ref mut perspective) => {
            let delta_zoom = -mouse_wheel_input.delta.y * ZOOM_SPEED;

            perspective.fov = perspective.fov + delta_zoom;
        }
    }
}

fn translate(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let mut transform = camera_query.single_mut();

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        transform.translation =
            transform.translation + transform.up() * time.delta_secs() * CAMERA_SPEED;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        transform.translation =
            transform.translation + transform.down() * time.delta_secs() * CAMERA_SPEED;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        transform.translation =
            transform.translation + transform.left() * time.delta_secs() * CAMERA_SPEED;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        transform.translation =
            transform.translation + transform.right() * time.delta_secs() * CAMERA_SPEED;
    }
}
