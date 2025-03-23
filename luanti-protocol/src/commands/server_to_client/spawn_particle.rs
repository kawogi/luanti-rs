use crate::{
    types::{LongString, MapNode, RangedParameter, TileAnimationParams, v3f},
    wire::{
        deser::{Deserialize, DeserializeResult, Deserializer},
        ser::{Serialize, SerializeResult, Serializer},
    },
};
use anyhow::Context;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

use super::add_particle_spawner::{CommonParticleParams, ServerParticleTexture};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SpawnParticleCommand {
    pub parameters: ParticleParameters,
}

/// This is the send format used by `SendSpawnParticle`
/// See `ParticleParameters::serialize`
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleParameters {
    pub pos: v3f,
    pub vel: v3f,
    pub acc: v3f,
    pub expiration_time: f32,
    pub size: f32,
    pub base: CommonParticleParams,
    pub drag: v3f,
    pub jitter: RangedParameter<v3f>,
    pub bounce: RangedParameter<f32>,
}

// CommonParticleParams() {
//     animation.type = TAT_NONE;
//     node.setContent(CONTENT_IGNORE);
// }

impl Serialize for ParticleParameters {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        v3f::serialize(&value.pos, serializer)?;
        v3f::serialize(&value.vel, serializer)?;
        v3f::serialize(&value.acc, serializer)?;
        f32::serialize(&value.expiration_time, serializer)?;
        f32::serialize(&value.size, serializer)?;

        bool::serialize(&value.base.collision_detection, serializer)?;
        LongString::serialize(&value.base.texture.string, serializer)?;
        bool::serialize(&value.base.vertical, serializer)?;
        bool::serialize(&value.base.collision_removal, serializer)?;
        TileAnimationParams::serialize(&value.base.animation, serializer)?;
        u8::serialize(&value.base.glow, serializer)?;
        bool::serialize(&value.base.object_collision, serializer)?;
        u16::serialize(&value.base.node.param0, serializer)?;
        u8::serialize(&value.base.node.param2, serializer)?;
        u8::serialize(&value.base.node_tile, serializer)?;

        v3f::serialize(&value.drag, serializer)?;
        RangedParameter::serialize(&value.jitter, serializer)?;
        RangedParameter::serialize(&value.bounce, serializer)?;
        ServerParticleTexture::serialize_special(&value.base.texture, serializer, true, true)?;

        Ok(())
    }
}

impl Deserialize for ParticleParameters {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let pos = v3f::deserialize(deserializer).context("ParticleParameters::pos")?;
        let vel = v3f::deserialize(deserializer).context("ParticleParameters::vel")?;
        let acc = v3f::deserialize(deserializer).context("ParticleParameters::acc")?;
        let expiration_time =
            f32::deserialize(deserializer).context("ParticleParameters::expiration_time")?;
        let size = f32::deserialize(deserializer).context("ParticleParameters::size")?;

        let collision_detection = bool::deserialize(deserializer)?;
        let texture_string = LongString::deserialize(deserializer)?;
        let vertical = bool::deserialize(deserializer)?;
        let collision_removal = bool::deserialize(deserializer)?;
        let animation = TileAnimationParams::deserialize(deserializer)?;
        let glow = u8::deserialize(deserializer)?;
        let object_collision = bool::deserialize(deserializer)?;
        let param0 = u16::deserialize(deserializer)?;
        let param2 = u8::deserialize(deserializer)?;
        let node = MapNode {
            param0,
            param2,
            ..MapNode::default()
        };
        let node_tile = u8::deserialize(deserializer)?;

        let drag = v3f::deserialize(deserializer).context("ParticleParameters::drag")?;
        let jitter =
            RangedParameter::deserialize(deserializer).context("ParticleParameters::jitter")?;
        let bounce =
            RangedParameter::deserialize(deserializer).context("ParticleParameters::bounce")?;
        let texture = ServerParticleTexture::deserialize_special(
            deserializer,
            texture_string.clone(),
            true,
            true,
        )
        .context("ParticleParameters::texture")?;

        let base = CommonParticleParams {
            collision_detection,
            vertical,
            collision_removal,
            animation,
            glow,
            object_collision,
            node,
            node_tile,
            texture,
        };

        Ok(Self {
            pos,
            vel,
            acc,
            expiration_time,
            size,
            base,
            drag,
            jitter,
            bounce,
        })
    }
}
