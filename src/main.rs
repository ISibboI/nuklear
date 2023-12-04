use container::WaterContainer;
use substance::water::Water;
use uom::si::{
    f64::{Mass, ThermodynamicTemperature, Volume},
    mass::kilogram,
    thermodynamic_temperature::degree_celsius,
    volume::cubic_meter,
};

pub mod container;
pub mod interpolation_table;
pub mod substance;

fn main() {
    let mut container = WaterContainer::new(
        Volume::new::<cubic_meter>(1.0),
        Water::new(
            Mass::new::<kilogram>(1.0),
            ThermodynamicTemperature::new::<degree_celsius>(20.0),
        ),
        Water::new(
            Mass::new::<kilogram>(1.0),
            ThermodynamicTemperature::new::<degree_celsius>(2000.0),
        ),
    );

    println!("Iteration  0: {container}");
    for iteration in 1..=10 {
        container.evaporate_condensate();
        println!("Iteration {iteration:2.}: {container}");
    }
}
