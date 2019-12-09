use godot::*;
use specs::prelude::*;

use std::collections::HashMap;

use euclid::{vec3, Angle, Transform3D};
use specs::world::Index;
use sync::Pos;

pub struct GdEntityTransform<'a> {
    pub spatials: &'a mut HashMap<Index, Spatial>,
}

/// InShipSpace is just used to make euler happy and help us track units...
pub struct InShipSpace;

impl<'a, 'b> System<'a> for GdEntityTransform<'b> {
    type SystemData = (Entities<'a>, ReadStorage<'a, Pos>);

    fn run(&mut self, (entities, positions): Self::SystemData) {
        for (entity, pos) in (&entities, &positions).join() {
            let translation = pos.0.translation;
            let rotation = pos.0.rotation;

            let transform = if let Some((rotation_axis, angle)) = rotation.axis_angle() {
                let transform = Transform3D::<_, InShipSpace, InShipSpace>::create_rotation(
                    rotation_axis.x,
                    rotation_axis.y,
                    rotation_axis.z,
                    Angle::radians(angle),
                )
                .post_translate(vec3(translation.x, translation.y, translation.z));

                Transform::from_transform(&transform)
            } else {
                Transform::translate(Vector3::new(translation.x, translation.y, translation.z))
            };

            if let Some(spatial) = self.spatials.get_mut(&entity.id()) {
                unsafe {
                    spatial.set_transform(transform);
                }
            }
        }
    }
}
