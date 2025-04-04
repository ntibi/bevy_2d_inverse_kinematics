use bevy::prelude::*;
use std::f32::consts::PI;

pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            apply_ik.after(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component)]
pub struct IKConstraint {
    /// target position for the end of the chain
    target: Option<Vec2>,
    /// path from the anchor of the constraint to the entity holding this component
    chain: Vec<Entity>,

    /// max number of iterations to solve the IK constraint
    iterations: usize,

    /// epsilon to consider the constraint solved
    epsilon: f32,

    distance_constraint: f32,
    angle_constraint: f32,
}

impl IKConstraint {
    pub fn new(chain: Vec<Entity>) -> Self {
        Self {
            target: None,
            chain,
            iterations: 10,
            epsilon: 1.0,
            distance_constraint: 10.,
            angle_constraint: 3. * PI / 4.,
        }
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_distance_constraint(mut self, distance_constraint: f32) -> Self {
        self.distance_constraint = distance_constraint;
        self
    }

    pub fn with_angle_constraint(mut self, angle_constraint: f32) -> Self {
        self.angle_constraint = angle_constraint;
        self
    }

    pub fn with_target(mut self, target: Vec2) -> Self {
        self.target = Some(target);
        self
    }

    pub fn target(&mut self, target: Vec2) {
        self.target = Some(target);
    }

    pub fn untarget(&mut self) {
        self.target = None;
    }
}

fn solve(
    target: Vec2,
    mut chain: Vec<(Entity, Vec2)>,
    dist: f32,
    max_angle: f32,
    clamp_angle: bool,
) -> Vec<(Entity, Vec2)> {
    chain.reverse();
    chain[0].1 = target;

    let mut prev_dir: Option<Vec2> = None;

    for i in 0..(chain.len() - 1) {
        let (_, pos) = chain[i];
        let (_, ref mut next_pos) = chain[i + 1];

        let mut dir = (*next_pos - pos).normalize();

        if clamp_angle {
            if let Some(prev_dir) = prev_dir {
                let angle = prev_dir.angle_to(dir);
                if angle > max_angle || angle < -max_angle {
                    let clamped_angle = angle.clamp(-max_angle, max_angle);
                    let rotation = Mat2::from_angle(clamped_angle);
                    dir = rotation * prev_dir;
                }
            }
        }

        *next_pos = pos + dir * dist;

        prev_dir = Some(dir);
    }

    chain.reverse();
    chain
}

fn apply_ik(
    ik_constraints: Query<&IKConstraint>,
    mut transforms: Query<(Option<&Parent>, &mut GlobalTransform, &mut Transform)>,
) {
    for constraint in ik_constraints.iter() {
        if let Some(target) = constraint.target {
            let mut chain = constraint
                .chain
                .iter()
                .map(|entity| {
                    (
                        *entity,
                        transforms.get(*entity).unwrap().1.translation().xy(),
                    )
                })
                .collect::<Vec<_>>();

            let anchor = chain[0].1;

            for _ in 0..constraint.iterations {
                chain = solve(
                    target,
                    chain,
                    constraint.distance_constraint,
                    constraint.angle_constraint,
                    false,
                );
                chain.reverse();
                // only apply rotation constraint on the backward pass
                chain = solve(
                    anchor,
                    chain,
                    constraint.distance_constraint,
                    constraint.angle_constraint,
                    true,
                );
                chain.reverse();

                if chain[chain.len() - 1].1.distance(target) < constraint.epsilon {
                    break;
                }
            }

            for (entity, new_pos) in chain {
                let (parent, _, _) = transforms.get(entity).unwrap();
                if let Some(parent) = parent {
                    // if parent
                    // do the GlobalTransform to local conversion

                    let (_, parent_global_tr, _) = transforms.get(**parent).unwrap();
                    let parent_global_tr = parent_global_tr.clone();
                    let (_, _, transform) = transforms.get(entity).unwrap();
                    let new_pos = new_pos.extend(transform.translation.z);

                    // compute translation from world space to parent space
                    let new_translation = parent_global_tr
                        .compute_matrix()
                        .inverse()
                        .transform_point3(new_pos)
                        .xy()
                        .extend(transform.translation.z);

                    let (_, mut global_tr, mut transform) = transforms.get_mut(entity).unwrap();
                    transform.translation = new_translation;
                    // here we re-do the job of propagate_transforms
                    // because we are scheduled to run after it's done (to have the hierarchy movement applied)
                    // but we still need the transform and global transform to be in synnc
                    *global_tr = parent_global_tr.mul_transform(*transform);
                } else {
                    let (_, mut global_tr, mut transform) = transforms.get_mut(entity).unwrap();
                    // if no parent, just set the translation
                    transform.translation = new_pos.extend(transform.translation.z);
                    *global_tr = GlobalTransform::from(*transform);
                }
            }
        }
    }
}
