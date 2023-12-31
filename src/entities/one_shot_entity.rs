use crate::app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency};

pub struct OneShotEntity {
    tag: String,
}

impl OneShotEntity {
    pub fn new<S>(tag: S) -> Self
    where
        S: Into<String>,
    {
        Self { tag: tag.into() }
    }
}

impl TEntity for OneShotEntity {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(self.tag.clone(), UpdateFrequency::Slow, false)
    }

    fn update(&mut self, delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        log::debug!(
            "I am a one-shot entity and will be deleted after this! (delta: {delta_time}ms)"
        );
        vec![EntityAction::Remove(vec![self.tag.clone()])]
    }
}
