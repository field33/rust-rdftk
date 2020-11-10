/*!
An extension to the core `Graph` to support named graphs.

# Example

TBD

*/

#![allow(clippy::module_name_repetitions)]

use crate::graph::MutableGraph;
use crate::Graph;
use rdftk_iri::IRIRef;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

pub trait NamedGraph: Graph {
    fn name(&self) -> &Option<IRIRef>;
}

pub trait MutableNamedGraph: NamedGraph + MutableGraph {
    fn set_name(&mut self, name: IRIRef) -> Option<IRIRef>;

    fn unset_name(&mut self) -> Option<IRIRef>;
}
