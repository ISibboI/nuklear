use std::fmt::Display;

use nalgebra::DMatrix;
use uom::{
    si::f64::{ElectricCurrent, ElectricalConductance, Power, Time},
    ConstZero,
};

use crate::synchronous_machine::SynchronousMachine;

pub mod resistance_network;

/*pub struct ElectricalGrid {
    synchronous_machines: Vec<SynchronousMachine>,
    // admittance_matrix: DMatrix<ElectricalAdmittance>,
    conductance_matrix: DMatrix<ElectricalConductance>,
    susceptance_matrix: DMatrix<ElectricalSusceptance>,
}

impl ElectricalGrid {
    /// Matrices should be symmetric.
    pub fn new(
        synchronous_machines: Vec<SynchronousMachine>,
        conductance_matrix: DMatrix<ElectricalConductance>,
        susceptance_matrix: DMatrix<ElectricalSusceptance>,
    ) -> Self {
        assert_eq!(
            conductance_matrix.row_iter().count(),
            synchronous_machines.len()
        );
        assert_eq!(
            conductance_matrix.column_iter().count(),
            synchronous_machines.len()
        );
        assert_eq!(
            susceptance_matrix.row_iter().count(),
            synchronous_machines.len()
        );
        assert_eq!(
            susceptance_matrix.column_iter().count(),
            synchronous_machines.len()
        );

        Self {
            synchronous_machines,
            conductance_matrix,
            susceptance_matrix,
        }
    }

    pub fn synchronous_machines(&self) -> &[SynchronousMachine] {
        &self.synchronous_machines
    }

    pub fn conductance_matrix(&self) -> &DMatrix<ElectricalConductance> {
        &self.conductance_matrix
    }

    pub fn susceptance_matrix(&self) -> &DMatrix<ElectricalSusceptance> {
        &self.susceptance_matrix
    }

    pub fn update(&mut self, mechanical_acceleration_powers: &[Power], delta_time: Time) {
        let internal_voltages: Vec<_> = self
            .synchronous_machines
            .iter()
            .map(SynchronousMachine::internal_voltages_per_phase)
            .collect();

        let electrical_deceleration_powers: Vec<_> = (0..self.synchronous_machines.len())
            .map(|i| {
                let internal_voltage_per_phase = internal_voltages[i];
                let current_per_phase = (0..self.synchronous_machines.len())
                    .map(|j| {
                        let other_internal_voltage_per_phase = internal_voltages[j];
                        let conductance = self.conductance_matrix[(i, j)];
                        let susceptance = self.susceptance_matrix[(i, j)];
                        let delta_phase = self.synchronous_machines[i].angular_position()
                            - self.synchronous_machines[j].angular_position();

                        //let admittance =
                        //    susceptance * delta_phase.cos() - conductance * delta_phase.sin();
                        let admittance =
                            susceptance * delta_phase.cos() - conductance * delta_phase.sin();
                        [
                            other_internal_voltage_per_phase[0] * admittance,
                            other_internal_voltage_per_phase[1] * admittance,
                            other_internal_voltage_per_phase[2] * admittance,
                        ]
                    })
                    .fold([ElectricCurrent::ZERO; 3], |mut accumulator, item| {
                        accumulator[0] += item[0];
                        accumulator[1] += item[1];
                        accumulator[2] += item[2];
                        accumulator
                    });

                let power_per_phase = [
                    internal_voltage_per_phase[0] * current_per_phase[0],
                    internal_voltage_per_phase[1] * current_per_phase[1],
                    internal_voltage_per_phase[2] * current_per_phase[2],
                ];
                power_per_phase[0] + power_per_phase[1] + power_per_phase[2]
            })
            .collect();

        for i in 0..self.synchronous_machines.len() {
            self.synchronous_machines[i].update(
                mechanical_acceleration_powers[i],
                // there is a sign error somewhere... with this, a lagging phase will result in acceleration
                -electrical_deceleration_powers[i],
                delta_time,
            )
        }
    }
}

impl Display for ElectricalGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut once = true;
        for synchronous_machine in &self.synchronous_machines {
            if once {
                once = false;
            } else {
                writeln!(f)?;
            }

            write!(f, "{}", synchronous_machine)?;
        }

        Ok(())
    }
}
*/
