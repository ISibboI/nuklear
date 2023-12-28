use crate::electrical_grid::resistance_network::ElectricalNodeTypes;

pub trait TypeParamerisation {
    type ElectricalNodeTypes: ElectricalNodeTypes;
}