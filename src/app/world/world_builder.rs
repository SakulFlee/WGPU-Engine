use cgmath::Vector3;
use wgpu::Color;

use crate::engine::{LogicalDevice, StandardAmbientLight, StandardPointLight};

use super::{BoxedEntity, EntityTagDuplicationBehaviour, World};

// TODO: Move out
#[derive(Debug, Clone, Copy, Default)]
pub struct WColor {
    pub r: f32,
    pub b: f32,
    pub g: f32,
}

impl From<(f32, f32, f32)> for WColor {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            r: value.0,
            b: value.1,
            g: value.2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<(f32, f32, f32)> for WPosition {
    fn from(value: (f32, f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WAmbientLight {
    pub color: WColor,
    pub strength: f32,
}

impl From<((f32, f32, f32), f32)> for WAmbientLight {
    fn from(value: ((f32, f32, f32), f32)) -> Self {
        Self {
            color: value.0.into(),
            strength: value.1,
        }
    }
}

impl From<WColor> for Vector3<f32> {
    fn from(val: WColor) -> Self {
        Vector3 {
            x: val.r,
            y: val.g,
            z: val.b,
        }
    }
}

impl From<WPosition> for Vector3<f32> {
    fn from(val: WPosition) -> Self {
        Vector3 {
            x: val.x,
            y: val.y,
            z: val.z,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WPointLight {
    pub color: WColor,
    pub position: WPosition,
    pub strength: f32,
}

impl From<((f32, f32, f32), (f32, f32, f32), f32)> for WPointLight {
    fn from(value: ((f32, f32, f32), (f32, f32, f32), f32)) -> Self {
        Self {
            color: value.0.into(),
            position: value.1.into(),
            strength: value.2,
        }
    }
}

pub struct WorldBuilder {
    clear_color: Option<Color>,
    entity_tag_duplication_behaviour: Option<EntityTagDuplicationBehaviour>,
    entities: Vec<BoxedEntity>,
    ambient_light: Option<WAmbientLight>,
    point_light: [Option<WPointLight>; 4],
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            clear_color: None,
            entity_tag_duplication_behaviour: None,
            entities: vec![],
            ambient_light: None,
            point_light: [None, None, None, None],
        }
    }

    pub fn build(self, logical_device: &LogicalDevice) -> World {
        let ambient_light_raw = self.ambient_light.unwrap_or(((1.0, 1.0, 1.0), 0.1).into());
        let ambient_light = StandardAmbientLight::new(
            logical_device,
            ambient_light_raw.color.into(),
            ambient_light_raw.strength,
        );

        let mut point_lights = vec![];
        for point_light in self.point_light {
            let point_light_data = point_light.unwrap_or_default();

            let point_light = StandardPointLight::new(
                logical_device,
                point_light_data.color.into(),
                point_light_data.position.into(),
                point_light_data.strength,
                point_light.is_some(),
            );
            point_lights.push(point_light);
        }

        let mut world = World {
            clear_color: self.clear_color.unwrap_or(Color::BLACK),
            entity_tag_duplication_behaviour: self
                .entity_tag_duplication_behaviour
                .unwrap_or(EntityTagDuplicationBehaviour::WarnOnDuplication),
            entities: vec![],
            ambient_light,
            point_lights: [
                // Take 4x times the zero'th entry. Assumes there are 4 lights.
                point_lights.remove(0),
                point_lights.remove(0),
                point_lights.remove(0),
                point_lights.remove(0),
            ],
        };

        for entity in self.entities {
            world.add_entity(entity);
        }

        world
    }

    pub fn with_clear_color(mut self, color: Color) -> Self {
        self.clear_color = Some(color);
        self
    }

    pub fn with_entity_tag_duplication_behaviour(
        mut self,
        entity_tag_duplication_behaviour: EntityTagDuplicationBehaviour,
    ) -> Self {
        self.entity_tag_duplication_behaviour = Some(entity_tag_duplication_behaviour);
        self
    }

    pub fn with_entities(mut self, entities: Vec<BoxedEntity>) -> Self {
        self.entities.extend(entities);
        self
    }

    pub fn with_ambient_light(mut self, color: (f32, f32, f32), strength: f32) -> Self {
        self.ambient_light = Some((color, strength).into());
        self
    }

    pub fn with_point_light(
        mut self,
        slot: usize,
        color: Vector3<f32>,
        position: Vector3<f32>,
        strength: f32,
    ) -> Self {
        self.point_light[slot] = Some((color.into(), position.into(), strength).into());
        self
    }
}

impl Default for WorldBuilder {
    fn default() -> Self {
        Self::new()
    }
}
