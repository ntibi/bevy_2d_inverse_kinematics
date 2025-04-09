use bevy::{color, ecs::query::QueryEntityError, prelude::*, utils::HashMap};
use std::f32::consts::PI;

/// add this plugin to your app to have IK constraints solved every frame
pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (map_new_ik, apply_ik)
                .chain()
                .after(TransformSystem::TransformPropagate),
        );
    }
}

/// length constraint of a bone (which is a relation between two `Joint`s)
#[derive(Clone)]
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

/// angle constraint of a joint
#[derive(Clone)]
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

/// add this component to an entity to make it the effector of an IK chain
/// all the entities in the chain must have a `Transform` and `GlobalTransform` component
/// their transform and global transform will be updated to satisfy the IK constraints
#[derive(Component)]
pub struct IKConstraint {
    /// target position for the effector
    pub target: Option<Vec2>,

    /// target rotation for the effector
    pub target_angle: Option<f32>,

    /// path from the anchor of the constraint to the entity holding this component
    /// the first entity in the chain is the anchor
    ///   it won't move nor rotate (ie: the body)
    /// the last entity in the chain is the effector
    ///   it will move and rotate to the target (ie: the hand)
    /// so a chain needs at least 3 entities (anchor, joint, effector)
    ///
    /// chain example: [body, shoulder, elbow, wrist, hand]
    /// the body wont be affected by the IK
    /// the hand will try to be respect `target` and `target_angle`
    /// the rest of the joints will accomodate
    pub chain: Vec<Entity>,

    /// bone data for each bone in the chain
    pub bone_data: HashMap<(Entity, Entity), Bone>,

    // joint data for each joint in the chain
    pub joint_constraints: HashMap<Entity, JointConstraint>,

    /// max number of iterations to solve the IK constraint
    pub iterations: usize,

    /// epsilon to consider the constraint solved
    /// must be smaller than the smaller distance constraint
    pub epsilon: f32,
    /// epsilon to consider the constraint solved
    /// must be smaller than the smaller angle constraint (in rad)
    pub angle_epsilon: f32,
}

impl IKConstraint {
    pub fn new(chain: Vec<Entity>) -> Self {
        Self {
            target: None,
            target_angle: None,
            chain,
            iterations: 10,
            epsilon: 1.0,
            angle_epsilon: 1.0,
            bone_data: HashMap::new(),
            joint_constraints: HashMap::new(),
        }
    }

    pub fn with_joint_constraints(mut self, constraints: Vec<(Entity, JointConstraint)>) -> Self {
        self.joint_constraints.extend(constraints);
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

    pub fn with_angle_epsilon(mut self, angle_epsilon: f32) -> Self {
        self.angle_epsilon = angle_epsilon;
        self
    }

    pub fn target(&mut self, target: Vec2) {
        self.target = Some(target);
    }

    pub fn target_angle(&mut self, rot: f32) {
        self.target_angle = Some(rot);
    }

    pub fn untarget(&mut self) {
        self.target = None;
        self.target_angle = None;
    }

    /// set absolute posiiton of an entity
    /// wether it's an orphan entity or a child of another entity
    fn set_position(
        &self,
        entity: Entity,
        pos: Vec2,
        parents: &Query<&Parent>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        match parents.get(entity) {
            Ok(parent) => {
                if let Ok([(mut gtr, mut tr), (parent_gtr, _)]) =
                    transforms.get_many_mut([entity, **parent])
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
        parents: &Query<&Parent>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        match parents.get(entity) {
            Ok(parent) => {
                if let Ok([(mut gtr, mut tr), (parent_gtr, _)]) =
                    transforms.get_many_mut([entity, **parent])
                {
                    let new_global_tr = GlobalTransform::from(Transform {
                        translation: gtr.translation(),
                        rotation: Quat::from_rotation_z(rot),
                        scale: gtr.scale(),
                    });
                    *tr = new_global_tr.reparented_to(&parent_gtr);
                    *gtr = new_global_tr;
                }
            }
            Err(_) => {
                if let Ok((mut gtr, mut tr)) = transforms.get_mut(entity) {
                    tr.rotation = Quat::from_rotation_z(rot);
                    *gtr = GlobalTransform::from(*tr);
                }
            }
        }
    }

    fn solve_iteration(
        &self,
        parents: &Query<&Parent>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        let effector = self.chain.last().unwrap();

        if let Some(target) = self.target {
            self.set_position(*effector, target, parents, transforms);
        }
        if let Some(target_angle) = self.target_angle {
            self.set_rotation(*effector, target_angle, parents, transforms);
        }

        let anchor = self.chain.first().unwrap();
        let anchor_gtr = transforms.get(*anchor).unwrap().0.clone();

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
            if e1_pos.distance(e0_pos) > bone.length {
                let new_e0_pos = e1_pos + (e0_pos - e1_pos).normalize() * bone.length;
                self.set_position(e0, new_e0_pos, parents, transforms);
            }
        }

        self.set_position(*anchor, anchor_gtr.translation().xy(), parents, transforms);
        self.set_rotation(
            *anchor,
            anchor_gtr.rotation().to_euler(EulerRot::XYZ).2,
            parents,
            transforms,
        );

        let mut prev_dir: Option<Vec2> = None;

        // pull the chain to the anchor
        // while respecting the length and angle constraints
        // iter from anchor to effector
        // e0 will pull e1
        for i in 0..self.chain.len() - 1 {
            let e0 = self.chain[i];
            let e1 = self.chain[i + 1];

            let [(e1_gtr, _), (e0_gtr, _)] = transforms.get_many([e1, e0]).unwrap();
            let e1_pos = e1_gtr.translation().xy();
            let e0_pos = e0_gtr.translation().xy();

            let mut dir = (e1_pos - e0_pos).normalize();
            let mut dist = e1_pos.distance(e0_pos);
            let mut rot = e1_gtr.rotation().to_euler(EulerRot::XYZ).2;

            if let Some(bone) = self.bone_data.get(&(e0, e1)) {
                dist = bone.length;
            }

            if let Some(prev_dir) = prev_dir {
                if let Some(&JointConstraint { ccw, cw }) = self.joint_constraints.get(&e0) {
                    let angle = prev_dir.angle_to(dir);
                    if angle < -ccw || angle > cw {
                        let angle = angle.clamp(-ccw, cw);
                        let rotation = Mat2::from_angle(angle);
                        dir = rotation * prev_dir;
                    }
                }
            }

            let new_e1_pos = e0_pos + dir * dist;
            self.set_position(e1, new_e1_pos, parents, transforms);
            // TODO
            self.set_rotation(e1, rot, parents, transforms);

            prev_dir = Some(dir);
        }
    }

    fn solve(
        &self,
        parents: &Query<&Parent>,
        transforms: &mut Query<(&mut GlobalTransform, &mut Transform)>,
    ) {
        let effector = self.chain.last().unwrap();

        for _ in 0..self.iterations {
            // early break if both effector constraints are within epsilons
            // or if there are no constrains
            let effector_gtr = transforms.get(*effector).unwrap().0;
            if self.target.map_or(true, |target| {
                effector_gtr.translation().xy().distance_squared(target)
                    < self.epsilon * self.epsilon
            }) && self.target_angle.map_or(true, |target| {
                (target - effector_gtr.rotation().to_euler(EulerRot::XYZ).2).abs()
                    < self.angle_epsilon
            }) {
                break;
            }

            self.solve_iteration(parents, transforms);
        }
    }
}

fn apply_ik(
    ik_constraints: Query<&IKConstraint>,
    parents: Query<&Parent>,
    mut transforms: Query<(&mut GlobalTransform, &mut Transform)>,
) {
    for constraint in ik_constraints.iter() {
        constraint.solve(&parents, &mut transforms);
    }
}

fn map_new_ik(
    mut ik_constraints: Query<&mut IKConstraint, Added<IKConstraint>>,
    entities: Query<&GlobalTransform>,
) {
    for mut ik in &mut ik_constraints {
        match ik
            .chain
            .iter()
            .map(|&e| Ok(entities.get(e)?))
            .into_iter()
            .collect::<Result<Vec<_>, QueryEntityError>>()
        {
            Ok(transforms) => {
                let mut prev: Option<(Entity, &GlobalTransform)> = None;

                for (e, tr) in ik.chain.clone().into_iter().zip(transforms) {
                    if let Some((prev_e, prev_tr)) = prev {
                        let dist = tr.translation().xy().distance(prev_tr.translation().xy());
                        ik.bone_data.insert((e, prev_e), Bone::new(dist));
                        ik.bone_data.insert((prev_e, e), Bone::new(dist));
                    }

                    //println!(
                    //"joint {}: {:.3}",
                    //e,
                    //tr.rotation().to_euler(EulerRot::XYZ).2
                    //);
                    //ik.joint_data
                    //.insert(e, Joint::new(tr.rotation().to_euler(EulerRot::XYZ).2));

                    prev = Some((e, tr));
                }
            }
            Err(e) => {
                warn!("unable to find element of IK chain {}", e);
                continue;
            }
        }
    }
}

//fn debug_ik(
//ik_constraints: Query<&IKConstraint, With<DebugIKConstraint>>,
//entities: Query<&GlobalTransform>,
//mut gizmos: Gizmos,
//) {
//for ik in &ik_constraints {
//for &e in &ik.chain {
//if let Ok(tr) = entities.get(e) {
//gizmos.circle_2d(tr.translation().xy(), 0.01, color::palettes::basic::GREEN);
//}
//}

//for i in 0..ik.chain.len() {
//let e0 = match i {
//i if i > 0 => Some(&ik.chain[i - 1]),
//_ => None,
//};
//let e1 = ik.chain.get(i);
//let e2 = ik.chain.get(i + 1);

//match (e0, e1, e2) {
//(Some(&e0), Some(&e1), Some(&e2)) => {
//if let (Ok(tr0), Ok(tr1), Ok(tr2)) =
//(entities.get(e0), entities.get(e1), entities.get(e2))
//{
//gizmos.line_2d(
//tr1.translation().xy(),
//tr2.translation().xy(),
//color::palettes::basic::GREEN,
//);

//let dir = (tr1.translation().xy() - tr0.translation().xy()).normalize();

//let &JointAngleConstraint { ccw, cw } =
//ik.joint_data.get(&e1).unwrap();

//let distance = ik.bone_data.get(&(e1, e2)).unwrap().length;

//gizmos.arc_2d(
//Isometry2d {
//translation: tr1.translation().xy(),
//rotation: Rot2::radians(-cw)
//* Rot2::radians(-dir.angle_to(Vec2::Y)),
//},
//ccw + cw,
//distance,
//color::palettes::basic::PURPLE,
//);
//}
//}
//_ => (),
//}
//}
//}
//}
