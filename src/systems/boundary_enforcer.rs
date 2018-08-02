use specs::{System, Join, ReadExpect, ReadStorage, WriteStorage};

use components::{Position, BoundingBox};
use map::LevelMap;

#[derive(SystemData)]
pub struct BoundaryEnforcerData<'a> {
    map: ReadExpect<'a, LevelMap>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    positions: WriteStorage<'a, Position>,
}

/// Enforces that entities with either a position or a position and a bounding box do not leave
/// the level boundary.
pub struct BoundaryEnforcer;

impl<'a> System<'a> for BoundaryEnforcer {
    type SystemData = BoundaryEnforcerData<'a>;

    fn run(&mut self, BoundaryEnforcerData {map, bounding_boxes, mut positions}: Self::SystemData) {
        let level_boundary = map.level_boundary();

        for (pos, bounds) in (&mut positions, &bounding_boxes).join() {
            //TODO: Clamp position such that bounding box is within the map boundary.
        }

        for (pos, ()) in (&mut positions, !&bounding_boxes).join() {
            //TODO: Clamp position such that it is within the map boundary.
        }
    }
}
