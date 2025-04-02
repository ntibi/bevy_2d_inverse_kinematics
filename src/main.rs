use bevy::{prelude::*, window::PrimaryWindow};

mod ik;

pub use ik::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(IKPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_target)
        .run();
}

fn spawn_arm(
    pos: Vec2,
    dir: Vec2,
    len: usize,
    color: Color,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> Entity {
    let mut entities = Vec::new();

    for i in 0..len {
        let id = commands
            .spawn((
                Transform::from_translation((pos + dir * i as f32 * 20.).extend(0.)),
                Mesh2d(meshes.add(Circle::new(10.0))),
                MeshMaterial2d(materials.add(color)),
            ))
            .id();

        entities.push(id);
    }

    let last = entities[entities.len() - 1];

    commands
        .entity(last)
        .insert(IKConstraint::new(entities, 10));

    last
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let color = Color::srgb(0.0, 0.0, 1.0);

    spawn_arm(
        Vec2::new(0., 0.),
        Vec2::new(1., 0.),
        5,
        color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

fn update_target(
    primary_window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut IKConstraint>,
) {
    let (main_camera, main_camera_transform) = *camera;
    let pos = primary_window.cursor_position().and_then(|cursor_pos| {
        main_camera
            .viewport_to_world_2d(main_camera_transform, cursor_pos)
            .ok()
    });

    if let Some(pos) = pos {
        for mut constraint in query.iter_mut() {
            constraint.target(pos);
        }
    }
}
