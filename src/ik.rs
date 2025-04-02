use bevy::{color, prelude::*};

pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_constraints)
            .add_systems(Update, show_constraints)
            .register_type::<Constrained>();
    }
}

#[derive(Reflect)]
pub struct DistanceConstraint {
    to: Entity,
    distance: f32,
}

#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Constrained(pub Vec<DistanceConstraint>);

fn solve_constraint(pos: Vec2, transforms: Query<(&Transform, Option<&Constrained>)>) {}

fn update_constraints(mut transforms: Query<(&mut Transform, Option<&Constrained>)>) {
    for (mut transform, constrained) in &mut transforms {
        if let Some(constraints) = constrained {
            for constraint in constraints.iter() {
                let src = transform.translation.xy();
                //let dst = transforms.get(constraint.to).unwrap().0.translation.xy();
            }
        }
    }
}

fn show_constraints(transforms: Query<(&Transform, Option<&Constrained>)>, mut gizmos: Gizmos) {
    for (transform, constraints) in transforms.iter() {
        if let Some(constraints) = constraints {
            for constraint in constraints.iter() {
                let src = transform.translation.xy();
                let dst = transforms.get(constraint.to).unwrap().0.translation.xy();
                gizmos.line_2d(src, dst, Color::srgba(0.0, 0.0, 1.0, 0.5));
            }
        }
    }
}

pub struct AddConstraint {
    pub entity1: Entity,
    pub entity2: Entity,
    pub distance: f32,
}

impl Command for AddConstraint {
    fn apply(self, world: &mut World) {
        let mut to_add: Vec<(Entity, DistanceConstraint)> = Vec::new();

        to_add.push((
            self.entity1,
            DistanceConstraint {
                to: self.entity2,
                distance: self.distance,
            },
        ));
        to_add.push((
            self.entity2,
            DistanceConstraint {
                to: self.entity1,
                distance: self.distance,
            },
        ));

        let mut constrained_query = world.query::<&mut Constrained>();

        for (entity, constraint) in to_add {
            if let Ok(mut constrained) = constrained_query.get_mut(world, entity) {
                constrained.push(constraint);
            } else {
                if let Ok(mut entity) = world.get_entity_mut(entity) {
                    entity.insert(Constrained(vec![constraint]));
                }
            }
        }
    }
}

pub trait AddConstraintExt {
    fn add_constraint(&mut self, entity1: Entity, entity2: Entity, distance: f32);
}

impl<'w, 's> AddConstraintExt for Commands<'w, 's> {
    /// add a two-way IK constraint between two entities
    fn add_constraint(&mut self, entity1: Entity, entity2: Entity, distance: f32) {
        self.queue(AddConstraint {
            entity1,
            entity2,
            distance,
        });
    }
}
