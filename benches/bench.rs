use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nuklear::{container::WaterContainer, substance::water::Water};
use uom::si::{
    area::square_meter,
    f64::{Area, Mass, ThermodynamicTemperature, Time, Volume},
    mass::{kilogram, megagram},
    thermodynamic_temperature::degree_celsius,
    time::second,
    volume::cubic_meter,
};

fn create_container() -> WaterContainer {
    // Rectangular container of dimensions 20x2x2
    WaterContainer::new(
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
    )
}

fn update_container(c: &mut Criterion) {
    let mut container = create_container();

    c.bench_function("update water container", |b| {
        b.iter(|| {
            container.convect(Time::new::<second>(black_box(0.1)));
            container.evaporate_condensate();
        })
    });
}

criterion_group!(benches, update_container);
criterion_main!(benches);
