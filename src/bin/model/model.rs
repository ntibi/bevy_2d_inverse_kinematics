use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};
use fabrik::ik::IKConstraint;

pub struct RiggedModelPlugin;

const SPEED: f32 = 2.;
const ROTATION_SPEED: f32 = PI;

impl Plugin for RiggedModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (input, movement).chain())
            .add_systems(Update, (update_target).chain());
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Visibility::Visible,
        Movable,
        Velocity::default(),
        AngularVelocity::default(),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("frog.gltf"))),
    ));
}

fn update_target(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut IKConstraint>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera.single();
    let window = window.single();

    if let Some(pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        for mut ik in query.iter_mut() {
            ik.target(pos);
        }
    }
}

#[derive(Component)]
struct Movable;

#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct AngularVelocity(f32);

fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut AngularVelocity), With<Movable>>,
) {
    for (mut vel, mut angvel) in query.iter_mut() {
        let mut dir = Vec2::ZERO;
        let mut rotation = 0.;

        if keyboard_input.pressed(KeyCode::KeyW) {
            dir += Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            dir -= Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            dir -= Vec2::X;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            dir += Vec2::X;
        }

        if keyboard_input.pressed(KeyCode::KeyQ) {
            rotation = 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            rotation = -1.;
        }

        **vel = dir.normalize_or_zero();
        **angvel = rotation;
    }
}

fn movement(mut query: Query<(&mut Transform, &Velocity, &AngularVelocity)>, time: Res<Time>) {
    for (mut transform, vel, angvel) in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(**angvel * ROTATION_SPEED * time.delta_secs());
        let tr = transform
            .rotation
            .mul_vec3(vel.extend(0.) * SPEED * time.delta_secs());
        transform.translation += tr;
    }
}
