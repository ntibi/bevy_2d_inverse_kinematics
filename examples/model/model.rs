use std::f32::consts::PI;

use bevy::{prelude::*, scene::SceneInstanceReady, window::PrimaryWindow};
use bevy_2d_inverse_kinematics::{
    Bone, DebugIKConstraint, IKConstraint, Joint, JointAngleConstraint,
};

pub struct RiggedModelPlugin;

const SPEED: f32 = 1.;
const ROTATION_SPEED: f32 = PI;

impl Plugin for RiggedModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, configure_gizmos))
            .add_systems(Update, (input, movement).chain())
            .add_systems(Update, (update_target).chain());
    }
}

fn configure_gizmos(mut conf: ResMut<GizmoConfigStore>) {
    for (_, c, _) in conf.iter_mut() {
        c.depth_bias = -1.;
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Visibility::Visible,
            Movable,
            Velocity::default(),
            AngularVelocity::default(),
            // TODO test with and without this initial transform
            //Transform::from_rotation(Quat::from_rotation_z(PI / 2.)),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("char.gltf"))),
        ))
        .observe(map_ik);
}

fn get_bones<const N: usize>(
    start: Entity,
    keys: [&str; N],
    query: &Query<(Option<&Name>, Option<&Children>)>,
) -> Option<[Entity; N]> {
    let mut found = [None; N];

    let mut to_visit = vec![start];

    while let Some(entity) = to_visit.pop() {
        let (name, children) = query.get(entity).unwrap();

        for (i, key) in keys.iter().enumerate() {
            if let Some(name) = name {
                if name.as_str() == *key {
                    found[i] = Some(entity);
                }
            }
        }

        for children in children.iter() {
            to_visit.extend(children.iter());
        }
    }

    found
        .into_iter()
        .collect::<Option<Vec<_>>>()?
        .try_into()
        .ok()
}

fn map_ik(
    trigger: Trigger<SceneInstanceReady>,
    query: Query<(Option<&Name>, Option<&Children>)>,
    mut commands: Commands,
) {
    let [left_arm, left_forearm, left_hand, left_hand_effector, right_arm, right_forearm, right_hand, right_hand_effector] =
        match get_bones(
            trigger.entity(),
            [
                "left arm bone",
                "left forearm bone",
                "left hand bone",
                "left hand effector",
                "right arm bone",
                "right forearm bone",
                "right hand bone",
                "right hand effector",
            ],
            &query,
        ) {
            None => {
                warn!("skipping IK mapping for {}", trigger.entity());
                return;
            }
            Some(bones) => bones,
        };

    commands.entity(left_hand).insert((
        DebugIKConstraint,
        IKConstraint::new(vec![left_arm, left_forearm, left_hand, left_hand_effector])
            .with_iterations(1)
            .with_epsilon(0.001)
            .with_angle_constraints(vec![
                (left_arm, JointAngleConstraint::new(0., PI / 2.)),
                (left_forearm, JointAngleConstraint::new(0., PI)),
                (left_hand, JointAngleConstraint::new(PI / 2., PI / 2.)),
            ]),
    ));
    //commands.entity(right_hand).insert((
    //DebugIKConstraint,
    //IKConstraint::new(vec![
    //right_arm,
    //right_forearm,
    //right_hand,
    //right_hand_effector,
    //])
    //.with_iterations(100)
    //.with_epsilon(0.001)
    //.with_angle_constraints(vec![
    //(right_arm, JointAngleConstraint::new(PI / 2., 0.)),
    //(right_forearm, JointAngleConstraint::new(PI, 0.)),
    //(right_hand, JointAngleConstraint::new(PI / 2., PI / 2.)),
    //]),
    //));
}

fn update_target(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut IKConstraint>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera.single();
    let window = window.single();

    if let Some(pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        for mut ik in query.iter_mut() {
            ik.target(pos);
        }
    }
}

#[derive(Component)]
struct Movable;

#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Default, Deref, DerefMut)]
struct AngularVelocity(f32);

fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut AngularVelocity), With<Movable>>,
) {
    for (mut vel, mut angvel) in query.iter_mut() {
        let mut dir = Vec2::ZERO;
        let mut rotation = 0.;

        if keyboard_input.pressed(KeyCode::KeyW) {
            dir += Vec2::X;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            dir -= Vec2::X;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            dir -= Vec2::Y;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            dir += Vec2::Y;
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

fn movement(mut query: Query<(&mut Transform, &Velocity, &AngularVelocity)>, time: Res<Time>) {
    for (mut transform, vel, angvel) in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_z(**angvel * ROTATION_SPEED * time.delta_secs());
        let tr = transform
            .rotation
            .mul_vec3(vel.extend(0.) * SPEED * time.delta_secs());
        transform.translation += tr;
    }
}
