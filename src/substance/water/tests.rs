use uom::si::{
    f64::{Mass, ThermodynamicTemperature},
    mass::kilogram,
    thermodynamic_temperature::kelvin,
};

use super::Water;

#[test]
fn add() {
    let water1 = Water::new(
        Mass::new::<kilogram>(1.0),
        ThermodynamicTemperature::new::<kelvin>(400.0),
    );

    let water2 = Water::new(
        Mass::new::<kilogram>(2.0),
        ThermodynamicTemperature::new::<kelvin>(100.0),
    );

    let water_sum = water1 + water2;
    assert!((water_sum.mass().get::<kilogram>() - 3.0).abs() < 1e-10);
    assert!((water_sum.temperature().get::<kelvin>() - 200.0).abs() < 1e-10);
}
