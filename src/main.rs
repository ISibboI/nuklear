use std::env::args;

use container::WaterContainer;
use electrical_grid::ElectricalGrid;
use log::error;
use nalgebra::DMatrix;
use substance::water::Water;
use synchronous_machine::SynchronousMachine;
use uom::{
    si::{
        angle::radian,
        angular_velocity::radian_per_second,
        area::square_meter,
        electrical_conductance::siemens,
        f64::{
            Angle, AngularVelocity, Area, ElectricalConductance, MagneticFlux, Mass,
            MomentOfInertia, Power, ThermodynamicTemperature, Time, Volume,
        },
        magnetic_flux::weber,
        mass::{kilogram, megagram},
        moment_of_inertia::kilogram_square_meter,
        power::watt,
        thermodynamic_temperature::degree_celsius,
        time::second,
        volume::cubic_meter,
    },
    ConstZero,
};

pub mod container;
pub mod electrical_grid;
pub mod error;
pub mod interpolation_table;
pub mod substance;
pub mod synchronous_machine;

pub fn main() {
    match args().nth(1) {
        None => electrical_grid_sim(),
        Some(string) => match string.as_str() {
            "container_sim" => container_sim(),
            "electrical_grid_sim" => electrical_grid_sim(),
            string => error!("Simulation not found: {:?}", string),
        },
    }
}

fn electrical_grid_sim() {
    let mut electrical_grid = ElectricalGrid::new(
        vec![
            SynchronousMachine::new(
                MomentOfInertia::new::<kilogram_square_meter>(1.0),
                Angle::new::<radian>(0.0),
                AngularVelocity::new::<radian_per_second>(1.0),
                Power::ZERO,
                MagneticFlux::new::<weber>(1.0),
                Power::ZERO,
            ),
            SynchronousMachine::new(
                MomentOfInertia::new::<kilogram_square_meter>(1.0),
                Angle::new::<radian>(0.0),
                AngularVelocity::new::<radian_per_second>(1.0),
                Power::ZERO,
                MagneticFlux::new::<weber>(1.0),
                Power::ZERO,
            ),
            SynchronousMachine::new(
                MomentOfInertia::new::<kilogram_square_meter>(0.1),
                Angle::new::<radian>(0.0),
                AngularVelocity::new::<radian_per_second>(1.0),
                Power::ZERO,
                MagneticFlux::new::<weber>(1.0),
                Power::ZERO,
            ),
        ],
        DMatrix::repeat(3, 3, ElectricalConductance::new::<siemens>(1.0)),
        DMatrix::zeros(3, 3),
    );

    let mut mechanical_acceleration_powers = vec![
        Power::new::<watt>(1.0),
        Power::new::<watt>(1.0),
        Power::new::<watt>(0.0),
    ];

    println!(" == Iteration 0 ==\n{electrical_grid}\n ====");
    for iteration in 1..=200 {
        let delta_time = Time::new::<second>(0.01);
        //mechanical_acceleration_powers[2] = (-electrical_grid.synchronous_machines()[2].signed_angular_kinetic_energy() / delta_time * 0.2).max(Power::new::<watt>(-2.0));
        mechanical_acceleration_powers[2] = Power::new::<watt>(
            if electrical_grid.synchronous_machines()[2].angular_velocity()
                > AngularVelocity::new::<radian_per_second>(2.0)
            {
                -2.0
            } else {
                0.0
            },
        );
        electrical_grid.update(&mechanical_acceleration_powers, delta_time);
        println!(" == Iteration {iteration:2.} ==\n{electrical_grid}\n ====");
    }
}

fn container_sim() {
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
