//! An incomplete parser for the Tiled Map Editor format.
//! We only define the bare-minimum fields needed to parse the level files we're trying
//! to read.
//!
//! Works for Tiled Editor 1.1.6 - https://www.mapeditor.org/

use std::{
    io,
    fs::File,
    path::Path,
    collections::HashMap,
};

use serde_json;

#[derive(Debug, Fail)]
pub enum ReadLevelError {
    #[fail(display = "failed to deserialize level file")]
    SerdeError(#[cause] serde_json::error::Error),
    #[fail(display = "IO error occurred while reading level file")]
    IOError(#[cause] io::Error)
}

impl From<serde_json::error::Error> for ReadLevelError {
    fn from(err: serde_json::error::Error) -> Self {
        ReadLevelError::SerdeError(err)
    }
}

impl From<io::Error> for ReadLevelError {
    fn from(err: io::Error) -> Self {
        ReadLevelError::IOError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    width: u32,
    height: u32,
    infinite: bool,
    layers: Vec<Layer>,
    orientation: Orientation,
    #[serde(rename = "renderorder")]
    render_order: RenderOrder,
    #[serde(rename = "tiledversion")]
    tiled_version: String,
    #[serde(rename = "tilewidth")]
    tile_width: u32,
    #[serde(rename = "tileheight")]
    tile_height: u32,
    #[serde(rename = "tilesets")]
    tile_sets: Vec<TileSet>,
    #[serde(rename = "type")]
    type_: String,
    version: u32,
    #[serde(rename = "nextobjectid")]
    next_object_id: i32,
}

impl Level {
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, ReadLevelError> {
        let file = File::open(path)?;
        let level = serde_json::from_reader(file)?;
        Ok(level)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    #[serde(rename = "orthogonal")]
    Orthogonal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderOrder {
    #[serde(rename = "right-down")]
    RightDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Layer {
    TileLayer {
        data: Vec<i32>,
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        name: String,
        opacity: f64,
        #[serde(rename = "type")]
        type_: String,
        visible: bool,
    },
    ObjectGroup {
        #[serde(rename = "draworder")]
        draw_order: DrawOrder,
        name: String,
        objects: Vec<Object>,
        #[serde(rename = "offsetx")]
        offset_x: f64,
        #[serde(rename = "offsety")]
        offset_y: f64,
        opacity: f64,
        #[serde(rename = "type")]
        type_: String,
        visible: bool,
        x: i32,
        y: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawOrder {
    #[serde(rename = "topdown")]
    TopDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
    rotation: f64,
    id: usize,
    name: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    point: bool,
    visible: bool,
}

fn is_false(x: &bool) -> bool { !x }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSet {
    columns: u32,
    #[serde(rename = "firstgid")]
    first_gid: u32,
    grid: Grid,
    margin: i32,
    name: String,
    spacing: i32,
    #[serde(rename = "tilecount")]
    tile_count: u32,
    #[serde(rename = "tilewidth")]
    tile_width: u32,
    #[serde(rename = "tileheight")]
    tile_height: u32,
    tiles: HashMap<String, Tile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    width: u32,
    height: u32,
    orientation: Orientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    image: String,
    #[serde(rename = "imagewidth")]
    image_width: u32,
    #[serde(rename = "imageheight")]
    image_height: u32,
}
