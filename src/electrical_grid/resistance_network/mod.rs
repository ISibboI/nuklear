use nalgebra::DMatrix;
use petgraph::{graph::UnGraph, visit::IntoNodeReferences};
use uom::si::{
    electric_potential::volt,
    electrical_conductance::siemens,
    electrical_resistance::ohm,
    f64::{ElectricPotential, ElectricalConductance, ElectricalResistance},
};

use crate::error::Error;

#[derive(Debug)]
pub struct ResistanceNetwork {
    graph: UnGraph<ElectricalNode, ElectricalResistance, usize>,
    unknown_voltage_indexes_to_node_indexes: Vec<usize>,
    node_indexes_to_unknown_voltage_indexes: Vec<usize>,
}

#[derive(Debug)]
pub enum ElectricalNode {
    VoltageSource {
        voltage: ElectricPotential,
    },
    Branch {
        voltage: ElectricPotential,
    },
    ResistanceConsumer {
        resistance: ElectricalResistance,
        voltage: ElectricPotential,
    },
}

impl ResistanceNetwork {
    pub fn new(graph: UnGraph<ElectricalNode, ElectricalResistance, usize>) -> Self {
        let unknown_voltage_indexes_to_node_indexes: Vec<_> = graph
            .node_references()
            .filter_map(|(index, node)| {
                if matches!(
                    node,
                    ElectricalNode::Branch { .. } | ElectricalNode::ResistanceConsumer { .. }
                ) {
                    Some(index.index())
                } else {
                    None
                }
            })
            .collect();

        let mut node_indexes_to_unknown_voltage_indexes = Vec::with_capacity(graph.node_count());
        for (to, from) in unknown_voltage_indexes_to_node_indexes
            .iter()
            .copied()
            .enumerate()
        {
            while node_indexes_to_unknown_voltage_indexes.len() < from {
                node_indexes_to_unknown_voltage_indexes.push(usize::MAX);
            }
            node_indexes_to_unknown_voltage_indexes.push(to);
        }
        while node_indexes_to_unknown_voltage_indexes.len() < graph.node_count() {
            node_indexes_to_unknown_voltage_indexes.push(usize::MAX);
        }

        dbg!(Self {
            graph,
            unknown_voltage_indexes_to_node_indexes,
            node_indexes_to_unknown_voltage_indexes,
        })
    }

    pub fn graph(&self) -> &UnGraph<ElectricalNode, ElectricalResistance, usize> {
        &self.graph
    }

    fn build_voltage_linear_equation_system(&self) -> (DMatrix<f64>, DMatrix<f64>) {
        let mut conductance_matrix = DMatrix::zeros(
            self.unknown_voltage_indexes_to_node_indexes.len(),
            self.unknown_voltage_indexes_to_node_indexes.len(),
        );
        let mut current_vector =
            DMatrix::zeros(self.unknown_voltage_indexes_to_node_indexes.len(), 1);

        for node_index in self.graph.node_indices() {
            let voltage_index = self.node_indexes_to_unknown_voltage_indexes[node_index.index()];
            if voltage_index == usize::MAX {
                // The voltage of this node is fixed.
                continue;
            }

            let node = self.graph.node_weight(node_index).unwrap();
            if let ElectricalNode::ResistanceConsumer { resistance, .. } = node {
                let conductance = 1.0 / resistance.get::<ohm>();
                conductance_matrix[(voltage_index, voltage_index)] += conductance;
                // The consumer is always connected to ground at voltage zero, hence the update to the current vector is zero.
            }

            for neighbor_index in self.graph.neighbors(node_index) {
                let conductance = self
                    .graph
                    .edges_connecting(node_index, neighbor_index)
                    .map(|edge| 1.0 / *edge.weight())
                    .sum::<ElectricalConductance>()
                    .get::<siemens>();
                let neighbor_voltage_index =
                    self.node_indexes_to_unknown_voltage_indexes[neighbor_index.index()];

                conductance_matrix[(voltage_index, voltage_index)] += conductance;
                if neighbor_voltage_index != usize::MAX {
                    conductance_matrix[(voltage_index, neighbor_voltage_index)] -= conductance;
                } else {
                    let ElectricalNode::VoltageSource {
                        voltage: neighbor_voltage,
                    } = self.graph.node_weight(neighbor_index).unwrap()
                    else {
                        unreachable!(
                            "only voltage sources have a given voltage\nnode: {node_index:?}, neighbor_node: {neighbor_index:?}"
                        );
                    };
                    current_vector[(voltage_index, 0)] +=
                        neighbor_voltage.get::<volt>() * conductance;
                }
            }
        }

        (conductance_matrix, current_vector)
    }

    pub fn update_voltages(&mut self) -> crate::error::Result<()> {
        if self.unknown_voltage_indexes_to_node_indexes.is_empty() {
            // No unknown voltages to compute.
            return Ok(());
        }

        let (conductance_matrix, current_vector) = self.build_voltage_linear_equation_system();
        let lu_decomposition = conductance_matrix.lu();
        let voltages = lu_decomposition.solve(&current_vector).ok_or(
            Error::NonInvertibleConductanceMatrix {
                matrix: self.build_voltage_linear_equation_system().0,
            },
        )?;

        for (unknown_voltage_index, computed_voltage) in voltages.iter().copied().enumerate() {
            let node_index = self.unknown_voltage_indexes_to_node_indexes[unknown_voltage_index];
            match self.graph.node_weight_mut(node_index.into()).unwrap() {
                ElectricalNode::VoltageSource { .. } => {
                    unreachable!("a voltage source has a known voltage");
                }
                ElectricalNode::Branch { voltage }
                | ElectricalNode::ResistanceConsumer { voltage, .. } => {
                    *voltage = ElectricPotential::new::<volt>(computed_voltage);
                }
            }
        }

        Ok(())
    }
}

impl ElectricalNode {
    fn voltage(&self) -> ElectricPotential {
        match self {
            ElectricalNode::VoltageSource { voltage }
            | ElectricalNode::Branch { voltage }
            | ElectricalNode::ResistanceConsumer { voltage, .. } => *voltage,
        }
    }
}

#[cfg(test)]
mod tests {
    use petgraph::graph::UnGraph;
    use uom::{
        si::{
            electric_potential::volt,
            electrical_resistance::ohm,
            f64::{ElectricPotential, ElectricalResistance},
        },
        ConstZero,
    };

    use super::{ElectricalNode, ResistanceNetwork};

    #[test]
    fn linear() {
        let mut graph = UnGraph::default();
        let n = [
            graph.add_node(ElectricalNode::VoltageSource {
                voltage: ElectricPotential::new::<volt>(1.0),
            }),
            graph.add_node(ElectricalNode::ResistanceConsumer {
                resistance: ElectricalResistance::new::<ohm>(1.0),
                voltage: ElectricPotential::ZERO,
            }),
            graph.add_node(ElectricalNode::ResistanceConsumer {
                resistance: ElectricalResistance::new::<ohm>(1.0),
                voltage: ElectricPotential::ZERO,
            }),
        ];
        graph.add_edge(n[0], n[1], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[1], n[2], ElectricalResistance::new::<ohm>(1.0));

        let mut resistance_network = ResistanceNetwork::new(graph);
        resistance_network.update_voltages().unwrap();

        let voltages: Vec<_> = resistance_network
            .graph()
            .node_weights()
            .map(ElectricalNode::voltage)
            .collect();
        let expected_voltages: Vec<_> = [1.0, 0.4, 0.2]
            .into_iter()
            .map(ElectricPotential::new::<volt>)
            .collect();

        for (i, (voltage, expected_voltage)) in voltages
            .iter()
            .copied()
            .zip(expected_voltages.iter().copied())
            .enumerate()
        {
            assert!(
                (voltage - expected_voltage).abs() < ElectricPotential::new::<volt>(1e-10),
                "Voltages at index {i} differ. Expected: {:.2}. Actual: {:.2}.",
                expected_voltage.into_format_args(volt, uom::fmt::DisplayStyle::Abbreviation),
                voltage.into_format_args(volt, uom::fmt::DisplayStyle::Abbreviation)
            );
        }
    }

    #[test]
    fn star() {
        let mut graph = UnGraph::default();
        let n = [
            graph.add_node(ElectricalNode::VoltageSource {
                voltage: ElectricPotential::new::<volt>(1.0),
            }),
            graph.add_node(ElectricalNode::VoltageSource {
                voltage: ElectricPotential::new::<volt>(2.0),
            }),
            graph.add_node(ElectricalNode::Branch {
                voltage: ElectricPotential::ZERO,
            }),
            graph.add_node(ElectricalNode::ResistanceConsumer {
                resistance: ElectricalResistance::new::<ohm>(1.0),
                voltage: ElectricPotential::ZERO,
            }),
            graph.add_node(ElectricalNode::ResistanceConsumer {
                resistance: ElectricalResistance::new::<ohm>(1.0),
                voltage: ElectricPotential::ZERO,
            }),
        ];
        graph.add_edge(n[0], n[2], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[1], n[2], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[3], n[2], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[4], n[2], ElectricalResistance::new::<ohm>(1.0));

        let mut resistance_network = ResistanceNetwork::new(graph);
        resistance_network.update_voltages().unwrap();

        let voltages: Vec<_> = resistance_network
            .graph()
            .node_weights()
            .map(ElectricalNode::voltage)
            .collect();
        let expected_voltages: Vec<_> = [1.0, 2.0, 1.0, 0.5, 0.5]
            .into_iter()
            .map(ElectricPotential::new::<volt>)
            .collect();

        for (i, (voltage, expected_voltage)) in voltages
            .iter()
            .copied()
            .zip(expected_voltages.iter().copied())
            .enumerate()
        {
            assert!(
                (voltage - expected_voltage).abs() < ElectricPotential::new::<volt>(1e-10),
                "Voltages at index {i} differ. Expected: {:.2}. Actual: {:.2}.",
                expected_voltage.into_format_args(volt, uom::fmt::DisplayStyle::Abbreviation),
                voltage.into_format_args(volt, uom::fmt::DisplayStyle::Abbreviation)
            );
        }
    }
}
