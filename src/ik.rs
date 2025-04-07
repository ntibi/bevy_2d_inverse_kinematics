use bevy::{prelude::*, utils::HashMap};
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

#[derive(Clone)]
pub struct Bone {
    max_angle: f32,
    length: f32,
}

impl Default for Bone {
    fn default() -> Self {
        Self {
            max_angle: 3. * PI / 4.,
            length: 10.0,
        }
    }
}

impl Bone {
    pub fn new(max_angle: f32, length: f32) -> Self {
        Self { max_angle, length }
    }
}

#[derive(Component)]
pub struct IKConstraint {
    /// target position for the end of the chain
    target: Option<Vec2>,
    /// path from the anchor of the constraint to the entity holding this component
    chain: Vec<Entity>,

    /// bone data for each bone in the chain
    bone_data: HashMap<(Entity, Entity), Bone>,

    /// max number of iterations to solve the IK constraint
    iterations: usize,

    /// epsilon to consider the constraint solved
    epsilon: f32,
}

impl IKConstraint {
    pub fn new(chain: Vec<Entity>) -> Self {
        Self {
            target: None,
            chain,
            iterations: 10,
            epsilon: 1.0,
            bone_data: HashMap::new(),
        }
    }

    pub fn with_bone_data(mut self, bone_data: Vec<(Entity, Entity, Bone)>) -> Self {
        let mut bone_map = HashMap::new();

        for (entity_a, entity_b, bone) in bone_data {
            bone_map.insert((entity_a, entity_b), bone.clone());
            bone_map.insert((entity_b, entity_a), bone.clone());
        }

        self.bone_data.extend(bone_map);

        self
    }

    /// apply the same bone data to all bones in the chain
    pub fn with_single_bone_data(mut self, bone: Bone) -> Self {
        for i in 0..self.chain.len() - 1 {
            let entity_a = self.chain[i];
            let entity_b = self.chain[i + 1];
            self.bone_data.insert((entity_a, entity_b), bone.clone());
            self.bone_data.insert((entity_b, entity_a), bone.clone());
        }
        self
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
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
    bone_data: &HashMap<(Entity, Entity), Bone>,
    target: Vec2,
    mut chain: Vec<(Entity, Vec2)>,
    clamp_angle: bool,
) -> Vec<(Entity, Vec2)> {
    chain.reverse();
    chain[0].1 = target;

    let mut prev_dir: Option<Vec2> = None;

    for i in 0..(chain.len() - 1) {
        let (entity, pos) = chain[i];
        let (next_entity, ref mut next_pos) = chain[i + 1];
        let bone = bone_data
            .get(&(entity, next_entity))
            .cloned()
            .unwrap_or_default();

        let mut dir = (*next_pos - pos).normalize();

        if let Some(prev_dir) = prev_dir {
            let angle = prev_dir.angle_to(dir);

            if clamp_angle {
                if angle > bone.max_angle || angle < -bone.max_angle {
                    let clamped_angle = angle.clamp(-bone.max_angle, bone.max_angle);
                    let rotation = Mat2::from_angle(clamped_angle);
                    dir = rotation * prev_dir;
                }
            }
        }

        *next_pos = pos + dir * bone.length;

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
                chain = solve(&constraint.bone_data, target, chain, false);
                chain.reverse();
                // only apply rotation constraint on the backward pass
                chain = solve(&constraint.bone_data, anchor, chain, true);
                chain.reverse();

                if chain[chain.len() - 1].1.distance(target) < constraint.epsilon {
                    break;
                }
            }

            for i in 0..chain.len() {
                let (entity, new_pos) = chain[i];
                let next_pos = match i {
                    // get the next position in the chain, to compute the angle
                    // if none, it means were at the effector, so use the target as the next position
                    i if i == chain.len() - 1 => target,
                    _ => chain[i + 1].1,
                };
                let rotation = Quat::from_rotation_z(Vec2::X.angle_to(next_pos - new_pos));

                let (parent, _, _) = transforms.get(entity).unwrap();
                if let Some(parent) = parent {
                    // if parent
                    // do the GlobalTransform to local conversion

                    let (_, parent_global_tr, _) = transforms.get(**parent).unwrap();
                    let parent_global_tr = parent_global_tr.clone();
                    let (_, _, transform) = transforms.get(entity).unwrap();
                    let new_pos = new_pos.extend(transform.translation.z);

                    let new_global_tr = GlobalTransform::from(Transform {
                        translation: new_pos,
                        rotation,
                        scale: transform.scale,
                    });

                    let (_, mut global_tr, mut transform) = transforms.get_mut(entity).unwrap();
                    *transform = new_global_tr.reparented_to(&parent_global_tr);
                    // here we re-do the job of propagate_transforms
                    // because we are scheduled to run after it's done (to have the hierarchy movement applied)
                    // but we still need the transform and global transform to be in synnc
                    *global_tr = new_global_tr;
                } else {
                    let (_, mut global_tr, mut transform) = transforms.get_mut(entity).unwrap();
                    // if no parent, just set the translation
                    transform.translation = new_pos.extend(transform.translation.z);
                    transform.rotation = rotation;
                    *global_tr = GlobalTransform::from(*transform);
                }
            }
        }
    }
}
