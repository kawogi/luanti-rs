use crate::{
    types::{Array16, SColor, SkyColor},
    wire::{
        deser::{Deserialize, DeserializeResult, Deserializer},
        ser::{Serialize, SerializeResult, Serializer},
    },
};
use anyhow::bail;
use luanti_protocol_derive::{LuantiDeserialize, LuantiSerialize};

#[derive(Debug, Clone, PartialEq, LuantiSerialize, LuantiDeserialize)]
pub struct SetSkyCommand {
    pub params: SkyboxParams,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkyboxParams {
    pub bgcolor: SColor,
    pub r#type: String,
    pub clouds: bool,
    pub fog_sun_tint: SColor,
    pub fog_moon_tint: SColor,
    pub fog_tint_type: String,
    pub data: SkyboxData,
    pub body_orbit_tilt: f32,
    pub fog_distance: i16,
    pub fog_start: f32,
    pub fog_color: SColor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkyboxData {
    /// If `skybox_type == "plain"`
    None,
    /// If `skybox_type == "skybox"`
    Textures(Vec<String>),
    /// If `skybox_type == "regular"`
    Color(SkyColor),
}

impl SkyboxData {
    fn as_str(&self) -> &'static str {
        match self {
            SkyboxData::None => "plain",
            SkyboxData::Textures(..) => "skybox",
            SkyboxData::Color(..) => "regular",
        }
    }
}

impl Serialize for SkyboxParams {
    type Input = Self;
    fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
        SColor::serialize(&value.bgcolor, ser)?;
        str::serialize(value.data.as_str(), ser)?;
        bool::serialize(&value.clouds, ser)?;
        SColor::serialize(&value.fog_sun_tint, ser)?;
        SColor::serialize(&value.fog_moon_tint, ser)?;
        String::serialize(&value.fog_tint_type, ser)?;
        match &value.data {
            SkyboxData::None => (),
            SkyboxData::Textures(value) => <Array16<String> as Serialize>::serialize(value, ser)?,
            SkyboxData::Color(value) => SkyColor::serialize(value, ser)?,
        }

        f32::serialize(&value.body_orbit_tilt, ser)?;
        i16::serialize(&value.fog_distance, ser)?;
        f32::serialize(&value.fog_start, ser)?;
        SColor::serialize(&value.fog_color, ser)?;

        Ok(())
    }
}

impl Deserialize for SkyboxParams {
    type Output = Self;
    fn deserialize(deser: &mut Deserializer<'_>) -> DeserializeResult<Self> {
        let bgcolor = SColor::deserialize(deser)?;
        let typ = String::deserialize(deser)?;
        let clouds = bool::deserialize(deser)?;
        let fog_sun_tint = SColor::deserialize(deser)?;
        let fog_moon_tint = SColor::deserialize(deser)?;
        let fog_tint_type = String::deserialize(deser)?;
        let data = match typ.as_str() {
            "skybox" => SkyboxData::Textures(<Array16<String> as Deserialize>::deserialize(deser)?),
            "regular" => SkyboxData::Color(SkyColor::deserialize(deser)?),
            "plain" => SkyboxData::None,
            invalid => {
                bail!("Invalid skybox type: {invalid}")
            }
        };
        let body_orbit_tilt = f32::deserialize(deser)?;
        let fog_distance = i16::deserialize(deser)?;
        let fog_start = f32::deserialize(deser)?;
        let fog_color = SColor::deserialize(deser)?;
        Ok(SkyboxParams {
            bgcolor,
            r#type: typ,
            clouds,
            fog_sun_tint,
            fog_moon_tint,
            fog_tint_type,
            data,
            body_orbit_tilt,
            fog_distance,
            fog_start,
            fog_color,
        })
    }
}
