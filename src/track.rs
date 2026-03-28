use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawTrack")]
pub struct Track {
    pub mbid: String,
    pub name: String,
    pub url: String,
    pub artist: NamedEntity,
    pub album: NamedEntity,
    pub image: Image,
    pub streamable: bool,
    pub nowplaying: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NamedEntity {
    pub mbid: String,
    #[serde(alias = "#text")]
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Image {
    pub small: String,
    pub medium: String,
    pub large: String,
    pub extralarge: String,
}

#[derive(Deserialize)]
struct RawTrack {
    mbid: String,
    name: String,
    url: String,
    artist: NamedEntity,
    album: NamedEntity,
    image: Vec<RawImage>,
    streamable: String,
    #[serde(rename = "@attr", default)]
    attr: Option<RawAttr>,
}

#[derive(Deserialize)]
struct RawImage {
    size: String,
    #[serde(rename = "#text")]
    url: String,
}

#[derive(Deserialize)]
struct RawAttr {
    nowplaying: Option<String>,
}

impl From<RawTrack> for Track {
    fn from(raw: RawTrack) -> Self {
        let mut image = Image::default();
        for ri in raw.image {
            match ri.size.as_str() {
                "small" => image.small = ri.url,
                "medium" => image.medium = ri.url,
                "large" => image.large = ri.url,
                "extralarge" => image.extralarge = ri.url,
                _ => {}
            }
        }

        Track {
            mbid: raw.mbid,
            name: raw.name,
            url: raw.url,
            artist: raw.artist,
            album: raw.album,
            image,
            streamable: raw.streamable != "0",
            nowplaying: raw.attr.and_then(|a| a.nowplaying).as_deref() == Some("true"),
        }
    }
}
