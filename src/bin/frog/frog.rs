use bevy::{ecs::entity::EntityHashMap, prelude::*};
use fabrik::ik::{Bone, IKConstraint};
use std::f32::consts::PI;

pub struct FrogPlugin;

impl Plugin for FrogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (input, move_animal, compute_foot_placement).chain());
    }
}

// returns the anchor and the effector
fn spawn_arm(
    pos: Vec2,
    dir: Vec2,
    len: usize,
    dist_constraint: f32,
    start_color: Color,
    end_color: Color,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> (Entity, Entity) {
    let mut entities = Vec::new();

    let get_limb_world_pos =
        |i: usize| (pos + dir.normalize() * i as f32 * 6.).extend(-2. - i as f32);

    for i in 0..len {
        let id = commands
            .spawn((
                Transform::from_translation(pos.extend(-1.)),
                // we set the global transform, so that set_parent_in_place works on the same frame
                GlobalTransform::from_translation(get_limb_world_pos(i)),
                Mesh2d(meshes.add(Circle::new(3.0))),
                MeshMaterial2d(
                    materials.add(Color::from(LinearRgba::from_vec4(
                        start_color
                            .to_linear()
                            .to_vec4()
                            .lerp(end_color.to_linear().to_vec4(), i as f32 / len as f32),
                    ))),
                ),
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
        IKConstraint::new(entities.clone())
            .with_iterations(10)
            .with_bone_data(
                entities
                    .iter()
                    .map(|&e| (e, Bone::new(3. * PI / 4., dist_constraint)))
                    .collect::<EntityHashMap<Bone>>(),
            )
            .with_target(get_limb_world_pos(len - 1).xy()),
    );

    (anchor, effector)
}

#[derive(Component)]
struct AnimalThingy;

#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct AngularVelocity(f32);

#[derive(Component)]
struct FootZone {
    foot_entity: Entity,
    max_distance: f32,

    /// default position for the next step
    default_rest_pos: Vec2,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let color = Color::srgb(0.0, 0.8, 0.0);
    let hand_color = Color::srgb(0.0, 0.0, 0.0);

    let id = commands
        // body
        .spawn((
            Transform::from_translation(Vec3::new(0., 0., 0.)),
            Mesh2d(meshes.add(Ellipse::new(20.0, 30.))),
            MeshMaterial2d(materials.add(color)),
            AnimalThingy,
            Velocity::default(),
            AngularVelocity::default(),
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

    let parts = 5;
    let dist_constraint = 6.;
    let max_distance = dist_constraint * (parts - 1) as f32 * 0.9;

    // bottom right leg
    let pos = Vec2::new(12., -17.);
    let next_step = Vec2::new(14., 2.);
    let (anchor, effector) = spawn_arm(
        pos,
        Vec2::new(1., 0.),
        parts,
        dist_constraint,
        color,
        hand_color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.entity(id).add_child(anchor);
    commands.entity(id).with_child((
        FootZone {
            max_distance,
            foot_entity: effector,
            default_rest_pos: next_step,
        },
        Transform::from_translation(pos.extend(0.)),
    ));

    // bottom left leg
    let pos = Vec2::new(-12., -17.);
    let next_step = Vec2::new(-14., 2.);
    let (anchor, effector) = spawn_arm(
        pos,
        Vec2::new(-1., 0.),
        parts,
        dist_constraint,
        color,
        hand_color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.entity(id).add_child(anchor);
    commands.entity(id).with_child((
        FootZone {
            max_distance,
            foot_entity: effector,
            default_rest_pos: next_step,
        },
        Transform::from_translation(pos.extend(0.)),
    ));

    // top right leg
    let pos = Vec2::new(12., 17.);
    let next_step = Vec2::new(15., 3.);
    let (anchor, effector) = spawn_arm(
        pos,
        Vec2::new(1., 0.),
        parts,
        dist_constraint,
        color,
        hand_color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.entity(id).add_child(anchor);
    commands.entity(id).with_child((
        FootZone {
            max_distance,
            foot_entity: effector,
            default_rest_pos: next_step,
        },
        Transform::from_translation(pos.extend(0.)),
    ));

    // top left leg
    let pos = Vec2::new(-12., 17.);
    let next_step = Vec2::new(-15., 3.);
    let (anchor, effector) = spawn_arm(
        pos,
        Vec2::new(-1., 0.),
        parts,
        dist_constraint,
        color,
        hand_color,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    commands.entity(id).add_child(anchor);
    commands.entity(id).with_child((
        FootZone {
            max_distance,
            foot_entity: effector,
            default_rest_pos: next_step,
        },
        Transform::from_translation(pos.extend(0.)),
    ));
}

const SPEED: f32 = 100.;
const ROTATION_SPEED: f32 = PI;

fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut AngularVelocity), With<AnimalThingy>>,
) {
    for (mut vel, mut angvel) in query.iter_mut() {
        let mut dir = Vec2::ZERO;
        let mut rotation = 0.;

        if keyboard_input.pressed(KeyCode::KeyW) {
            dir += Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            dir -= Vec2::Y;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            dir -= Vec2::X;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            dir += Vec2::X;
        }

        if keyboard_input.pressed(KeyCode::KeyQ) {
            rotation = 1.;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            rotation = -1.;
        }

        **vel = dir.normalize_or_zero();
        **angvel = rotation;
    }
}

fn move_animal(mut query: Query<(&mut Transform, &Velocity, &AngularVelocity)>, time: Res<Time>) {
    for (mut transform, vel, angvel) in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(**angvel * ROTATION_SPEED * time.delta_secs());
        let tr = transform
            .rotation
            .mul_vec3(vel.extend(0.) * SPEED * time.delta_secs());
        transform.translation += tr;
    }
}

fn compute_foot_placement(
    _agent: Query<(&Transform, &Velocity), With<AnimalThingy>>,
    mut foot_zones: Query<(&GlobalTransform, &FootZone, &Parent)>,
    mut effectors: Query<(&mut IKConstraint, &GlobalTransform)>,
    mut gizmos: Gizmos,
) {
    for (transform, foot_zone, _parent) in foot_zones.iter_mut() {
        //let (agent_transform, agent_vel) = agent.get(**parent).unwrap();

        let (mut effector, foot_pos) = effectors.get_mut(foot_zone.foot_entity).unwrap();
        let foot_pos = foot_pos.translation().xy();
        let base_pos = transform.translation().xy();
        let default_rest_pos = base_pos
            + transform
                .rotation()
                .mul_vec3(foot_zone.default_rest_pos.extend(0.))
                .xy();

        gizmos.circle_2d(default_rest_pos, 3., Color::srgb(1., 0., 0.));

        //let movement_dir = agent_transform
        //.rotation
        //.mul_vec3(agent_vel.extend(0.))
        //.xy()
        //.normalize_or_zero();

        gizmos.circle_2d(foot_pos, 1., Color::srgb(0., 1., 1.));
        gizmos.circle_2d(base_pos, foot_zone.max_distance, Color::srgb(0., 1., 0.));
        if foot_pos.distance(base_pos) > foot_zone.max_distance {
            effector.target(default_rest_pos);
        }
    }
}
