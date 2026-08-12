use super::types::*;

impl FCoseLayout {
    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_ideal_edge_length(mut self, length: f32) -> Self {
        self.ideal_edge_length = length;
        self
    }

    pub fn with_nesting_factor(mut self, factor: f32) -> Self {
        self.nesting_factor = factor;
        self
    }

    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_node_repulsion(mut self, repulsion: f32) -> Self {
        self.node_repulsion = repulsion;
        self
    }

    pub fn with_initial_temp(mut self, temp: f32) -> Self {
        self.initial_temp = temp;
        self
    }

    pub fn with_cooling_factor(mut self, factor: f32) -> Self {
        self.cooling_factor = factor;
        self
    }

    pub fn with_randomize(mut self, randomize: bool) -> Self {
        self.randomize = randomize;
        self
    }

    pub fn with_compound_padding(mut self, padding: f32) -> Self {
        self.compound_padding = padding;
        self
    }

    pub fn with_gravity_range(mut self, range: f32) -> Self {
        self.gravity_range = range;
        self
    }

    pub fn with_gravity_compound(mut self, g: f32) -> Self {
        self.gravity_compound = g;
        self
    }

    pub fn with_gravity_range_compound(mut self, r: f32) -> Self {
        self.gravity_range_compound = r;
        self
    }

    pub fn with_tile(mut self, tile: bool) -> Self {
        self.tile = tile;
        self
    }

    pub fn with_tiling_padding_horizontal(mut self, p: f32) -> Self {
        self.tiling_padding_horizontal = p;
        self
    }

    pub fn with_tiling_padding_vertical(mut self, p: f32) -> Self {
        self.tiling_padding_vertical = p;
        self
    }

    pub fn with_pack_components(mut self, pack: bool) -> Self {
        self.pack_components = pack;
        self
    }

    pub fn with_node_dimensions_include_labels(mut self, include: bool) -> Self {
        self.node_dimensions_include_labels = include;
        self
    }

    pub fn with_constraints(mut self, constraints: FCoseConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_fixed_node_constraint(mut self, constraint: FixedNodeConstraint) -> Self {
        self.constraints.fixed_nodes.push(constraint);
        self
    }

    pub fn with_alignment_constraint(mut self, alignment: AlignmentConstraint) -> Self {
        self.constraints.alignment = alignment;
        self
    }

    pub fn with_relative_placement_constraint(mut self, relative: RelativePlacementConstraint) -> Self {
        self.constraints.relative_placement.push(relative);
        self
    }

    pub fn with_node_repulsion_metric(mut self, metric: NodeRepulsionMetric) -> Self {
        self.node_repulsion_metric = Some(metric);
        self
    }

    pub fn with_ideal_edge_length_metric(mut self, metric: EdgeMetric) -> Self {
        self.ideal_edge_length_metric = Some(metric);
        self
    }

    pub fn with_edge_elasticity_metric(mut self, metric: EdgeMetric) -> Self {
        self.edge_elasticity_metric = Some(metric);
        self
    }
}
