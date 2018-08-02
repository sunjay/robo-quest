use std::{
    cmp,
    path::Path,
};

use sdl2::rect::{Point, Rect};

use texture_manager::{TextureManager, TextureId};
use level_file::{ReadLevelError, Level, Layer, TileId, Object};

/// A grid of tiles. Must have at least one row and one column.
#[derive(Debug, Clone)]
pub struct TileGrid(Vec<Vec<Option<Tile>>>);

impl TileGrid {
    /// Returns (rows, columns) representing the dimensions of this grid
    pub fn dimensions(&self) -> (usize, usize) {
        (self.0.len(), self.0[0].len())
    }

    pub fn slice_within(&self, tile_width: usize, tile_height: usize, bounds: Rect) -> impl Iterator<Item=&Tile> {
        // While the user is allowed to ask for tiles within a boundary that starts at negative
        // coordinates, the top left of the map is defined as (0, 0). That means that we can at
        // most request tiles up to that top left corner. The calls to `max()` here help enforce
        // that by making sure we don't convert a negative number to an unsigned type.
        let x = cmp::max(bounds.x(), 0) as usize;
        let y = cmp::max(bounds.y(), 0) as usize;
        let width = bounds.width() as usize;
        let height = bounds.height() as usize;

        let (rows, columns) = self.dimensions();
        let clamp_col = |col| cmp::min(cmp::max(col, 0), columns-1);
        let clamp_row = |row| cmp::min(cmp::max(row, 0), rows-1);

        let start_col = clamp_col(x / tile_width);
        let start_row = clamp_row(y / tile_height);
        let end_col = clamp_col((x + width) / tile_width);
        let end_row = clamp_row((y + height) / tile_height);

        self.0[start_row..=end_row].iter().flat_map(move |col| col[start_col..=end_col].iter().filter_map(|x| x.as_ref()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub texture_id: TextureId,
    pub image_width: u32,
    pub image_height: u32,
}

/// Resource that represents a map of tiles for an entire level of the game.
///
/// Top-left of the top-left tile is at position (0, 0). Each tile is of constant width and height,
/// so computing tiles within a boundary rectangle is fairly trivial.
#[derive(Debug, Clone)]
pub struct LevelMap {
    level_start: Point,
    level_boundary: Rect,
    rows: usize,
    columns: usize,
    tile_width: usize,
    tile_height: usize,
    background: TileGrid,
    background_items: TileGrid,
    map: TileGrid,
}

impl LevelMap {
    pub fn load_file<P: AsRef<Path>>(path: P, texture_manager: &mut TextureManager) -> Result<Self, ReadLevelError> {
        let path = path.as_ref();
        let level = Level::load_file(path)?;

        // We support Tiled version 1.0 compatible maps
        assert_eq!(level.version, 1);

        // All paths within the level file must be resolved relative to the directory that the
        // level file was loaded from
        let resolve_dir = path.parent().expect("Loaded level map should not be the root directory");

        let Level {
            width: columns,
            height: rows,
            tile_width,
            tile_height,
            ref tile_sets,
            ref layers,
            ..
        } = level;

        let lookup_tile = |tile_id| {
            for tile_set in tile_sets {
                let id_offset = tile_set.first_gid;
                if id_offset > tile_id {
                    continue;
                }
                let tile_id = tile_id - id_offset;
                let key = &TileId(tile_id);
                if tile_set.tiles.contains_key(key) {
                    return Some(&tile_set.tiles[key]);
                }
            }

            None
        };

        let mut background = None;
        let mut background_items = None;
        let mut map = None;
        let mut level_start = None;
        let mut level_boundary = None;

        for layer in layers {
            match layer {
                Layer::TileLayer {name, data, width: layer_width, height: layer_height, ..} => {
                    assert_eq!(columns, *layer_width);
                    assert_eq!(rows, *layer_height);

                    let mut tile_rows = Vec::with_capacity(rows as usize);

                    let mut row = -1;
                    let mut col = 0;
                    for (i, &id) in data.into_iter().enumerate() {
                        if i as u32 % columns == 0 {
                            row += 1;
                            col = 0;
                            tile_rows.push(Vec::with_capacity(columns as usize));
                        }
                        let tile = lookup_tile(id).map(|tile| {
                            //FIXME: Remove this unwrap() when we start using proper error types
                            let image_path = resolve_dir.join(&tile.image).canonicalize().unwrap();
                            //FIXME: Remove this unwrap() when we start using proper error types
                            let texture_id = texture_manager.create_png_texture(image_path).unwrap();
                            Tile {
                                x: col * tile_width as i32,
                                y: row * tile_height as i32,
                                texture_id,
                                image_width: tile.image_width,
                                image_height: tile.image_height,
                            }
                        });
                        tile_rows.last_mut().unwrap().push(tile);
                        col += 1;
                    }

                    assert_eq!(tile_rows.len(), rows as usize);
                    assert!(tile_rows.iter().all(|r| r.len() == columns as usize));

                    let tile_grid = TileGrid(tile_rows);
                    match name.as_str() {
                        "background" => background = Some(tile_grid),
                        "background_items" => background_items = Some(tile_grid),
                        "map" => map = Some(tile_grid),
                        _ => unreachable!("Unrecognized layer name: {}", name),
                    }
                },
                Layer::ObjectGroup {name, objects, ..} => {
                    assert_eq!(name, "markers");

                    for &Object {ref name, x, y, width, height, rotation, point, ..} in objects {
                        match name.as_str() {
                            "level_start" => {
                                assert!(point);
                                // Must not be rotated
                                assert!(rotation < ::std::f64::EPSILON);
                                // Point should not have any size information
                                assert!(width < ::std::f64::EPSILON);
                                assert!(height < ::std::f64::EPSILON);
                                level_start = Some(Point::new(x as i32, y as i32));
                            },
                            "level_boundary" => {
                                assert!(!point);
                                // Must not be rotated
                                assert!(rotation < ::std::f64::EPSILON);

                                level_boundary = Some(Rect::new(
                                    x as i32,
                                    y as i32,
                                    width as u32,
                                    height as u32,
                                ));
                            },
                            _ => unreachable!("Unrecognized object name in markers layer: {}", name),
                        }
                    }
                },
            }
        }

        Ok(Self {
            level_start: level_start.unwrap(),
            level_boundary: level_boundary.unwrap(),
            rows: rows as usize,
            columns: columns as usize,
            tile_width: tile_width as usize,
            tile_height: tile_height as usize,
            background: background.unwrap(),
            background_items: background_items.unwrap(),
            map: map.unwrap(),
        })
    }

    /// The width of each tile in pixels
    pub fn tile_width(&self) -> usize {
        self.tile_width
    }

    /// The height of each tile in pixels
    pub fn tile_height(&self) -> usize {
        self.tile_height
    }

    pub fn level_start(&self) -> Point {
        self.level_start
    }

    pub fn level_boundary(&self) -> Rect {
        self.level_boundary
    }

    pub fn background_within(&self, bounds: Rect) -> impl Iterator<Item=&Tile> {
        self.background.slice_within(self.tile_width, self.tile_height, bounds)
    }

    pub fn background_items_within(&self, bounds: Rect) -> impl Iterator<Item=&Tile> {
        self.background_items.slice_within(self.tile_width, self.tile_height, bounds)
    }

    pub fn map_within(&self, bounds: Rect) -> impl Iterator<Item=&Tile> {
        self.map.slice_within(self.tile_width, self.tile_height, bounds)
    }
}
