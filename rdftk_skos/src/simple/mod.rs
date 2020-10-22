/*!
A simple model for constructing SKOS thesauri. This is not a complete API in
that it's extensibility with OWL is limited.

Details TBD

# Example

TBD

*/

use crate::ns;
use rdftk_core::graph::{Graph, PrefixMappings};
use rdftk_core::{ObjectNode, Statement, SubjectNode};
use rdftk_iri::IRIRef;
use rdftk_memgraph::{Mappings, MemGraph};
use rdftk_names::{dc, owl, rdf, xsd};
use std::rc::Rc;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

pub trait Named {
    fn uri(&self) -> &IRIRef;
}

pub trait Propertied {
    fn add_property(&mut self, property: LiteralProperty);

    fn has_property(&self, predicate: &IRIRef) -> bool {
        self.properties()
            .iter()
            .any(|property| property.predicate() == predicate)
    }

    fn has_properties(&self) -> bool {
        !self.properties().is_empty()
    }

    fn properties(&self) -> &Vec<LiteralProperty>;
}

pub trait Labeled {
    fn add_label(&mut self, label: Label);

    fn add_preferred_label(&mut self, text: &str, language: &str) {
        self.add_label(Label::preferred(text, language))
    }

    fn add_alternative_label(&mut self, text: &str, language: &str) {
        self.add_label(Label::alternative(text, language))
    }

    fn add_hidden_label(&mut self, text: &str, language: &str) {
        self.add_label(Label::hidden(text, language))
    }

    fn preferred_label(&self, language: &str) -> String;

    fn has_labels(&self) -> bool;

    fn labels(&self) -> &Vec<Label>;
}

pub trait ToStatements {
    fn to_statements(&self, in_scheme: Option<&ObjectNode>) -> Vec<Statement>;
}

pub trait ToStatement {
    fn to_statement(&self, subject: &SubjectNode) -> Statement;
}

pub trait ToURI {
    fn to_uri(&self) -> IRIRef;
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub fn to_rdf_graph(scheme: &Scheme, default_namespace: Option<IRIRef>) -> MemGraph {
    let mut graph = MemGraph::default();

    let mut mappings = standard_mappings();
    if let Some(default_namespace) = default_namespace {
        mappings.insert_default(default_namespace.clone());
    }
    graph.mappings(Rc::new(mappings));

    for statement in scheme.to_statements(None) {
        graph.insert(statement);
    }

    graph
}

pub fn standard_mappings() -> Mappings {
    let mut mappings = Mappings::default();
    mappings.insert(ns::default_prefix(), ns::namespace_iri().clone());
    mappings.insert(ns::xl::default_prefix(), ns::xl::namespace_iri().clone());
    mappings.insert(ns::iso::default_prefix(), ns::iso::namespace_iri().clone());
    mappings.insert(
        dc::terms::default_prefix(),
        dc::terms::namespace_iri().clone(),
    );
    mappings.insert(rdf::default_prefix(), rdf::namespace_iri().clone());
    mappings.insert(owl::default_prefix(), owl::namespace_iri().clone());
    mappings.insert(xsd::default_prefix(), xsd::namespace_iri().clone());
    mappings
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

pub mod scheme;
pub use scheme::Scheme;

pub mod concept;
pub use concept::Concept;

pub mod collection;
pub use collection::Collection;

pub mod properties;
pub use properties::{Label, LiteralProperty};