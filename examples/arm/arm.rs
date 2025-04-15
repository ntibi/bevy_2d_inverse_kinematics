use bevy::{prelude::*, window::PrimaryWindow};
use bevy_2d_inverse_kinematics::{DebugIK, IKConstraint, JointConstraint};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use std::f32::consts::PI;

pub struct ArmPlugin;

impl Plugin for ArmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update_target).chain())
            .add_plugins(ResourceInspectorPlugin::<DebugIK>::default())
            .init_resource::<DebugIK>();
    }
}

const LIMBS: usize = 5;
const DIST: f32 = 50.;
const ANGLE: f32 = PI / 2.;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("press RIGHT MOUSE BUTTON to manually set IK target");

    let start_color = Color::srgb(0., 0., 0.8);
    let end_color = Color::srgb(0., 0., 0.2);

    let mut entities = Vec::new();

    for i in 0..LIMBS {
        let id = commands
            .spawn((
                Transform::from_translation(Vec3::new(DIST * i as f32, 0., i as f32)),
                Mesh2d(meshes.add(Circle::new(20.))),
                MeshMaterial2d(materials.add(Color::from(LinearRgba::from_vec4(
                    start_color.to_linear().to_vec4().lerp(
                        end_color.to_linear().to_vec4(),
                        1. - i as f32 / LIMBS as f32,
                    ),
                )))),
            ))
            .with_child((
                Transform::from_translation(Vec3::new(10., 0., 1.)),
                Mesh2d(meshes.add(Circle::new(3.))),
                MeshMaterial2d(materials.add(Color::srgb(1., 0., 0.))),
            ))
            .id();

        entities.push(id);
    }

    let effector = entities[entities.len() - 1];

    commands.entity(effector).insert(
        IKConstraint::new(entities.clone())
            .with_iterations(10)
            .with_joint_constraints(
                entities
                    .iter()
                    .map(|e| (*e, JointConstraint::new(ANGLE, ANGLE)))
                    .collect::<Vec<_>>(),
            ),
    );
}

fn update_target(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut IKConstraint>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !buttons.pressed(MouseButton::Right) {
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
