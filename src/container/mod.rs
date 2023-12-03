use uom::si::f64::Volume;

use crate::substance::water::Water;

pub struct Container {
    volume: Volume,
    water: Water,
}

impl Container {
    pub fn new(volume: Volume) -> Self {
        Self {
            volume,
            water: Water::zero(),
        }
    }
}
