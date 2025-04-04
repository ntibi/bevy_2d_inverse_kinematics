use bevy::{prelude::*, window::PrimaryWindow};

use crate::IKConstraint;

pub struct RiggedModelPlugin;

impl Plugin for RiggedModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update_target).chain());
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("model.gltf")),
    ),));

    //commands.entity(effector).insert(
    //IKConstraint::new(entities)
    //.with_iterations(10)
    //.with_distance_constraint(DIST)
    //.with_angle_constraint(ANGLE),
    //);
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
