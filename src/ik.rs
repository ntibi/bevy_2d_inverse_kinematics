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

fn apply_ik(ik_constraints: Query<&IKConstraint>, mut transforms: Query<&mut Transform>) {
    for constraint in ik_constraints.iter() {
        if let Some(target) = constraint.target {
            let mut chain = constraint
                .chain
                .iter()
                .map(|entity| (*entity, transforms.get(*entity).unwrap().translation.xy()))
                .collect::<Vec<_>>();

            for (entity, new_pos) in chain {
                let mut transform = transforms.get_mut(entity).unwrap();
                transform.translation = new_pos.extend(transform.translation.z);
            }
        }
    }
}
