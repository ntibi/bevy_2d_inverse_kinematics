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
        .add_systems(Update, (move_animal, foot_zones).chain())
        .run();
}

// returns the anchor and the effector
fn spawn_arm(
    pos: Vec2,
    dir: Vec2,
    len: usize,
    dist_constraint: f32,
    color: Color,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> (Entity, Entity) {
    let mut entities = Vec::new();

    let get_limb_world_pos =
        |i: usize| (pos + dir.normalize() * i as f32 * 6.).extend(1. + i as f32);

    for i in 0..len {
        let id = commands
            .spawn((
                Transform::from_translation(pos.extend(1.)),
                // we set the global transform, so that set_parent_in_place works on the same frame
                GlobalTransform::from_translation(get_limb_world_pos(i)),
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
            .with_distance_constraint(dist_constraint)
            .with_angle_constraint(3. * PI / 4.)
            .with_target(get_limb_world_pos(len - 1).xy()),
    );

    (anchor, effector)
}

#[derive(Component)]
struct AnimalThingy;

#[derive(Component)]
struct FootZone {
    foot_entity: Entity,
    max_distance: f32,

    /// if set, the foot will translate to this position
    translate_to: Option<Vec2>,
}

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
    let parts = 5;
    let dist_constraint = 6.;
    let pos = Vec2::new(12., -17.);
    let (anchor, effector) = spawn_arm(
        pos,
        Vec2::new(1., 0.),
        parts,
        dist_constraint,
        color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.entity(id).add_child(anchor);
    commands.entity(id).with_child((
        FootZone {
            max_distance: dist_constraint * parts as f32 * 0.8,
            foot_entity: effector,
            translate_to: None,
        },
        Transform::from_translation(pos.extend(0.)),
    ));
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

fn foot_zones(
    mut foot_zones: Query<(&GlobalTransform, &mut FootZone)>,
    time: Res<Time>,
    mut effectors: Query<(&mut IKConstraint, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    for (transform, mut foot_zone) in foot_zones.iter_mut() {
        let (mut effector, foot_pos) = effectors.get_mut(foot_zone.foot_entity).unwrap();
        let foot_pos = foot_pos.translation().xy();
        let base_pos = transform.translation().xy();

        if let Some(translating) = foot_zone.translate_to {
            if foot_pos.distance(translating) < 1. {
                foot_zone.translate_to = None;
            }

            let diff = translating - foot_pos;
            let diff_len = diff.length();

            let dir = diff.normalize();
            let delta_movement = (dir * SPEED * time.delta_secs()).clamp_length_max(diff_len);
            gizmos.circle_2d(foot_pos + delta_movement, 5., Color::srgb(1., 0., 0.));
            gizmos.circle_2d(translating, 5., Color::srgb(0., 1., 0.));

            effector.target(foot_pos + delta_movement);
        } else {
            gizmos.circle_2d(foot_pos, 5., Color::srgb(0., 1., 0.));
            if foot_pos.distance(base_pos) > foot_zone.max_distance {
                foot_zone.translate_to = Some(base_pos + base_pos - foot_pos);
            }
        }
    }
}
