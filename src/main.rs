use bevy::prelude::*;

mod ik;

pub use ik::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(IKPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let color = Color::srgb(0.0, 0.0, 1.0);
    let mut prev = None;

    for i in 0..10 {
        let id = commands
            .spawn((
                Transform::from_translation(Vec3::new(i as f32 * 50., 0., 0.)),
                Mesh2d(meshes.add(Circle::new(10.0))),
                MeshMaterial2d(materials.add(color)),
            ))
            .observe(on_drag_move)
            .id();
        if let Some(prev) = prev {
            commands.add_constraint(prev, id, 50.0);
        }

        prev = Some(id);
    }
}

fn on_drag_move(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.entity()) {
        transform.translation += Vec3::new(drag.delta.x, -drag.delta.y, 0.0);
    }
}
