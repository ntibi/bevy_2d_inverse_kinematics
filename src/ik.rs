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

/// constraints of a bone (which is a relation between two `Joint`
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

#[derive(Clone, Default)]
pub struct Joint {
    /// default angle of this joint (in rad)
    /// 0. means a 0 angle from Vec2::X
    angle: f32,
}

impl Joint {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }
}

/// add this component to an entity to make it the effector of an IK chain
#[derive(Component)]
pub struct IKConstraint {
    /// target position for the end of the chain
    target: Option<Vec2>,
    /// path from the anchor of the constraint to the entity holding this component
    chain: Vec<Entity>,

    /// bone data for each bone in the chain
    bone_data: HashMap<(Entity, Entity), Bone>,

    /// bone data for each bone in the chain
    joint_data: HashMap<Entity, Joint>,

    /// max number of iterations to solve the IK constraint
    iterations: usize,

    /// epsilon to consider the constraint solved
    /// must be smaller than the smaller distance constraint
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
            joint_data: HashMap::new(),
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

    pub fn with_joint_data(mut self, joint_data: Vec<(Entity, Joint)>) -> Self {
        self.joint_data.extend(joint_data.into_iter());
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

    pub fn with_epsilon(mut self, epsilon: f32) -> Self {
        self.epsilon = epsilon;
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

                let forward_bone = match next_pos - new_pos {
                    v if v.length() < constraint.epsilon => {
                        if let Some((_, prev_pos)) = chain.get(i - 1) {
                            // use the backward bone, bc we got no forward bone
                            new_pos - prev_pos
                        } else {
                            warn!("no prev pos to compute angle for joint {}", entity);
                            // next - new == 0
                            // so we took new - prev
                            // to compute an angle for the joint
                            // (its probably the effector, since other joints have a distance constraint
                            //   and cant't overlap with another joint, whereas the effector tries to be ON the target)
                            // but we didnt find any prev pos
                            // so either we're not the effector, but that's weird
                            //   considering we have a non zero distance constraint we shouldnt have overlapping points
                            // or there is an issue in the code somehow
                            Vec2::Y
                        }
                    }
                    v => v,
                };
                let angle = Vec2::X.angle_to(forward_bone);
                let rotation = Quat::from_rotation_z(
                    constraint
                        .joint_data
                        .get(&entity)
                        .cloned()
                        .unwrap_or_default()
                        .angle
                        + angle,
                );

                let (parent, _, _) = transforms.get(entity).unwrap();
                if let Some(parent) = parent {
                    // if parent
                    // do the GlobalTransform to local conversion

                    let (_, parent_global_tr, _) = transforms.get(**parent).unwrap();
                    let parent_global_tr = parent_global_tr.clone();
                    let (_, _, transform) = transforms.get(entity).unwrap();
                    let new_pos = new_pos.extend(transform.translation.z);

                    let (_, mut global_tr, mut transform) = transforms.get_mut(entity).unwrap();
                    let new_global_tr = GlobalTransform::from(Transform {
                        translation: new_pos,
                        rotation,
                        scale: transform.scale,
                    });

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
