use anyhow::bail;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

use crate::{
    types::{Array16, SColor, SkyColor},
    wire::{
        deser::{Deserialize, DeserializeResult, Deserializer},
        ser::{Serialize, SerializeResult, Serializer},
    },
};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetSkyCommand {
    pub params: SkyboxParams,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkyboxParams {
    pub bgcolor: SColor,
    pub clouds: bool,
    pub fog_sun_tint: SColor,
    pub fog_moon_tint: SColor,
    pub fog_tint_type: String,
    pub data: SkyboxData,
    pub body_orbit_tilt: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkyboxData {
    None,                  // If skybox_type == "plain"
    Textures(Vec<String>), // If skybox_type == "skybox"
    Color(SkyColor),       // If skybox_type == "regular"
}

impl Serialize for SkyboxParams {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        SColor::serialize(&value.bgcolor, ser)?;
        let skybox_type = match &value.data {
            SkyboxData::None => "plain",
            SkyboxData::Textures(..) => "skybox",
            SkyboxData::Color(..) => "regular",
        };
        str::serialize(skybox_type, ser)?;
        bool::serialize(&value.clouds, ser)?;
        SColor::serialize(&value.fog_sun_tint, ser)?;
        SColor::serialize(&value.fog_moon_tint, ser)?;
        String::serialize(&value.fog_tint_type, ser)?;
        match &value.data {
            SkyboxData::None => (),
            SkyboxData::Textures(value) => <Array16<String> as Serialize>::serialize(value, ser)?,
            SkyboxData::Color(value) => SkyColor::serialize(value, ser)?,
        }
        <Option<f32> as Serialize>::serialize(&value.body_orbit_tilt, ser)?;
        Ok(())
    }
}

impl Deserialize for SkyboxParams {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let bgcolor = SColor::deserialize(deser)?;
        let typ = String::deserialize(deser)?;
        Ok(SkyboxParams {
            bgcolor,
            clouds: bool::deserialize(deser)?,
            fog_sun_tint: SColor::deserialize(deser)?,
            fog_moon_tint: SColor::deserialize(deser)?,
            fog_tint_type: String::deserialize(deser)?,
            data: {
                if typ == "skybox" {
                    SkyboxData::Textures(<Array16<String> as Deserialize>::deserialize(deser)?)
                } else if typ == "regular" {
                    SkyboxData::Color(SkyColor::deserialize(deser)?)
                } else if typ == "plain" {
                    SkyboxData::None
                } else {
                    bail!("Invalid skybox type: {:?}", typ)
                }
            },
            body_orbit_tilt: <Option<f32> as Deserialize>::deserialize(deser)?,
        })
    }
}
