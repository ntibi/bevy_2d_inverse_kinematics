use bevy::prelude::*;

pub struct ConstraintPlugin;

impl Plugin for ConstraintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_constraints)
            .add_systems(Update, show_constraints)
            .register_type::<Constrained>();
    }
}

#[derive(Reflect, Copy, Clone)]
pub struct DistanceConstraint {
    to: Entity,
    distance: f32,
}

/// this entity has one or multiple constraints
/// constraints need to be two-way
/// if entity A has a constraint to entity B, entity B needs to have the same constraint to entity A
/// add them with `commands.add_constraint(e1, e2, dist)`
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct Constrained(pub Vec<DistanceConstraint>);

/// max angle this entity can maintain with its two neighbours
/// PI means no constraints
/// PI / 2 means it cannot have an angle smaller than 90 degrees
#[derive(Component, Reflect, Deref, DerefMut)]
pub struct MaxBend(pub f32);

/// this entity is being manually moved, ignore constraints
#[derive(Component)]
pub struct Moved;

// TODO prevent infinite recursion by keeping track of which entities have been solved
fn solve_constraint(
    entity: Entity,
    pos: Vec2,
    to: Entity,
    distance: f32,
    transforms: &mut Query<
        (Entity, &mut Transform, &Constrained, Option<&MaxBend>),
        Without<Moved>,
    >,
) {
    // solve this constraint
    let (new_entity, mut transform, constraints, max_bend) = transforms.get_mut(to).unwrap();

    // vector between parent and current entity
    let dir = (transform.translation.xy() - pos).normalize();
    transform.translation = (pos + dir * distance).extend(transform.translation.z);

    let remaining_constraints = constraints
        .iter()
        .filter(|c| c.to != entity)
        .copied()
        .collect::<Vec<_>>();

    // pos of the current entity
    let current_pos = transform.translation.xy();

    if let Some(&MaxBend(max_angle)) = max_bend {
        // vector between current and next entities
        if !remaining_constraints.is_empty() {
            let next_pos = remaining_constraints
                .iter()
                .map(|DistanceConstraint { to, .. }| {
                    transforms.get(*to).unwrap().1.translation.xy()
                })
                .sum::<Vec2>();
            let next_dir = (next_pos - current_pos).normalize();

            // angle between the two vectors
            let angle = dir.angle_to(next_dir);
            if !(angle >= -max_angle && angle <= max_angle) {
                // TODO apply angle constraint
                // not sure if i should move the current entity
                // or the next entity
            }
        }
    }

    // recursively solve constraints
    for DistanceConstraint { to, distance } in remaining_constraints {
        solve_constraint(new_entity, current_pos, to, distance, transforms);
    }
}

fn update_constraints(
    moved_transforms: Query<(Entity, &Transform, &Constrained), With<Moved>>,
    mut transforms: Query<(Entity, &mut Transform, &Constrained, Option<&MaxBend>), Without<Moved>>,
) {
    for (entity, transform, constraints) in moved_transforms.iter() {
        for &DistanceConstraint { to, distance } in constraints.iter() {
            solve_constraint(
                entity,
                transform.translation.xy(),
                to,
                distance,
                &mut transforms,
            );
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
                gizmos.ray_2d(
                    src,
                    10. * transform.rotation.mul_vec3(Vec3::X).truncate().normalize(),
                    Color::srgba(1.0, 0.0, 0.0, 1.),
                );
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
