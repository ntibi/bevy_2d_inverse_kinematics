use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, render::camera::ScalingMode};
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
        .add_systems(Update, zoom)
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

fn zoom(
    //camera: Single<&mut OrthographicProjection, With<Camera>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    //let mut projection = camera.into_inner();
    //projection.scale += -mouse_wheel_input.delta.y * 0.1;
}
