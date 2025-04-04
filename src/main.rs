use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod arm;
mod frog;
mod ik;

pub use arm::*;
pub use frog::*;
pub use ik::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(IKPlugin)
        //.add_plugins(FrogPlugin)
        .add_plugins(ArmPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, zoom)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn zoom(
    camera: Single<&mut OrthographicProjection, With<Camera>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    let mut projection = camera.into_inner();
    projection.scale += -mouse_wheel_input.delta.y * 0.1;
}
