use crate::types::{Array16, LongString, MapNode, RangedParameter, TileAnimationParams, v2f};
use crate::{
    types::v3f,
    wire::{
        deser::{Deserialize, DeserializeResult, Deserializer},
        ser::{Serialize, SerializeResult, Serializer},
    },
};
use anyhow::{Context, bail};
use log::info;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

#[derive(Debug, Clone, PartialEq)]
pub struct AddParticlespawnerCommand {
    /// from base class
    pub base: CommonParticleParams,

    /// default: 1
    pub amount: u16,
    /// default: 1.0
    pub time: f32,

    pub texpool: Vec<ServerParticleTexture>,

    pub pos: TweenedParameter<RangedParameter<v3f>>,
    pub vel: TweenedParameter<RangedParameter<v3f>>,
    pub acc: TweenedParameter<RangedParameter<v3f>>,
    pub drag: TweenedParameter<RangedParameter<v3f>>,
    pub radius: TweenedParameter<RangedParameter<v3f>>,
    pub jitter: TweenedParameter<RangedParameter<v3f>>,

    pub attractor: Attractor,
    pub exptime: TweenedParameter<RangedParameter<f32>>,
    pub size: TweenedParameter<RangedParameter<f32>>,
    pub bounce: TweenedParameter<RangedParameter<f32>>,

    server_id: u32,
    attached_id: u16,
}

impl Deserialize for AddParticlespawnerCommand {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        let amount = u16::deserialize(deserializer).context("amount")?;
        let time = f32::deserialize(deserializer).context("time")?;
        if time.is_sign_negative() {
            bail!("particle spawner time may not be negative");
        }

        // All tweenable parameters
        let pos = TweenedParameter::deserialize(deserializer)?;
        let vel = TweenedParameter::deserialize(deserializer)?;
        let acc = TweenedParameter::deserialize(deserializer)?;
        let exptime = TweenedParameter::deserialize(deserializer)?;
        let size = TweenedParameter::deserialize(deserializer)?;

        let collision_detection = bool::deserialize(deserializer)?;
        let texture_string = LongString::deserialize(deserializer)?;
        let server_id = u32::deserialize(deserializer)?;
        let vertical = bool::deserialize(deserializer)?;
        let collision_removal = bool::deserialize(deserializer)?;
        let attached_id = u16::deserialize(deserializer)?;
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

        let texture = ServerParticleTexture::deserialize_special(
            deserializer,
            texture_string.clone(),
            true,
            false,
        )?;

        let drag = TweenedParameter::deserialize(deserializer)?;
        let jitter = TweenedParameter::deserialize(deserializer)?;
        let bounce = TweenedParameter::deserialize(deserializer)?;
        let attractor = Attractor::deserialize(deserializer)?;
        let radius = TweenedParameter::deserialize(deserializer)?;
        let texpool = Array16::<ServerParticleTexture>::deserialize(deserializer)?;

        // TODO(kawogi) this was part of the original deserialization code but it looks like it's just triggering a side-effect which should be the job of the caller
        // auto event = new ClientEvent();
        // event->type                            = CE_ADD_PARTICLESPAWNER;
        // event->add_particlespawner.p           = new ParticleSpawnerParameters(p);
        // event->add_particlespawner.attached_id = attached_id;
        // event->add_particlespawner.id          = server_id;
        // m_client_event_queue.push(event);

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
            base,
            amount,
            time,
            texpool,
            pos,
            vel,
            acc,
            drag,
            radius,
            jitter,
            attractor,
            exptime,
            size,
            bounce,
            server_id,
            attached_id,
        })
    }
}

impl Serialize for AddParticlespawnerCommand {
    type Input = Self;

    fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
        // TODO(kawogi) this surely doesn't look like something that belongs into the serializer
        // static thread_local const float radius =
        // g_settings->getS16("max_block_send_distance") * MAP_BLOCKSIZE * BS;

        // if (peer_id == PEER_ID_INEXISTENT) {
        //     std::vector<session_t> clients = m_clients.getClientIDs();
        //     const v3f pos = (
        //         p.pos.start.min.val +
        //         p.pos.start.max.val +
        //         p.pos.end.min.val +
        //         p.pos.end.max.val
        //     ) / 4.0f * BS;
        //     const float radius_sq = radius * radius;
        //     /* Don't send short-lived spawners to distant players.
        //     * This could be replaced with proper tracking at some point.
        //     * A lifetime of 0 means that the spawner exists forever.*/
        //     const bool distance_check = !attached_id && p.time <= 1.0f && p.time != 0.0f;

        //     for (const session_t client_id : clients) {
        //         RemotePlayer *player = m_env->getPlayer(client_id);
        //         if (!player)
        //             continue;

        //         if (distance_check) {
        //             PlayerSAO *sao = player->getPlayerSAO();
        //             if (!sao)
        //                 continue;
        //             if (sao->getBasePosition().getDistanceFromSQ(pos) > radius_sq)
        //                 continue;
        //         }

        //         SendAddParticleSpawner(client_id, player->protocol_version,
        //             p, attached_id, id);
        //     }
        //     return;
        // }
        // assert(protocol_version != 0);

        // NetworkPacket pkt(TOCLIENT_ADD_PARTICLESPAWNER, 100, peer_id);

        u16::serialize(&value.amount, serializer)?;
        f32::serialize(&value.time, serializer)?;

        // Serialize entire thing
        TweenedParameter::serialize(&value.pos, serializer)?;
        TweenedParameter::serialize(&value.vel, serializer)?;
        TweenedParameter::serialize(&value.acc, serializer)?;
        TweenedParameter::serialize(&value.exptime, serializer)?;
        TweenedParameter::serialize(&value.size, serializer)?;

        bool::serialize(&value.base.collision_detection, serializer)?;
        LongString::serialize(&value.base.texture.string, serializer)?;
        u32::serialize(&value.server_id, serializer)?;
        bool::serialize(&value.base.vertical, serializer)?;
        bool::serialize(&value.base.collision_removal, serializer)?;
        u16::serialize(&value.attached_id, serializer)?;
        TileAnimationParams::serialize(&value.base.animation, serializer)?;
        u8::serialize(&value.base.glow, serializer)?;
        bool::serialize(&value.base.object_collision, serializer)?;
        u16::serialize(&value.base.node.param0, serializer)?;
        u8::serialize(&value.base.node.param2, serializer)?;
        u8::serialize(&value.base.node_tile, serializer)?;

        ServerParticleTexture::serialize_special(&value.base.texture, serializer, true, false)?;

        // new properties
        TweenedParameter::serialize(&value.drag, serializer)?;
        TweenedParameter::serialize(&value.jitter, serializer)?;
        TweenedParameter::serialize(&value.bounce, serializer)?;
        Attractor::serialize(&value.attractor, serializer)?;
        TweenedParameter::serialize(&value.radius, serializer)?;
        Array16::<ServerParticleTexture>::serialize(&value.texpool, serializer)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[expect(clippy::struct_excessive_bools, reason = "this is mandated by the API")]
pub struct CommonParticleParams {
    pub(crate) collision_detection: bool,
    pub(crate) vertical: bool,
    pub(crate) collision_removal: bool,
    pub(crate) animation: TileAnimationParams,
    pub(crate) glow: u8,
    pub(crate) object_collision: bool,
    pub(crate) node: MapNode,
    pub(crate) node_tile: u8,
    pub(crate) texture: ServerParticleTexture,
}

// impl Serialize for CommonParticleParams {
//     type Input = Self;

//     fn serialize<S: Serializer>(value: &Self::Input, serializer: &mut S) -> SerializeResult {
//         bool::serialize(&value.collision_detection, serializer)?;
//         LongString::serialize(&value.texture_string, serializer)?;
//         u32::serialize(&value.server_id, serializer)?;
//         bool::serialize(&value.vertical, serializer)?;
//         bool::serialize(&value.collision_removal, serializer)?;
//         u16::serialize(&value.attached_id, serializer)?;
//         TileAnimationParams::serialize(&value.animation, serializer)?;
//         u8::serialize(&value.glow, serializer)?;
//         bool::serialize(&value.object_collision, serializer)?;
//         u16::serialize(&value.node.param0, serializer)?;
//         u8::serialize(&value.node.param2, serializer)?;
//         u8::serialize(&value.node_tile, serializer)?;
//         // ServerParticleTexture::serialize(&value.texture, serializer)?; // properties for legacy texture field
//         Ok(())
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub enum Attractor {
    None,
    Point(PointAttractor),
    Line(LineAttractor),
    Plane(PlaneAttractor),
}

impl Serialize for Attractor {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        let kind: u8 = match value {
            Attractor::None => 0,
            Attractor::Point(_) => 1,
            Attractor::Line(_) => 2,
            Attractor::Plane(_) => 3,
        };
        u8::serialize(&kind, ser)?;
        match value {
            Attractor::None => (),
            Attractor::Point(value) => PointAttractor::serialize(value, ser)?,
            Attractor::Line(value) => LineAttractor::serialize(value, ser)?,
            Attractor::Plane(value) => PlaneAttractor::serialize(value, ser)?,
        }
        Ok(())
    }
}

impl Deserialize for Attractor {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let kind = u8::deserialize(deser)?;
        Ok(match kind {
            0 => Attractor::None,
            1 => Attractor::Point(PointAttractor::deserialize(deser)?),
            2 => Attractor::Line(LineAttractor::deserialize(deser)?),
            3 => Attractor::Plane(PlaneAttractor::deserialize(deser)?),
            _ => bail!("Invalid AttractorKind: {}", kind),
        })
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PointAttractor {
    pub attract: TweenedParameter<RangedParameter<f32>>,
    pub origin: TweenedParameter<v3f>,
    pub attachment: u16,
    pub kill: u8,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct LineAttractor {
    pub attract: TweenedParameter<RangedParameter<f32>>,
    pub origin: TweenedParameter<v3f>,
    pub attachment: u16,
    pub kill: u8,
    pub direction: TweenedParameter<v3f>,
    pub direction_attachment: u16,
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct PlaneAttractor {
    pub attract: TweenedParameter<RangedParameter<f32>>,
    pub origin: TweenedParameter<v3f>,
    pub attachment: u16,
    pub kill: u8,
    pub direction: TweenedParameter<v3f>,
    pub direction_attachment: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleTexture {
    pub blend_mode: BlendMode,
    pub alpha: TweenedParameter<f32>,
    pub scale: TweenedParameter<v2f>,
    pub animation: Option<TileAnimationParams>,
}

impl Default for ParticleTexture {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::Alpha,
            alpha: TweenedParameter::new_simple(1.0),
            scale: TweenedParameter::new_simple(v2f { x: 1.0, y: 1.0 }),
            animation: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerParticleTexture {
    // inherited from base class
    pub base: ParticleTexture,
    pub string: String, // LongString
}

impl Serialize for ServerParticleTexture {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        Self::serialize_special(value, ser, false, false)
    }
}

impl Deserialize for ServerParticleTexture {
    type Output = Self;

    fn deserialize(deserializer: &mut Deserializer<'_>) -> DeserializeResult<Self::Output> {
        Self::deserialize_special(deserializer, String::new(), false, false)
    }
}

impl ServerParticleTexture {
    pub(crate) fn serialize_special<S: Serializer>(
        value: &Self,
        ser: &mut S,
        new_properties_only: bool,
        skip_animation: bool,
    ) -> SerializeResult {
        let animated = value.base.animation.is_some();
        let flags = (value.base.blend_mode.to_u8() << 1) | u8::from(animated);
        u8::serialize(&flags, ser)?;

        <TweenedParameter<f32>>::serialize(&value.base.alpha, ser)?;
        <TweenedParameter<v2f>>::serialize(&value.base.scale, ser)?;
        if !new_properties_only {
            LongString::serialize(&value.string, ser)?;
        }
        if !skip_animation {
            if let Some(animation) = value.base.animation.as_ref() {
                TileAnimationParams::serialize(animation, ser)?;
            }
        }
        Ok(())
    }

    pub(crate) fn deserialize_special(
        deserializer: &mut Deserializer<'_>,
        string: String,
        new_properties_only: bool,
        skip_animation: bool,
    ) -> DeserializeResult<Self> {
        let flags = ParticleTextureFlags::deserialize(deserializer)
            .context("ServerParticleTexture::flags")?;
        info!("flags {flags:?}");
        // new texture properties were missing in ParticleParameters::serialize before Minetest 5.9.0
        if !deserializer.has_remaining() {
            return Ok(Self {
                base: ParticleTexture::default(),
                string,
            });
        }

        let animated = flags.animated();
        info!("animated {animated:?}");
        let blend_mode = flags.blend_mode()?;
        info!("blend_mode {blend_mode:?}");

        let alpha =
            TweenedParameter::deserialize(deserializer).context("ServerParticleTexture::alpha")?;
        info!("alpha {alpha:?}");
        info!("remaining {:?}", deserializer.peek_all());
        let scale =
            TweenedParameter::deserialize(deserializer).context("ServerParticleTexture::scale")?;
        let string = if new_properties_only {
            string
        } else {
            LongString::deserialize(deserializer).context("ServerParticleTexture::string")?
        };

        let animation = (animated && !skip_animation)
            .then(|| TileAnimationParams::deserialize(deserializer))
            .transpose()
            .context("ServerParticleTexture::animation")?;

        let base = ParticleTexture {
            blend_mode,
            alpha,
            scale,
            animation,
        };

        Ok(Self { base, string })
    }
}

#[derive(Clone, Copy, Debug, LuantiDeserialize, LuantiSerialize)]
struct ParticleTextureFlags(u8);

impl ParticleTextureFlags {
    fn animated(self) -> bool {
        (self.0 & 0x0000_0001) != 0
    }
    fn blend_mode(self) -> DeserializeResult<BlendMode> {
        BlendMode::from_u8(self.0 >> 1)
    }
}

/// This is serialized as part of a combined 'flags' field on
/// `ServerParticleTexture`, so it doesn't implement the  methods
/// on its own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendMode {
    Alpha,
    Add,
    Sub,
    Screen,
}

impl BlendMode {
    fn to_u8(self) -> u8 {
        match self {
            BlendMode::Alpha => 0,
            BlendMode::Add => 1,
            BlendMode::Sub => 2,
            BlendMode::Screen => 3,
        }
    }

    fn from_u8(value: u8) -> DeserializeResult<BlendMode> {
        Ok(match value {
            0 => BlendMode::Alpha,
            1 => BlendMode::Add,
            2 => BlendMode::Sub,
            3 => BlendMode::Screen,
            _ => bail!("Invalid BlendMode u8: {}", value),
        })
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct TweenedParameter<T: Serialize + Deserialize>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    pub style: TweenStyle,
    pub reps: u16,
    pub beginning: f32,
    pub start: T,
    pub end: T,
}

impl<T: Default + Serialize + Deserialize> Default for TweenedParameter<T>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    fn default() -> Self {
        Self {
            style: TweenStyle::Fwd,
            reps: 1,
            beginning: 0.0,
            start: T::default(),
            end: T::default(),
        }
    }
}

impl<T: Clone + Serialize + Deserialize> TweenedParameter<T>
where
    T: Serialize<Input = T>,
    T: Deserialize<Output = T>,
{
    fn new_simple(value: T) -> Self {
        Self {
            style: TweenStyle::Fwd,
            reps: 1,
            beginning: 0.0,
            start: value.clone(),
            end: value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub enum TweenStyle {
    Fwd,
    Rev,
    Pulse,
    Flicker,
}
