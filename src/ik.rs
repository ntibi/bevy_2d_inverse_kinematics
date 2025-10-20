use bevy::{
    ecs::query::QueryEntityError, platform::collections::HashMap, prelude::*,
    transform::plugins::TransformSystems,
};
use std::f32::consts::{FRAC_PI_2, PI};

/// add this plugin to your app to have IK constraints solved every frame
pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (map_new_ik, solve_ik, debug_ik)
                .chain()
                .after(TransformSystems::Propagate),
        )
        .register_type::<DebugIK>()
        .register_type::<Bone>()
        .register_type::<JointRest>()
        .register_type::<JointConstraint>()
        .register_type::<IKConstraint>();
    }
}

/// insert this resource to enable debug gizmos
#[derive(Resource, Reflect)]
pub struct DebugIK {
    /// size of the circle to draw on joints
    pub joints: Option<f32>,
    /// draw ik bones
    pub bones: bool,
    /// draw ik joint constraints
    pub constraints: Option<f32>,
}

impl Default for DebugIK {
    fn default() -> Self {
        Self {
            joints: None,
            bones: false,
            constraints: None,
        }
    }
}

impl DebugIK {
    /// *scale* is the size of the gizmos
    pub fn all(scale: f32) -> Self {
        Self {
            joints: Some(scale / 2.),
            bones: false,
            constraints: Some(scale),
        }
    }

    pub fn none() -> Self {
        Self {
            joints: None,
            bones: false,
            constraints: None,
        }
    }
}

/// length constraint of a bone (which is a relation between two `Joint`s)
#[derive(Clone, Debug, Reflect)]
pub struct Bone {
    length: f32,
}

impl Default for Bone {
    fn default() -> Self {
        Self::new(10.0)
    }
}

impl Bone {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

/// absolute angle of the bone in the resting position
#[derive(Clone, Default, Debug, Reflect)]
pub struct JointRest {
    angle: f32,
}

impl JointRest {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }
}

/// angle constraint of a joint
#[derive(Clone, Debug, Reflect)]
pub struct JointConstraint {
    /// max counter clockwise angle from initial angle
    /// must be between -PI and PI
    ccw: f32,
    /// max clockwise angle from initial angle
    /// must be between -PI and PI
    cw: f32,
}

impl Default for JointConstraint {
    fn default() -> Self {
        Self::new(PI / 2., PI / 2.)
    }
}

impl JointConstraint {
    pub fn new(ccw: f32, cw: f32) -> Self {
        Self { ccw, cw }
    }
}

#[derive(Clone, Debug, Reflect)]
pub enum IKTarget {
    None,
    Pos(Vec2),
    Entity(Entity),
}

/// add this component to an entity to make it the effector of an IK chain
/// all the entities in the chain must have a `Transform` and `GlobalTransform` component
/// their transforms and global transforms will be updated to satisfy the IK constraints without breaking the parent-child hierarchy
#[derive(Component, Debug, Reflect)]
pub struct IKConstraint {
    /// target of the IK constraint
    pub target: IKTarget,

    /// path from the anchor of the constraint to the entity holding this component
    /// the first entity in the chain is the anchor
    ///   it won't move, but it can rotate (ie: the shoulder)
    /// the last entity in the chain is the effector
    ///   it will move and rotate to the target (ie: the hand)
    /// so a chain needs at least 3 entities (anchor, joint, effector)
    ///
    /// chain example: [shoulder, elbow, wrist, hand]
    /// the body wont be affected by the IK
    /// the hand will try to be respect `target`
    /// the rest of the joints will accomodate
    pub chain: Vec<Entity>,

    /// bone length for each bone in the chain
    /// it will get computed automatically when the chain is created
    pub bone_data: HashMap<(Entity, Entity), Bone>,

    /// absolute bones angles at each joint ar rest
    /// it will get computed automatically when the chain is created
    pub joint_data: HashMap<Entity, JointRest>,

    /// initial rest rotation of the joint
    /// it will get computed automatically when the chain is created
    pub rest_data: HashMap<Entity, Quat>,

    /// z rotation of the parent of the anchor at rest
    /// it's used to compute the relative angle of the anchor
    /// it will get computed automatically when the chain is created
    pub anchor_parent_rest_rot: f32,

    // joint data for each joint in the chain
    pub joint_constraints: HashMap<Entity, JointConstraint>,

    /// max number of iterations to solve the IK constraint
    pub iterations: usize,

    /// epsilon to consider the constraint solved
    /// must be smaller than the smaller distance constraint
    pub epsilon: f32,
}

impl IKConstraint {
    pub fn new(chain: Vec<Entity>) -> Self {
        Self {
            target: IKTarget::None,
            chain,
            iterations: 10,
            epsilon: 1.0,
            bone_data: HashMap::new(),
            joint_data: HashMap::new(),
            joint_constraints: HashMap::new(),
            rest_data: HashMap::new(),
            anchor_parent_rest_rot: 0.0,
        }
    }

    /// adds a list of joints angles constraint
    pub fn with_joint_constraints(mut self, constraints: Vec<(Entity, JointConstraint)>) -> Self {
        self.joint_constraints.extend(constraints);
        self
    }

    /// set the number of iterations to solve the IK constraint
    /// default is 10
    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_epsilon(mut self, epsilon: f32) -> Self {
        self.epsilon = epsilon;
        self
    }

    pub fn with_target(mut self, target: IKTarget) -> Self {
        self.target = target;
        self
    }

    pub fn set_target(&mut self, target: IKTarget) {
        self.target = target;
    }

    pub fn remove_target(&mut self) {
        self.target = IKTarget::None;
    }

    /// set absolute posiiton of an entity
    /// wether it's an orphan entity or a child of another entity
    fn set_position(
        &self,
        entity: Entity,
        pos: Vec2,
        parents: &Query<&ChildOf>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        match parents.get(entity) {
            Ok(parent) => {
                if let Ok([(mut gtr, mut tr), (parent_gtr, _)]) =
                    transforms.get_many_mut([entity, parent.parent()])
                {
                    let new_global_tr = GlobalTransform::from(Transform {
                        translation: pos.extend(gtr.translation().z),
                        rotation: gtr.rotation(),
                        scale: gtr.scale(),
                    });
                    *tr = new_global_tr.reparented_to(&parent_gtr);
                    *gtr = new_global_tr;
                }
            }
            Err(_) => {
                if let Ok((mut gtr, mut tr)) = transforms.get_mut(entity) {
                    tr.translation = pos.extend(tr.translation.z);
                    *gtr = GlobalTransform::from(*tr);
                }
            }
        }
    }

    /// set absolute rotation of an entity
    /// wether it's an orphan entity or a child of another entity
    fn set_rotation(
        &self,
        entity: Entity,
        rot: f32,
        parents: &Query<&ChildOf>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        let base_rot = self.rest_data.get(&entity).unwrap();
        let base_angle = self.joint_data.get(&entity).unwrap().angle;
        let diff_from_rest = rot - base_angle;

        match parents.get(entity) {
            Ok(parent) => {
                if let Ok([(mut gtr, mut tr), (parent_gtr, _)]) =
                    transforms.get_many_mut([entity, parent.parent()])
                {
                    let new_global_tr = GlobalTransform::from(Transform {
                        translation: gtr.translation(),
                        rotation: Quat::from_rotation_z(diff_from_rest) * *base_rot,
                        scale: gtr.scale(),
                    });

                    *tr = new_global_tr.reparented_to(&parent_gtr);

                    *gtr = new_global_tr;
                }
            }
            Err(_) => {
                if let Ok((mut gtr, mut tr)) = transforms.get_mut(entity) {
                    tr.rotation = *base_rot * Quat::from_rotation_z(diff_from_rest);
                    *gtr = GlobalTransform::from(*tr);
                }
            }
        }
    }

    fn get_effector_dir(
        &self,
        target: Vec2,
        effector_gtr: GlobalTransform,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) -> Vec2 {
        if !(target - effector_gtr.translation().xy())
            .normalize()
            .is_nan()
        {
            (target - effector_gtr.translation().xy()).normalize()
        } else {
            // if effector is already at target pos
            // use the angle from the prev bone
            let prev = self.chain[self.chain.len() - 2];
            let prev_gtr = transforms.get(prev).unwrap().0;
            (effector_gtr.translation().xy() - prev_gtr.translation().xy()).normalize()
        }
    }

    fn solve_iteration(
        &self,
        target: Vec2,
        parents: &Query<&ChildOf>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        let effector = self.chain.last().unwrap();
        let anchor = self.chain.first().unwrap();

        let anchor_gtr = transforms.get(*anchor).unwrap().0.clone();

        // absolute dir of the anchor
        let anchor_dir = match parents.get(*anchor) {
            Ok(parent) => {
                let parent_z_rot = transforms
                    .get(parent.parent())
                    .unwrap()
                    .0
                    .rotation()
                    .to_euler(EulerRot::ZXY)
                    .0;

                Vec2::from_angle(
                    self.joint_data.get(anchor).unwrap().angle + parent_z_rot
                        - self.anchor_parent_rest_rot,
                )
            }
            Err(_) => Vec2::from_angle(self.joint_data.get(anchor).unwrap().angle),
        };

        // bring the effector to the target position
        self.set_position(*effector, target, parents, transforms);

        // pull the chain to the effector
        // while respecting the length constraints
        // iter from effector to anchor
        // e1 will pull e0
        for i in (1..self.chain.len()).rev() {
            let e1 = self.chain[i];
            let e0 = self.chain[i - 1];

            let bone = self.bone_data.get(&(e1, e0)).unwrap();

            let [(e1_gtr, _), (e0_gtr, _)] = transforms.get_many([e1, e0]).unwrap();
            let e1_pos = e1_gtr.translation().xy();
            let e0_pos = e0_gtr.translation().xy();
            let new_e0_pos = e1_pos + (e0_pos - e1_pos).normalize() * bone.length;
            self.set_position(e0, new_e0_pos, parents, transforms);
        }

        let dir = self.get_effector_dir(
            target,
            transforms.get(*effector).unwrap().0.clone(),
            transforms,
        );

        self.set_rotation(*effector, dir.to_angle(), parents, transforms);

        // bring the anchor back to its original position
        self.set_position(*anchor, anchor_gtr.translation().xy(), parents, transforms);

        // use the anchor's (potentially relative, if it has a parent) rotation as the original direction
        // to also apply the angle constraint on the anchor rotation
        let mut prev_dir = anchor_dir;

        // pull the chain to the anchor
        // while respecting the length and angle constraints
        // iter from anchor to effector
        // e0 will pull e1
        // and rotate e0 accordingly
        for i in 0..self.chain.len() - 1 {
            let e0 = self.chain[i];
            let e1 = self.chain[i + 1];

            let [(e1_gtr, _), (e0_gtr, _)] = transforms.get_many([e1, e0]).unwrap();
            let e1_pos = e1_gtr.translation().xy();
            let e0_pos = e0_gtr.translation().xy();

            let mut dir = (e1_pos - e0_pos).normalize();
            let mut dist = e1_pos.distance(e0_pos);

            if let Some(bone) = self.bone_data.get(&(e0, e1)) {
                dist = bone.length;
            }

            let angle = prev_dir.angle_to(dir);
            let rotation = Mat2::from_angle(match self.joint_constraints.get(&e0) {
                Some(&JointConstraint { ccw, cw }) => angle.clamp(-cw, ccw),
                None => angle,
            });

            dir = (rotation * prev_dir).normalize();

            let new_e1_pos = e0_pos + dir * dist;
            self.set_position(e1, new_e1_pos, parents, transforms);

            self.set_rotation(e0, dir.to_angle(), parents, transforms);

            prev_dir = dir;
        }

        // restrain the effector's angle
        // since it doesnt happen in the loop above
        let dir = self.get_effector_dir(
            target,
            transforms.get(*effector).unwrap().0.clone(),
            transforms,
        );
        let angle = prev_dir.angle_to(dir);
        let rotation = Mat2::from_angle(match self.joint_constraints.get(effector) {
            Some(&JointConstraint { ccw, cw }) => angle.clamp(-cw, ccw),
            None => angle,
        });
        let dir = rotation * prev_dir;
        self.set_rotation(*effector, dir.to_angle(), parents, transforms);
    }

    fn solve(
        &self,
        target: Vec2,
        parents: &Query<&ChildOf>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        let effector = self.chain.last().unwrap();

        for _ in 0..self.iterations {
            // early break if both effector constraints are within epsilons
            // or if there are no constrains
            let effector_gtr = transforms.get(*effector).unwrap().0;
            if effector_gtr.translation().xy().distance_squared(target)
                < self.epsilon * self.epsilon
            {
                break;
            }

            self.solve_iteration(target, parents, transforms);
        }
    }
}

pub fn solve_ik(
    ik_constraints: Query<&IKConstraint>,
    parents: Query<&ChildOf>,
    mut transforms: Query<(&mut GlobalTransform, &mut Transform)>,
) {
    for constraint in ik_constraints.iter() {
        let target = match constraint.target {
            IKTarget::None => continue,
            IKTarget::Pos(target) => target,
            IKTarget::Entity(target) => {
                if let Ok((gtr, _)) = transforms.get(target) {
                    gtr.translation().xy()
                } else {
                    warn!("unable to find target entity {}", target);
                    continue;
                }
            }
        };

        constraint.solve(target, &parents, &mut transforms);
    }
}

pub fn map_new_ik(
    mut ik_constraints: Query<&mut IKConstraint, Added<IKConstraint>>,
    transforms: Query<(&Transform, &GlobalTransform)>,
    parents: Query<&ChildOf>,
) {
    for mut ik in &mut ik_constraints {
        ik.anchor_parent_rest_rot = match parents.get(*ik.chain.first().unwrap()) {
            Ok(parent) => {
                transforms
                    .get(parent.parent())
                    .unwrap()
                    .1
                    .rotation()
                    .to_euler(EulerRot::ZXY)
                    .0
            }
            Err(_) => 0.0,
        };

        // cache all the transforms
        // it might be useless perf wise, but it avoid a lot of unwraps
        match ik
            .chain
            .iter()
            .map(|&e| Ok(transforms.get(e)?))
            .into_iter()
            .collect::<Result<Vec<_>, QueryEntityError>>()
        {
            Ok(transforms) => {
                for i in 0..ik.chain.len() {
                    let e = ik.chain[i];
                    let (_, gtr) = transforms[i];

                    ik.rest_data.insert(e, gtr.rotation());

                    if let Some(prev_i) = i.checked_sub(1) {
                        let prev_e = ik.chain[prev_i];
                        let (_, prev_gtr) = transforms[prev_i];

                        let dist = gtr.translation().xy().distance(prev_gtr.translation().xy());
                        ik.bone_data.insert((e, prev_e), Bone::new(dist));
                        ik.bone_data.insert((prev_e, e), Bone::new(dist));
                    }

                    match i {
                        // we are at the anchor
                        // take the direction to the next joint as the angle
                        i if i == 0 => {
                            let anchor_gtr = transforms[0].1;
                            let anchor_child_gtr = transforms[1].1;
                            let dir =
                                anchor_child_gtr.translation().xy() - anchor_gtr.translation().xy();
                            ik.joint_data.insert(e, JointRest::new(dir.to_angle()));
                        }
                        _ => {
                            let (_, prev_gtr) = transforms[i - 1];

                            let dir = gtr.translation().xy() - prev_gtr.translation().xy();

                            ik.joint_data.insert(e, JointRest::new(dir.to_angle()));
                        }
                    }
                }
            }
            Err(e) => {
                warn!("unable to find element of IK chain {}", e);
                continue;
            }
        }
    }
}

fn debug_ik(
    ik_constraints: Query<&IKConstraint>,
    transforms: Query<&GlobalTransform>,
    parents: Query<&ChildOf>,
    mut gizmos: Gizmos,
    debug: Option<Res<DebugIK>>,
) {
    let Some(debug) = debug else { return };

    for constraint in ik_constraints.iter() {
        let anchor = constraint.chain.first().unwrap();
        let anchor_dir = match parents.get(*anchor) {
            Ok(parent) => {
                let parent_z_rot = transforms
                    .get(parent.parent())
                    .unwrap()
                    .rotation()
                    .to_euler(EulerRot::ZXY)
                    .0;

                Vec2::from_angle(
                    constraint.joint_data.get(anchor).unwrap().angle + parent_z_rot
                        - constraint.anchor_parent_rest_rot,
                )
            }
            Err(_) => Vec2::from_angle(constraint.joint_data.get(anchor).unwrap().angle),
        };

        let mut prev_dir = anchor_dir;

        for i in 0..constraint.chain.len() {
            let e = constraint.chain[i];
            let next = constraint.chain.get(i + 1);

            let Ok(gtr) = transforms.get(e) else {
                continue;
            };

            if let Some(joint) = debug.joints {
                gizmos.circle_2d(gtr.translation().xy(), joint, Color::srgb(0., 1., 0.));
            }
            let Some(next) = next else {
                continue;
            };

            if debug.bones {
                gizmos.line_2d(
                    gtr.translation().xy(),
                    transforms.get(*next).unwrap().translation().xy(),
                    Color::srgb(0., 1., 0.),
                );
            }

            if let Some(len) = debug.constraints {
                let Some(&JointConstraint { cw, ccw }) = constraint.joint_constraints.get(&e)
                else {
                    continue;
                };
                let Some(rest_rot) = constraint.rest_data.get(&e) else {
                    continue;
                };
                let Some(&JointRest { angle: rest_angle }) = constraint.joint_data.get(&e) else {
                    continue;
                };

                let arc_dir = Vec2::from_angle(rest_angle) * len;

                let diff_from_rest =
                    gtr.rotation().to_euler(EulerRot::ZXY).0 - rest_rot.to_euler(EulerRot::ZXY).0;

                gizmos.ray_2d(
                    gtr.translation().xy(),
                    Rot2::radians(diff_from_rest) * arc_dir,
                    Color::srgb(1., 0., 0.),
                );

                let bone_dir = (transforms.get(*next).unwrap().translation().xy()
                    - gtr.translation().xy())
                .normalize();
                let angle_offset = prev_dir.angle_to(bone_dir);

                prev_dir = bone_dir.normalize();

                gizmos.arc_2d(
                    Isometry2d {
                        translation: gtr.translation().xy(),
                        //                      -cw because the arc is drawn counter clockwise
                        rotation: Rot2::radians(-cw)
                            //                           FRAC_PI_2  bc the arc is drawn starting from Vec2::Y
                            * Rot2::radians(rest_angle - FRAC_PI_2)
                            * Rot2::radians(diff_from_rest)
                            //              -angle_offset to remove the impact of the rotation of the bone from the arc (which shouldnt turn with the bone)
                            * Rot2::radians(-angle_offset),
                        ..default()
                    },
                    cw + ccw,
                    len,
                    Color::srgb(1., 0.5, 0.),
                );
            }
        }
    }
}
