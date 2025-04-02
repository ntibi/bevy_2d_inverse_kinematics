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
) -> (Entity, Entity) {
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

    let anchor = entities[0];
    let effector = entities[entities.len() - 1];

    commands
        .entity(effector)
        .insert(IKConstraint::new(entities, 10));

    (anchor, effector)
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

    spawn_arm(
        Vec2::new(0., 0.),
        Vec2::new(1., 0.),
        5,
        color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    commands
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
        });
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
