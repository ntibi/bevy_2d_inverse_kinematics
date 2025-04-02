use bevy::prelude::*;
use std::f32::consts::PI;

// subset of the capabilities of the IK plugin
// im just keeping it for future reference

mod constraint;

pub use constraint::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(ConstraintPlugin)
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

    let n = 10;
    for i in 0..n {
        let id = commands
            .spawn((
                Transform::from_translation(Vec3::new(i as f32 * 50., 0., 0.)),
                Mesh2d(meshes.add(Circle::new(10.0))),
                MeshMaterial2d(materials.add(color)),
                MaxBend(PI / 2.0),
            ))
            .observe(on_drag_start)
            .observe(on_drag_move)
            .observe(on_drag_end)
            .id();

        if let Some(prev) = prev {
            commands.add_constraint(prev, id, 50.0);
        }

        prev = Some(id);
    }
}

fn on_drag_move(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.entity()) {
        let delta = Vec2::new(drag.delta.x, -drag.delta.y);
        transform.translation += delta.extend(0.0);
    }
}

fn on_drag_start(drag: Trigger<Pointer<DragStart>>, mut commands: Commands) {
    commands.entity(drag.entity()).insert(Moved);
}

fn on_drag_end(drag: Trigger<Pointer<DragEnd>>, mut commands: Commands) {
    commands.entity(drag.entity()).remove::<Moved>();
}
