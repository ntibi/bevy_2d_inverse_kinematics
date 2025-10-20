use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};
use bevy_2d_inverse_kinematics::IKPlugin;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use frog::FrogPlugin;

mod frog;

// basic IK example with a procedural frog walking around
// its feet are IK targets
// move the frog around with WASD+QE

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(IKPlugin)
        .add_plugins(FrogPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, zoom)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn zoom(
    mut camera: Single<&mut Projection, With<Camera>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    match **camera {
        Projection::Orthographic(ref mut projection) => {
            projection.scale += -mouse_wheel_input.delta.y * 0.1;
        }
        _ => panic!("only orthographic projection is supported"),
    }
}
