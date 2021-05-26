/*!
A simple model for constructing SKOS thesauri. This is not a complete API in
that it's extensibility with OWL is limited.

Details TBD

# Example

TBD

*/

use crate::ns;
use rdftk_core::graph::{GraphRef, PrefixMappings};
use rdftk_iri::IRIRef;
use rdftk_memgraph::simple::graph_factory;
use rdftk_names::{dc, owl, rdf, xsd};
use std::rc::Rc;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

pub trait Labeled {
    fn add_label(&mut self, label: Label);

    fn has_labels(&self) -> bool;

    fn labels(&self) -> &Vec<Label>;

    // --------------------------------------------------------------------------------------------

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

    // --------------------------------------------------------------------------------------------

    fn define(&mut self, text: &str, language: &str) -> &mut Self {
        self.add_property(LiteralProperty::definition_with(text, language));
        self
    }

    fn notation(&mut self, notation: &str) -> &mut Self {
        self.add_property(LiteralProperty::notation(notation));
        self
    }

    fn copyright(&mut self, publisher: &str, rights: &str) -> &mut Self {
        self.add_property(LiteralProperty::publisher(publisher));
        self.add_property(LiteralProperty::rights(rights));
        self
    }
}

pub trait Resource: Labeled + Propertied {
    fn uri(&self) -> &IRIRef;
}

pub trait ToStatements {
    fn to_statements(&self, in_scheme: Option<&ObjectNodeRef>) -> StatementList;
}

pub trait ToStatement {
    fn to_statement(&self, subject: &SubjectNodeRef) -> StatementRef;
}

pub trait ToUri {
    fn to_uri(&self) -> IRIRef;
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub fn to_rdf_graph(scheme: &Scheme, default_namespace: Option<IRIRef>) -> GraphRef {
    let ns_mappings = standard_mappings();
    if let Some(default_namespace) = default_namespace {
        let mut ns_mappings = ns_mappings.borrow_mut();
        let _ = ns_mappings.set_default_namespace(default_namespace);
    }
    to_rdf_graph_with_mappings(scheme, ns_mappings)
}

pub fn to_rdf_graph_with_mappings(scheme: &Scheme, ns_mappings: PrefixMappingRef) -> GraphRef {
    let graph = graph_factory().new_graph();
    {
        let mut graph = graph.borrow_mut();
        let _ = graph.set_prefix_mappings(ns_mappings);

        for statement in scheme.to_statements(None) {
            graph.insert(statement.into());
        }
    }
    graph
}

pub fn from_rdf_graph<'a>(graph: &GraphRef) -> Vec<Scheme> {
    let schemes = Default::default();
    let graph = graph.borrow();
    let scheme_subjects: Vec<&SubjectNodeRef> = graph
        .statements()
        .filter_map(|st| {
            if st.predicate() == rdf::a_type() && st.object().eq_iri(ns::concept_scheme()) {
                Some(st.subject())
            } else {
                None
            }
        })
        .collect();
    for subject in scheme_subjects {}
    todo!();
    schemes
}

pub fn standard_mappings() -> PrefixMappingRef {
    let mut mappings = PrefixMappings::default();
    let _ = mappings.insert(ns::default_prefix(), ns::namespace_iri().clone());
    let _ = mappings.insert(ns::xl::default_prefix(), ns::xl::namespace_iri().clone());
    let _ = mappings.insert(ns::iso::default_prefix(), ns::iso::namespace_iri().clone());
    let _ = mappings.insert(
        ns::term_status::default_prefix(),
        ns::term_status::namespace_iri().clone(),
    );
    let _ = mappings.insert(
        dc::terms::default_prefix(),
        dc::terms::namespace_iri().clone(),
    );
    let _ = mappings.insert(rdf::default_prefix(), rdf::namespace_iri().clone());
    let _ = mappings.insert(owl::default_prefix(), owl::namespace_iri().clone());
    let _ = mappings.insert(xsd::default_prefix(), xsd::namespace_iri().clone());
    Rc::new(RefCell::new(mappings))
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
use rdftk_core::graph::mapping::PrefixMappingRef;
use rdftk_core::statement::{ObjectNodeRef, StatementList, StatementRef, SubjectNodeRef};
use std::cell::RefCell;
