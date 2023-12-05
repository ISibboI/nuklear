use container::WaterContainer;
use substance::water::Water;
use uom::si::{
    area::square_meter,
    f64::{Area, Mass, ThermodynamicTemperature, Volume, Time},
    mass::{kilogram, megagram},
    thermodynamic_temperature::degree_celsius,
    volume::cubic_meter, time::second,
};

pub mod container;
pub mod error;
pub mod interpolation_table;
pub mod substance;

fn main() {
    // Rectangular container of dimensions 20x2x2
    let mut container = WaterContainer::new(
        Volume::new::<cubic_meter>(100.0),
        Area::new::<square_meter>(40.0),
        Water::new(
            Mass::new::<megagram>(49.85),
            ThermodynamicTemperature::new::<degree_celsius>(360.0),
        ),
        Water::new(
            Mass::new::<kilogram>(275.0),
            ThermodynamicTemperature::new::<degree_celsius>(350.0),
        ),
    );

    println!("Iteration  0: {container}");
    for iteration in 1..=10 {
        container.evaporate_condensate();
        container.convect(Time::new::<second>(0.1));
        println!("Iteration {iteration:2.}: {container}");
    }
}
