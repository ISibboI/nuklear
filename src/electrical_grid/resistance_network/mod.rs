use std::fmt::Debug;

use interior_mut::InteriorMut;
use nalgebra::DMatrix;
use petgraph::{graph::UnGraph, visit::IntoNodeReferences};
use uom::{
    si::{
        electric_potential::volt,
        electrical_conductance::siemens,
        electrical_resistance::ohm,
        f64::{ElectricCurrent, ElectricPotential, ElectricalConductance, ElectricalResistance},
    },
    ConstZero,
};

use crate::{error::Error, type_parameterisation::TypeParamerisation};

#[derive(Debug)]
pub struct ResistanceNetwork<Types: TypeParamerisation> {
    graph: UnGraph<ElectricalNode<Types>, ElectricalResistance, usize>,
    unknown_voltage_indexes_to_node_indexes: Vec<usize>,
    node_indexes_to_unknown_voltage_indexes: Vec<usize>,
}

#[derive(Debug)]
pub enum ElectricalNode<Types: TypeParamerisation> {
    ConstantVoltageSource(<<Types as TypeParamerisation>::ElectricalNodeTypes as ElectricalNodeTypes>::ConstantVoltageSourceWrapper),
    Branch(<<Types as TypeParamerisation>::ElectricalNodeTypes as ElectricalNodeTypes>::BranchWrapper),
    ConstantResistanceConsumer(<<Types as TypeParamerisation>::ElectricalNodeTypes as ElectricalNodeTypes>::ConstantResistanceConsumerWrapper),
}

pub trait ConstantVoltageSource: Debug {
    fn voltage(&self) -> ElectricPotential;

    /// The resistance between the constant voltage source and the external connection of this source.
    fn inner_resistance(&self) -> ElectricalResistance;

    fn set_current(&mut self, current: ElectricCurrent);
}

pub trait Branch: Debug {
    fn set_voltage(&mut self, voltage: ElectricPotential);
}

pub trait ConstantResistanceConsumer: Debug {
    /// The resistance between ground and the external connection of this consumer.
    fn inner_resistance(&self) -> ElectricalResistance;

    fn set_voltage(&mut self, voltage: ElectricPotential);

    fn set_current(&mut self, current: ElectricCurrent);
}

pub trait ElectricalNodeTypes {
    type ConstantVoltageSource: ConstantVoltageSource + ?Sized;
    type ConstantVoltageSourceWrapper: InteriorMut<Self::ConstantVoltageSource> + Debug;
    type Branch: Branch + ?Sized;
    type BranchWrapper: InteriorMut<Self::Branch> + Debug;
    type ConstantResistanceConsumer: ConstantResistanceConsumer + ?Sized;
    type ConstantResistanceConsumerWrapper: InteriorMut<Self::ConstantResistanceConsumer> + Debug;
}

impl<Types: TypeParamerisation> ResistanceNetwork<Types> {
    pub fn new(graph: UnGraph<ElectricalNode<Types>, ElectricalResistance, usize>) -> Self {
        let unknown_voltage_indexes_to_node_indexes: Vec<_> = graph
            .node_references()
            .filter_map(|(index, node)| {
                if matches!(
                    node,
                    ElectricalNode::Branch(_) | ElectricalNode::ConstantResistanceConsumer(_)
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

        Self {
            graph,
            unknown_voltage_indexes_to_node_indexes,
            node_indexes_to_unknown_voltage_indexes,
        }
    }

    pub fn graph(&self) -> &UnGraph<ElectricalNode<Types>, ElectricalResistance, usize> {
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
            if let ElectricalNode::ConstantResistanceConsumer(consumer) = node {
                let conductance = 1.0
                    / consumer
                        .borrow_int()
                        .unwrap()
                        .inner_resistance()
                        .get::<ohm>();
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
                    let ElectricalNode::ConstantVoltageSource(source) =
                        self.graph.node_weight(neighbor_index).unwrap()
                    else {
                        unreachable!(
                            "only voltage sources have a given voltage\nnode: {node_index:?}, neighbor_node: {neighbor_index:?}"
                        );
                    };
                    current_vector[(voltage_index, 0)] +=
                        source.borrow_int().unwrap().voltage().get::<volt>() * conductance;
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

        for (node_index, node) in self.graph.node_references() {
            match node {
                ElectricalNode::ConstantVoltageSource(source) => {
                    let mut source = source.borrow_int_mut().unwrap();
                    let mut current = ElectricCurrent::ZERO;
                    for neighbor_index in self.graph.neighbors(node_index) {
                        if neighbor_index == node_index {
                            // Self loops do not contribute to current.
                            continue;
                        }

                        let conductance = self
                            .graph
                            .edges_connecting(node_index, neighbor_index)
                            .map(|edge| 1.0 / *edge.weight())
                            .sum::<ElectricalConductance>();
                        let voltage = source.voltage()
                            - match self.graph.node_weight(neighbor_index).unwrap() {
                                ElectricalNode::ConstantVoltageSource(source) => {
                                    source.borrow_int().unwrap().voltage()
                                }
                                ElectricalNode::Branch(_)
                                | ElectricalNode::ConstantResistanceConsumer(_) => {
                                    let neighbor_voltage_index = self
                                        .node_indexes_to_unknown_voltage_indexes
                                        [neighbor_index.index()];
                                    debug_assert_ne!(neighbor_voltage_index, usize::MAX);
                                    ElectricPotential::new::<volt>(voltages[neighbor_voltage_index])
                                }
                            };
                        current += voltage * conductance;
                    }
                    source.set_current(current);
                }
                ElectricalNode::Branch(branch) => {
                    let computed_voltage = voltages[(
                        self.node_indexes_to_unknown_voltage_indexes[node_index.index()],
                        0,
                    )];
                    let computed_voltage = ElectricPotential::new::<volt>(computed_voltage);
                    
                    branch
                        .borrow_int_mut()
                        .unwrap()
                        .set_voltage(computed_voltage);
                }
                ElectricalNode::ConstantResistanceConsumer(consumer) => {
                    let computed_voltage = voltages[(
                        self.node_indexes_to_unknown_voltage_indexes[node_index.index()],
                        0,
                    )];
                    let computed_voltage = ElectricPotential::new::<volt>(computed_voltage);
                    
                    let mut consumer = consumer.borrow_int_mut().unwrap();
                    consumer.set_voltage(computed_voltage);
                    let inner_resistance = consumer.inner_resistance();
                    consumer.set_current(computed_voltage / inner_resistance);
                }
            }
        }

        for (unknown_voltage_index, computed_voltage) in voltages.iter().copied().enumerate() {
            let node_index = self.unknown_voltage_indexes_to_node_indexes[unknown_voltage_index];
            let computed_voltage = ElectricPotential::new::<volt>(computed_voltage);
            match self.graph.node_weight_mut(node_index.into()).unwrap() {
                ElectricalNode::ConstantVoltageSource(_) => {
                    unreachable!("a voltage source has a known voltage");
                }
                ElectricalNode::Branch(branch) => branch
                    .borrow_int_mut()
                    .unwrap()
                    .set_voltage(computed_voltage),
                ElectricalNode::ConstantResistanceConsumer(consumer) => {
                    let mut consumer = consumer.borrow_int_mut().unwrap();
                    consumer.set_voltage(computed_voltage);
                    let inner_resistance = consumer.inner_resistance();
                    consumer.set_current(computed_voltage / inner_resistance);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use petgraph::graph::UnGraph;
    use uom::{
        si::{
            electric_current::ampere,
            electric_potential::volt,
            electrical_resistance::ohm,
            f64::{ElectricCurrent, ElectricPotential, ElectricalResistance},
        },
        ConstZero,
    };

    use crate::type_parameterisation::TypeParamerisation;

    use super::{
        Branch, ConstantResistanceConsumer, ConstantVoltageSource, ElectricalNode,
        ElectricalNodeTypes, ResistanceNetwork,
    };

    #[derive(Default, Debug)]
    struct TestVoltageSource {
        pub voltage: ElectricPotential,
        pub inner_resistance: ElectricalResistance,
        pub current: ElectricCurrent,
    }

    impl TestVoltageSource {
        fn new(voltage: f64) -> Self {
            Self {
                voltage: ElectricPotential::new::<volt>(voltage),
                ..Default::default()
            }
        }
    }

    impl ConstantVoltageSource for TestVoltageSource {
        fn voltage(&self) -> ElectricPotential {
            self.voltage
        }

        fn inner_resistance(&self) -> ElectricalResistance {
            self.inner_resistance
        }

        fn set_current(&mut self, current: ElectricCurrent) {
            self.current = current;
        }
    }

    #[derive(Default, Debug)]
    struct TestBranch {
        pub voltage: ElectricPotential,
    }

    impl TestBranch {
        fn new() -> Self {
            Default::default()
        }
    }

    impl Branch for TestBranch {
        fn set_voltage(&mut self, voltage: ElectricPotential) {
            self.voltage = voltage;
        }
    }

    #[derive(Default, Debug)]
    struct TestResistanceConsumer {
        pub voltage: ElectricPotential,
        pub inner_resistance: ElectricalResistance,
        pub current: ElectricCurrent,
    }

    impl TestResistanceConsumer {
        fn new(resistance: f64) -> Self {
            Self {
                inner_resistance: ElectricalResistance::new::<ohm>(resistance),
                ..Default::default()
            }
        }
    }

    impl ConstantResistanceConsumer for TestResistanceConsumer {
        fn inner_resistance(&self) -> ElectricalResistance {
            self.inner_resistance
        }

        fn set_current(&mut self, current: ElectricCurrent) {
            self.current = current;
        }

        fn set_voltage(&mut self, voltage: ElectricPotential) {
            self.voltage = voltage;
        }
    }

    struct TestTypeParameterisation;

    struct TestElectricalNodeTypes;

    impl TypeParamerisation for TestTypeParameterisation {
        type ElectricalNodeTypes = TestElectricalNodeTypes;
    }

    impl ElectricalNodeTypes for TestElectricalNodeTypes {
        type ConstantVoltageSource = TestVoltageSource;

        type ConstantVoltageSourceWrapper = RefCell<TestVoltageSource>;

        type Branch = TestBranch;

        type BranchWrapper = RefCell<TestBranch>;

        type ConstantResistanceConsumer = TestResistanceConsumer;

        type ConstantResistanceConsumerWrapper = RefCell<TestResistanceConsumer>;
    }

    fn verify_voltage_currents(
        resistance_network: &ResistanceNetwork<TestTypeParameterisation>,
        expected_voltages_currents: &[(f64, f64)],
    ) {
        let voltages_currents: Vec<(ElectricPotential, ElectricCurrent)> = resistance_network
            .graph()
            .node_weights()
            .map(|node| match node {
                ElectricalNode::ConstantVoltageSource(source) => {
                    (source.borrow().voltage, source.borrow().current)
                }
                ElectricalNode::Branch(branch) => (branch.borrow().voltage, ElectricCurrent::ZERO),
                ElectricalNode::ConstantResistanceConsumer(consumer) => {
                    (consumer.borrow().voltage, consumer.borrow().current)
                }
            })
            .collect();
        let expected_voltages_currents: Vec<_> = expected_voltages_currents
            .iter()
            .copied()
            .map(|(voltage, current)| {
                (
                    ElectricPotential::new::<volt>(voltage),
                    ElectricCurrent::new::<ampere>(current),
                )
            })
            .collect();

        let mut has_difference = false;
        for (i, ((voltage, current), (expected_voltage, expected_current))) in voltages_currents
            .iter()
            .copied()
            .zip(expected_voltages_currents.iter().copied())
            .enumerate()
        {
            if (voltage - expected_voltage).abs() > ElectricPotential::new::<volt>(1e-10) {
                println!(
                    "Voltages at index {i} differ. Expected: {:.2}. Actual: {:.2}.",
                    expected_voltage.into_format_args(volt, uom::fmt::DisplayStyle::Abbreviation),
                    voltage.into_format_args(volt, uom::fmt::DisplayStyle::Abbreviation)
                );
                has_difference = true;
            }
            if (current - expected_current).abs() > ElectricCurrent::new::<ampere>(1e-10) {
                println!(
                    "Currents at index {i} differ. Expected: {:.2}. Actual: {:.2}.",
                    expected_current.into_format_args(ampere, uom::fmt::DisplayStyle::Abbreviation),
                    current.into_format_args(ampere, uom::fmt::DisplayStyle::Abbreviation)
                );
                has_difference = true;
            }
        }

        assert!(!has_difference);
    }

    #[test]
    fn linear() {
        let mut graph = UnGraph::default();
        let n = [
            graph.add_node(
                ElectricalNode::<TestTypeParameterisation>::ConstantVoltageSource(
                    TestVoltageSource::new(1.0).into(),
                ),
            ),
            graph.add_node(ElectricalNode::ConstantResistanceConsumer(
                TestResistanceConsumer::new(1.0).into(),
            )),
            graph.add_node(ElectricalNode::ConstantResistanceConsumer(
                TestResistanceConsumer::new(1.0).into(),
            )),
        ];
        graph.add_edge(n[0], n[1], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[1], n[2], ElectricalResistance::new::<ohm>(1.0));

        let mut resistance_network = ResistanceNetwork::new(graph);
        resistance_network.update_voltages().unwrap();

        verify_voltage_currents(&resistance_network, &[(1.0, 0.6), (0.4, 0.4), (0.2, 0.2)]);
    }

    #[test]
    fn star() {
        let mut graph = UnGraph::default();
        let n = [
            graph.add_node(ElectricalNode::ConstantVoltageSource(
                TestVoltageSource::new(1.0).into(),
            )),
            graph.add_node(ElectricalNode::ConstantVoltageSource(
                TestVoltageSource::new(2.0).into(),
            )),
            graph.add_node(ElectricalNode::Branch(TestBranch::new().into())),
            graph.add_node(ElectricalNode::ConstantResistanceConsumer(
                TestResistanceConsumer::new(1.0).into(),
            )),
            graph.add_node(ElectricalNode::ConstantResistanceConsumer(
                TestResistanceConsumer::new(1.0).into(),
            )),
        ];
        graph.add_edge(n[0], n[2], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[1], n[2], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[3], n[2], ElectricalResistance::new::<ohm>(1.0));
        graph.add_edge(n[4], n[2], ElectricalResistance::new::<ohm>(1.0));

        let mut resistance_network = ResistanceNetwork::new(graph);
        resistance_network.update_voltages().unwrap();

        verify_voltage_currents(
            &resistance_network,
            &[(1.0, 0.0), (2.0, 1.0), (1.0, 0.0), (0.5, 0.5), (0.5, 0.5)],
        );
    }
}
