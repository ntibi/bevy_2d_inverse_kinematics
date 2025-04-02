use std::f32::consts::PI;

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
        .add_systems(Update, move_animal)
        .run();
}

// returns the anchor and the effector
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
                Transform::from_translation(pos.extend(1.)),
                // we set the global transform, so that set_parent_in_place works on the same frame
                GlobalTransform::from_translation(
                    (pos + dir.normalize() * i as f32 * 6.).extend(1. + i as f32),
                ),
                Mesh2d(meshes.add(Circle::new(3.0))),
                MeshMaterial2d(materials.add(color)),
            ))
            .id();

        if let Some(prev) = entities.last() {
            commands.entity(id).set_parent_in_place(*prev);
        }

        entities.push(id);
    }

    let anchor = entities[0];
    let effector = entities[entities.len() - 1];

    commands.entity(effector).insert(
        IKConstraint::new(entities)
            .with_iterations(10)
            .with_distance_constraint(6.)
            .with_angle_constraint(3. * PI / 4.),
    );

    anchor
}

#[derive(Component)]
struct AnimalThingy;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let color = Color::srgb(0.0, 0.0, 1.0);

    let id = commands
        // body
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 1.)),
            Mesh2d(meshes.add(Ellipse::new(20.0, 30.))),
            MeshMaterial2d(materials.add(color)),
            AnimalThingy,
        ))
        .with_children(|parent| {
            // left eye
            parent.spawn((
                Transform::from_translation(Vec3::new(-10., 25., 1.)),
                Mesh2d(meshes.add(Circle::new(5.0))),
                MeshMaterial2d(materials.add(Color::srgba(1., 0., 0., 1.))),
            ));
            // right eye
            parent.spawn((
                Transform::from_translation(Vec3::new(10., 25., 1.)),
                Mesh2d(meshes.add(Circle::new(5.0))),
                MeshMaterial2d(materials.add(Color::srgba(1., 0., 0., 1.))),
            ));
        })
        .id();

    // bottom right arm
    let anchor = spawn_arm(
        Vec2::new(12., -17.),
        Vec2::new(1., 0.),
        5,
        color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.entity(id).add_child(anchor);
}

const SPEED: f32 = 5.;
const ROTATION_SPEED: f32 = 0.1;

fn move_animal(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<AnimalThingy>>,
) {
    for mut transform in query.iter_mut() {
        let mut dir = Vec2::ZERO;
        let mut rotation = 0.;

        if keyboard_input.pressed(KeyCode::KeyW) {
            dir += Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            dir -= Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            rotation = ROTATION_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            rotation = -ROTATION_SPEED;
        }

        transform.rotation *= Quat::from_rotation_z(rotation);
        let tr = transform.rotation.mul_vec3(dir.extend(0.) * SPEED);
        transform.translation += tr;
    }
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
