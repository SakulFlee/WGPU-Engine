use crate::engine::LogicalDevice;

use super::{BoxedEntity, EntityConfiguration};

pub struct EntityContainer {
    entity_configuration: EntityConfiguration,
    entity: BoxedEntity,
    is_prepared: bool,
}

impl EntityContainer {
    pub fn from_boxed_entity(entity: BoxedEntity) -> Self {
        let entity_configuration = entity.entity_configuration();
        Self {
            entity_configuration,
            entity,
            is_prepared: false,
        }
    }

    pub fn prepare_entity(&mut self, logical_device: &LogicalDevice) {
        match self.entity.prepare_render(logical_device) {
            Ok(_) => self.is_prepared = true,
            Err(e) => {
                log::error!("Error encountered while preparing Entity for rendering! ({e:?})")
            }
        }
    }

    pub fn entity_configuration(&self) -> &EntityConfiguration {
        &self.entity_configuration
    }

    pub fn entity(&self) -> &BoxedEntity {
        &self.entity
    }

    pub fn entity_mut(&mut self) -> &mut BoxedEntity {
        &mut self.entity
    }

    pub fn and_move_entity(self) -> BoxedEntity {
        self.entity
    }

    pub fn is_prepared(&self) -> bool {
        self.is_prepared
    }

    pub fn is_tag(&self, tag: &str) -> bool {
        self.entity_configuration.tag() == tag
    }
}
