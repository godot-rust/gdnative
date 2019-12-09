use godot::*;
use specs::prelude::*;
use specs::world::Index;
use std::collections::HashMap;

use crate::sync::{GdSceneLoc, GdSpatial, Template3dEntity, Template3dName};

pub struct GdEntityCreation<'a> {
    pub grandparent: Spatial,
    pub templates: &'a HashMap<Template3dName, Template3dEntity>,
    pub spatials: &'a mut HashMap<Index, Spatial>,

    pub default_parent: Spatial,
}

unsafe impl<'a> Sync for GdEntityCreation<'a> {}
unsafe impl<'a> Send for GdEntityCreation<'a> {}

impl<'a> System<'a> for GdEntityCreation<'a> {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, GdSpatial>,
        ReadStorage<'a, GdSceneLoc>,
    );

    fn run(&mut self, (entities, mut gd_spatials, gd_scenelocs): Self::SystemData) {
        for (entity, gd_sceneloc, gd_spatial_entry) in
            (&entities, &gd_scenelocs, gd_spatials.entries()).join()
        {
            let GdEntityCreation {
                grandparent,
                templates,
                spatials,
                default_parent: _,
            } = self;

            unsafe {
                let parent = if let Some(parent) = grandparent.find_node(
                    GodotString::from_str(gd_sceneloc.parent_name),
                    false,
                    false,
                ) {
                    parent
                } else {
                    // TODO: Create parent here.

                    let mut new_parent = Spatial::new()
                        .cast()
                        .expect("Spatial is a hardcoded subclass of Node! WTF!?");
                    grandparent.add_child(Some(new_parent), false);
                    new_parent.set_name(GodotString::from_str(gd_sceneloc.parent_name));
                    new_parent
                };

                // TODO: Allow moving the spatial
                let gd_spatial = gd_spatial_entry.or_insert_with(|| {
                    // If a this specs entity has no spatial yet...
                    let mut new_spatial = GdEntityCreation::instance_template(
                        parent,
                        gd_sceneloc.template,
                        templates,
                    );

                    new_spatial.set_name(GodotString::from_str(gd_sceneloc.child_name.clone()));
                    let old_spatial = spatials.insert(entity.id(), new_spatial);
                    old_spatial.map(|mut spatial| spatial.queue_free());

                    GdSpatial {
                        curr_template: gd_sceneloc.template,
                    }
                });

                // If a previous frame already created a template.
                if gd_spatial.curr_template != gd_sceneloc.template {
                    let mut new_spatial = GdEntityCreation::instance_template(
                        parent,
                        gd_sceneloc.template,
                        templates,
                    );

                    let old_spatial = spatials.insert(entity.id(), new_spatial);
                    old_spatial.map(|mut spatial| spatial.queue_free());

                    gd_spatial.curr_template = gd_sceneloc.template;

                    new_spatial.set_name(GodotString::from_str(gd_sceneloc.child_name.clone()));
                    // Add code which moves from old world to new world...
                }
            }
        }
    }
}

impl GdEntityCreation<'_> {
    unsafe fn instance_template(
        mut parent: Node,
        template_name: Option<Template3dName>,
        templates: &HashMap<Template3dName, Template3dEntity>,
    ) -> Spatial {
        let template = template_name.and_then(|template_name| templates.get(&template_name));

        let new_spatial = match template {
            Some(Template3dEntity::Scene(packed_scene)) => {
                let scene_opt = instance_scene::<Spatial>(packed_scene).ok();

                scene_opt
            }
            Some(Template3dEntity::Mesh(mesh)) => {
                let mut instance = MeshInstance::new();
                instance.set_mesh(Some(mesh.clone()));
                instance.cast::<Spatial>()
            }
            None | Some(Template3dEntity::None) => None,
        }
        .unwrap_or_else(|| Spatial::new());

        parent.add_child(new_spatial.cast::<Node>(), false);

        new_spatial
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManageErrs {
    CouldNotMakeInstance,
    RootClassNotSpatial(String),
    ResourceNotFound,
    ChildNotFound,
}

pub fn instance_scene<Root>(scene: &PackedScene) -> Result<Root, ManageErrs>
where
    Root: godot::GodotObject,
{
    const GEN_EDIT_STATE_DISABLED: i64 = 0;
    let inst_option = scene.instance(GEN_EDIT_STATE_DISABLED);

    if let Some(instance) = inst_option {
        if let Some(instance_root) = unsafe { instance.cast::<Root>() } {
            Ok(instance_root)
        } else {
            Err(ManageErrs::RootClassNotSpatial(
                unsafe { instance.get_name() }.to_string(),
            ))
        }
    } else {
        Err(ManageErrs::CouldNotMakeInstance)
    }
}
