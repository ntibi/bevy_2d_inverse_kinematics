use bevy::prelude::*;

pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_ik);
    }
}

#[derive(Component)]
pub struct IKConstraint {
    /// target position for the end of the chain
    target: Option<Vec2>,
    /// path from the anchor of the constraint to the entity holding this component
    chain: Vec<Entity>,
}

impl IKConstraint {
    pub fn new(chain: Vec<Entity>) -> Self {
        Self {
            target: None,
            chain,
        }
    }

    pub fn target(&mut self, target: Vec2) {
        self.target = Some(target);
    }

    pub fn untarget(&mut self) {
        self.target = None;
    }
}

const DIST_CONSTRAINT: f32 = 50.0;

fn solve(target: Vec2, mut chain: Vec<(Entity, Vec2)>) -> Vec<(Entity, Vec2)> {
    chain.reverse();
    chain[0].1 = target;

    for i in 0..(chain.len() - 1) {
        let (_, pos) = chain[i];
        let (_, ref mut next_pos) = chain[i + 1];

        let dir = (*next_pos - pos).normalize();
        *next_pos = pos + dir * DIST_CONSTRAINT;
    }

    chain.reverse();
    chain
}

fn apply_ik(ik_constraints: Query<&IKConstraint>, mut transforms: Query<&mut Transform>) {
    for constraint in ik_constraints.iter() {
        if let Some(target) = constraint.target {
            let chain = constraint
                .chain
                .iter()
                .map(|entity| (*entity, transforms.get(*entity).unwrap().translation.xy()))
                .collect::<Vec<_>>();

            let chain = solve(target, chain);

            for (entity, new_pos) in chain {
                let mut transform = transforms.get_mut(entity).unwrap();
                transform.translation = new_pos.extend(transform.translation.z);
            }
        }
    }
}
