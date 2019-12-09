use crate::sync::GdSpatial;
use godot::*;
use specs::prelude::*;
use specs::world::Index;
use std::collections::HashMap;

pub struct GDDeleteOldSpatials {
    gdspatial_reader: ReaderId<ComponentEvent>,
}
impl GDDeleteOldSpatials {
    pub fn new(world: &mut specs::World) -> GDDeleteOldSpatials {
        GDDeleteOldSpatials {
            gdspatial_reader: world.write_storage::<GdSpatial>().register_reader(),
        }
    }

    pub fn apply(
        &mut self,
        gd_spatials: ReadStorage<GdSpatial>,
        spatials: &mut HashMap<Index, Spatial>,
    ) {
        let mut new_gd_spatials = BitSet::new();
        let mut removed_gd_spatials = BitSet::new();

        let events = gd_spatials.channel().read(&mut self.gdspatial_reader);
        proc_add_remove_events(&mut new_gd_spatials, &mut removed_gd_spatials, events);

        for deleted_id in removed_gd_spatials {
            if let Some(mut spatial) = spatials.remove(&deleted_id) {
                unsafe {
                    spatial.queue_free();
                }
            }
        }
    }
}

pub fn proc_add_remove_events<'a>(
    new_itemgrids: &mut BitSet,
    removed_itemgrids: &mut BitSet,
    events: impl Iterator<Item = &'a ComponentEvent>,
) {
    for grid_event in events {
        match grid_event {
            ComponentEvent::Inserted(id) => {
                new_itemgrids.add(*id);
            }
            ComponentEvent::Removed(id) => {
                // Note - we cannot access entities which have been removed, but we can remove
                //  relevant links by ID#
                removed_itemgrids.add(*id);
            }
            ComponentEvent::Modified(_) => (),
        }
    }
}
