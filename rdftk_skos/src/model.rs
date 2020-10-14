/*!
A simple model for constructing SKOS thesauri. This is not a complete API in
that it's extensibility with OWL is limited.

Details TBD

# Example

TBD

*/

use crate::ns;
use rdftk_core::{Literal, ObjectNode, Statement, SubjectNode};
use rdftk_graph::{Graph, PrefixMappings};
use rdftk_iri::{IRIRef, IRI};
use rdftk_memgraph::{Mappings, MemGraph};
use rdftk_names::{dc, owl, rdf, xsd};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Scheme {
    uri: IRIRef,
    concepts: HashMap<IRIRef, Concept>,
    top_concepts: HashSet<IRIRef>,
    collections: HashSet<Collection>,
    properties: Vec<LiteralProperty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Concept {
    uri: IRIRef,
    relations: Vec<ObjectProperty>,
    properties: Vec<LiteralProperty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Collection {
    uri: IRIRef,
    ordered: bool,
    members: Vec<IRIRef>,
    properties: Vec<LiteralProperty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LiteralProperty {
    predicate: IRIRef,
    value: Literal,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectProperty {
    predicate: IRIRef,
    other: IRIRef,
}

// ------------------------------------------------------------------------------------------------

pub trait Named {
    fn new(uri: IRIRef) -> Self;

    fn new_with_label(uri: IRIRef, text: &str, language: Option<&str>) -> Self;

    fn uri(&self) -> &IRIRef;
}

pub trait Propertied {
    fn add_property(&mut self, property: LiteralProperty) {
        self.properties_mut().push(property);
    }
    fn has_property(&self, predicate: &IRIRef) -> bool {
        self.properties()
            .iter()
            .any(|property| property.predicate() == predicate)
    }
    fn has_properties(&self) -> bool {
        !self.properties().is_empty()
    }
    fn properties(&self) -> Vec<&LiteralProperty>;
    fn properties_mut(&mut self) -> &mut Vec<LiteralProperty>;
}

pub trait Labeled: Propertied {
    fn preferred_label(&self, for_language: &str) -> Option<String> {
        let preferred: HashMap<&Option<String>, &String> = self
            .properties()
            .iter()
            .filter(|property| property.predicate() == &ns::pref_label())
            .map(|label| (label.value().language(), label.value().lexical_form()))
            .collect();
        let language = Some(for_language.to_string());
        if preferred.contains_key(&language) {
            preferred.get(&language).cloned().cloned()
        } else if preferred.contains_key(&None) {
            preferred.get(&None).cloned().cloned()
        } else {
            let first_language = <&std::option::Option<std::string::String>>::clone(
                preferred.keys().next().unwrap(),
            );
            preferred.get(first_language).cloned().cloned()
        }
    }
    fn has_labels(&self) -> bool {
        self.properties().iter().any(|property| {
            property.predicate == *ns::pref_label()
                || property.predicate == *ns::alt_label()
                || property.predicate == *ns::hidden_label()
        })
    }
    fn labels(&self) -> Vec<&LiteralProperty> {
        self.properties()
            .into_iter()
            .filter(|property| {
                property.predicate == *ns::pref_label()
                    || property.predicate == *ns::alt_label()
                    || property.predicate == *ns::hidden_label()
            })
            .collect()
    }
    fn add_preferred_label(&mut self, text: &str) {
        self.properties_mut()
            .push(LiteralProperty::preferred_label(text))
    }
    fn add_preferred_label_with(&mut self, text: &str, language: &str) {
        self.properties_mut()
            .push(LiteralProperty::preferred_label_with(text, language))
    }
    fn add_alternative_label(&mut self, text: &str) {
        self.properties_mut()
            .push(LiteralProperty::alternative_label(text))
    }
    fn add_alternative_label_with(&mut self, text: &str, language: &str) {
        self.properties_mut()
            .push(LiteralProperty::alternative_label_with(text, language))
    }
    fn add_hidden_label(&mut self, text: &str) {
        self.properties_mut()
            .push(LiteralProperty::hidden_label(text))
    }
    fn add_hidden_label_with(&mut self, text: &str, language: &str) {
        self.properties_mut()
            .push(LiteralProperty::hidden_label_with(text, language))
    }
}

pub trait ToStatements {
    fn to_statements(&self) -> Vec<Statement>;
}

pub trait ToStatement {
    fn to_statement(&self, subject: &SubjectNode) -> Statement;
}

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub fn to_rdf_graph(scheme: &Scheme, default_namespace: Option<IRI>) -> MemGraph {
    let mut graph = MemGraph::default();

    let mut mappings = standard_mappings();
    if let Some(default_namespace) = default_namespace {
        mappings.insert_default(default_namespace.clone());
    }
    graph.mappings(Rc::new(mappings));

    for statement in scheme.to_statements() {
        graph.insert(statement);
    }

    graph
}

pub fn standard_mappings() -> Mappings {
    let mut mappings = Mappings::default();
    mappings.insert(ns::prefix(), ns::namespace());
    mappings.insert(ns::xl::prefix(), ns::xl::namespace());
    mappings.insert(ns::iso::prefix(), ns::iso::namespace());
    mappings.insert(dc::terms::prefix(), dc::terms::namespace());
    mappings.insert(rdf::prefix(), rdf::namespace());
    mappings.insert(owl::prefix(), owl::namespace());
    mappings.insert(xsd::prefix(), xsd::namespace());
    mappings
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Named for Scheme {
    fn new(uri: IRIRef) -> Self {
        Self {
            uri,
            concepts: Default::default(),
            top_concepts: Default::default(),
            collections: Default::default(),
            properties: Default::default(),
        }
    }

    fn new_with_label(uri: IRIRef, text: &str, language: Option<&str>) -> Self {
        let mut scheme = Self::new(uri);
        scheme.properties.push(match language {
            None => LiteralProperty::preferred_label(text),
            Some(language) => LiteralProperty::preferred_label_with(text, language),
        });
        scheme
    }

    fn uri(&self) -> &IRIRef {
        &self.uri
    }
}

impl Propertied for Scheme {
    fn properties(&self) -> Vec<&LiteralProperty> {
        self.properties.iter().collect()
    }

    fn properties_mut(&mut self) -> &mut Vec<LiteralProperty> {
        &mut self.properties
    }
}

impl Labeled for Scheme {}

impl ToStatements for Scheme {
    fn to_statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Default::default();
        let subject = SubjectNode::named(*self.uri().clone());
        statements.push(Statement::new(
            subject.clone(),
            rdf::a_type(),
            ns::concept_scheme().into(),
        ));
        for member in self.concepts() {
            statements.extend(member.to_statements().drain(..));
            if self.top_concepts.contains(member.uri()) {
                statements.push(Statement::new(
                    SubjectNode::named(member.uri().clone()),
                    ns::top_concept_of(),
                    subject.clone().into(),
                ));
            }
            statements.push(Statement::new(
                SubjectNode::named(member.uri().clone()),
                ns::in_scheme(),
                ObjectNode::named(*self.uri().clone()),
            ));
        }
        for member in &self.top_concepts {
            statements.push(Statement::new(
                subject.clone(),
                ns::has_top_concept(),
                ObjectNode::named(*member.clone()),
            ));
        }
        for member in self.collections() {
            statements.extend(member.to_statements().drain(..));
            statements.push(Statement::new(
                SubjectNode::named(member.uri().clone()),
                ns::in_scheme(),
                ObjectNode::named(*self.uri().clone()),
            ));
        }
        for property in self.properties() {
            statements.push(property.to_statement(&subject));
        }
        statements
    }
}

impl Scheme {
    pub fn add_concept(&mut self, concept: Concept) {
        self.concepts.insert(concept);
    }
    pub fn add_top_concept(&mut self, concept: Concept) {
        let uri = concept.uri().clone();
        self.concepts.insert(concept);
        self.top_concepts.insert(uri);
    }
    pub fn is_top_collection(&self, uri: &IRI) -> bool {
        self.top_concepts.contains(uri)
    }
    pub fn has_concepts(&self) -> bool {
        !self.concepts.is_empty()
    }
    pub fn has_concept(&self, uri: &IRI) -> bool {
        self.concepts().any(|concept| concept.uri() == uri)
    }
    pub fn concepts(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.iter()
    }
    pub fn concept(&self, uri: &IRI) -> Option<&Concept> {
        self.concepts().find(|concept| concept.uri() == uri)
    }

    pub fn add_collection(&mut self, collection: Collection) {
        self.collections.insert(collection);
    }
    pub fn has_collections(&self) -> bool {
        !self.collections.is_empty()
    }
    pub fn has_collection(&self, uri: &IRI) -> bool {
        self.collections().any(|collection| collection.uri() == uri)
    }
    pub fn collections(&self) -> impl Iterator<Item = &Collection> {
        self.collections.iter()
    }
    pub fn collection(&self, uri: &IRI) -> Option<&Collection> {
        self.collections()
            .find(|collection| collection.uri() == uri)
    }
}

// ------------------------------------------------------------------------------------------------

impl Named for Concept {
    fn new(uri: IRI) -> Self {
        Self {
            uri,
            relations: Default::default(),
            properties: Default::default(),
        }
    }

    fn new_with_label(uri: IRI, text: &str, language: Option<&str>) -> Self {
        let mut concept = Self::new(uri);
        concept.properties.push(match language {
            None => LiteralProperty::preferred_label(text),
            Some(language) => LiteralProperty::preferred_label_with(text, language),
        });
        concept
    }

    fn uri(&self) -> &IRI {
        &self.uri
    }
}

impl Propertied for Concept {
    fn properties(&self) -> Vec<&LiteralProperty> {
        self.properties.iter().collect()
    }

    fn properties_mut(&mut self) -> &mut Vec<LiteralProperty> {
        &mut self.properties
    }
}

impl Labeled for Concept {}

impl ToStatements for Concept {
    fn to_statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Default::default();
        let subject = SubjectNode::named(self.uri().clone());
        statements.push(Statement::new(
            subject.clone(),
            rdf::a_type(),
            ns::concept().into(),
        ));
        for relation in self.relations() {
            statements.push(relation.to_statement(&subject));
        }
        for property in self.properties() {
            statements.push(property.to_statement(&subject));
        }
        statements
    }
}

impl Concept {
    pub fn broader(&mut self, uri: IRI) -> Self {
        let mut new_concept = Self::new(uri);
        new_concept.add_relation(ObjectProperty::narrower(self.uri().clone()));
        self.add_relation(ObjectProperty::broader(new_concept.uri().clone()));
        new_concept
    }

    pub fn narrower(&mut self, uri: IRI) -> Self {
        let mut new_concept = Self::new(uri);
        new_concept.add_relation(ObjectProperty::broader(self.uri().clone()));
        self.add_relation(ObjectProperty::narrower(new_concept.uri().clone()));
        new_concept
    }

    pub fn add_relation(&mut self, relation: ObjectProperty) {
        self.relations.push(relation);
    }
    pub fn has_relation(&self, predicate: &IRI) -> bool {
        self.relations()
            .any(|relation| relation.predicate() == predicate)
    }
    pub fn has_relations(&self) -> bool {
        !self.relations.is_empty()
    }
    pub fn relations(&self) -> impl Iterator<Item = &ObjectProperty> {
        self.relations.iter()
    }
}

// ------------------------------------------------------------------------------------------------

impl Named for Collection {
    fn new(uri: IRI) -> Self {
        Self {
            uri,
            ordered: false,
            members: Default::default(),
            properties: Default::default(),
        }
    }

    fn new_with_label(uri: IRI, text: &str, language: Option<&str>) -> Self {
        let mut collection = Self {
            uri,
            ordered: false,
            members: Default::default(),
            properties: Default::default(),
        };
        collection.add_property(match language {
            None => LiteralProperty::preferred_label(text),
            Some(language) => LiteralProperty::preferred_label_with(text, language),
        });
        collection
    }

    fn uri(&self) -> &IRI {
        &self.uri
    }
}

impl Propertied for Collection {
    fn properties(&self) -> Vec<&LiteralProperty> {
        self.properties.iter().collect()
    }

    fn properties_mut(&mut self) -> &mut Vec<LiteralProperty> {
        &mut self.properties
    }
}

impl Labeled for Collection {}

impl ToStatements for Collection {
    fn to_statements(&self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Default::default();
        let subject = SubjectNode::named(self.uri().clone());
        if self.ordered {
            statements.push(Statement::new(
                subject.clone(),
                rdf::a_type(),
                ns::ordered_collection().into(),
            ));
        } else {
            statements.push(Statement::new(
                subject.clone(),
                rdf::a_type(),
                ns::collection().into(),
            ));
        }
        for member in self.members() {
            statements.push(member.to_statement(&subject));
        }
        for property in self.properties() {
            statements.push(property.to_statement(&subject));
        }
        statements
    }
}

impl Collection {
    pub fn member_collection(&mut self, uri: IRI, ordered: bool) -> Self {
        let mut member = Self::new(uri.clone());
        member.set_ordered(ordered);
        self.add_member(uri);
        member
    }

    pub fn member_collection_labeled(
        &mut self,
        uri: IRI,
        ordered: bool,
        label: &str,
        language: Option<&str>,
    ) -> Self {
        let mut member = Self::new_with_label(uri.clone(), label, language);
        member.set_ordered(ordered);
        self.add_member(uri);
        member
    }

    pub fn is_ordered(&self) -> bool {
        self.ordered
    }

    pub fn set_ordered(&mut self, ordered: bool) {
        self.ordered = ordered;
    }

    pub fn add_member(&mut self, uri: IRI) {
        self.members.push(ObjectProperty::member(uri));
    }
    pub fn add_list_member(&mut self, uri: IRI) {
        self.members.push(ObjectProperty::member_list(uri));
    }
    pub fn has_members(&self) -> bool {
        !self.members.is_empty()
    }
    pub fn members(&self) -> impl Iterator<Item = &ObjectProperty> {
        self.members.iter()
    }
}

// ------------------------------------------------------------------------------------------------

impl ToStatement for LiteralProperty {
    fn to_statement(&self, subject: &SubjectNode) -> Statement {
        Statement::new(
            subject.clone(),
            self.predicate().clone(),
            self.value().clone().into(),
        )
    }
}

impl LiteralProperty {
    pub fn new(predicate: IRI, value: Literal) -> Self {
        Self { predicate, value }
    }

    pub fn preferred_label(text: &str) -> Self {
        Self::new(ns::pref_label(), text.into())
    }
    pub fn preferred_label_with(text: &str, language: &str) -> Self {
        Self::new(ns::pref_label(), Literal::with_language(text, language))
    }
    pub fn alternative_label(text: &str) -> Self {
        Self::new(ns::alt_label(), text.into())
    }
    pub fn alternative_label_with(text: &str, language: &str) -> Self {
        Self::new(ns::alt_label(), Literal::with_language(text, language))
    }
    pub fn hidden_label(text: &str) -> Self {
        Self::new(ns::hidden_label(), text.into())
    }
    pub fn hidden_label_with(text: &str, language: &str) -> Self {
        Self::new(ns::hidden_label(), Literal::with_language(text, language))
    }

    pub fn change_note(text: &str) -> Self {
        Self::new(ns::change_note(), text.into())
    }
    pub fn change_note_with(text: &str, language: &str) -> Self {
        Self::new(ns::change_note(), Literal::with_language(text, language))
    }
    pub fn definition(text: &str) -> Self {
        Self::new(ns::definition(), text.into())
    }
    pub fn definition_with(text: &str, language: &str) -> Self {
        Self::new(ns::definition(), Literal::with_language(text, language))
    }
    pub fn editorial_note(text: &str) -> Self {
        Self::new(ns::editorial_note(), text.into())
    }
    pub fn editorial_note_with(text: &str, language: &str) -> Self {
        Self::new(ns::editorial_note(), Literal::with_language(text, language))
    }
    pub fn example(text: &str) -> Self {
        Self::new(ns::example(), text.into())
    }
    pub fn example_with(text: &str, language: &str) -> Self {
        Self::new(ns::example(), Literal::with_language(text, language))
    }
    pub fn history_note(text: &str) -> Self {
        Self::new(ns::history_note(), text.into())
    }
    pub fn history_note_with(text: &str, language: &str) -> Self {
        Self::new(ns::history_note(), Literal::with_language(text, language))
    }
    pub fn note(text: &str) -> Self {
        Self::new(ns::note(), text.into())
    }
    pub fn note_with(text: &str, language: &str) -> Self {
        Self::new(ns::note(), Literal::with_language(text, language))
    }
    pub fn scope_note(text: &str) -> Self {
        Self::new(ns::scope_note(), text.into())
    }
    pub fn scope_note_with(text: &str, language: &str) -> Self {
        Self::new(ns::scope_note(), Literal::with_language(text, language))
    }

    pub fn notation(text: &str) -> Self {
        Self::new(ns::notation(), Literal::new(text))
    }

    pub fn created(text: &str) -> Self {
        Self::new(dc::terms::created(), Literal::new(text))
    }
    pub fn creator(text: &str) -> Self {
        Self::new(dc::terms::creator(), Literal::new(text))
    }
    pub fn description(text: &str) -> Self {
        Self::new(dc::terms::description(), Literal::new(text))
    }
    pub fn issued(text: &str) -> Self {
        Self::new(dc::terms::issued(), Literal::new(text))
    }
    pub fn modified(text: &str) -> Self {
        Self::new(dc::terms::modified(), Literal::new(text))
    }
    pub fn publisher(text: &str) -> Self {
        Self::new(dc::terms::publisher(), Literal::new(text))
    }
    pub fn rights(text: &str) -> Self {
        Self::new(dc::terms::rights(), Literal::new(text))
    }
    pub fn subject(text: &str) -> Self {
        Self::new(dc::terms::subject(), Literal::new(text))
    }
    pub fn title(text: &str) -> Self {
        Self::new(dc::terms::title(), Literal::new(text))
    }

    pub fn predicate(&self) -> &IRI {
        &self.predicate
    }

    pub fn value(&self) -> &Literal {
        &self.value
    }
}

// ------------------------------------------------------------------------------------------------

impl ToStatement for ObjectProperty {
    fn to_statement(&self, subject: &SubjectNode) -> Statement {
        Statement::new(
            subject.clone(),
            self.predicate().clone(),
            self.other().clone().into(),
        )
    }
}

impl ObjectProperty {
    pub fn new(predicate: IRI, other: IRI) -> Self {
        Self { predicate, other }
    }

    pub fn broader(other: IRI) -> Self {
        Self::new(ns::broader(), other)
    }
    pub fn transitively_broader(other: IRI) -> Self {
        Self::new(ns::broader_transitive(), other)
    }
    pub fn narrower(other: IRI) -> Self {
        Self::new(ns::narrower(), other)
    }
    pub fn transitively_narrower(other: IRI) -> Self {
        Self::new(ns::narrower_transitive(), other)
    }
    pub fn related_to(other: IRI) -> Self {
        Self::new(ns::related(), other)
    }

    pub fn broad_match(other: IRI) -> Self {
        Self::new(ns::broad_match(), other)
    }
    pub fn close_match(other: IRI) -> Self {
        Self::new(ns::close_match(), other)
    }
    pub fn exact_match(other: IRI) -> Self {
        Self::new(ns::exact_match(), other)
    }
    pub fn narrow_match(other: IRI) -> Self {
        Self::new(ns::narrow_match(), other)
    }
    pub fn related_match(other: IRI) -> Self {
        Self::new(ns::related_match(), other)
    }

    pub fn member(other: IRI) -> Self {
        Self::new(ns::member(), other)
    }
    pub fn member_list(other: IRI) -> Self {
        Self::new(ns::member_list(), other)
    }

    // ISO relationships
    pub fn broader_generic(other: IRI) -> Self {
        Self::new(ns::iso::broader_generic(), other)
    }
    pub fn broader_instantial(other: IRI) -> Self {
        Self::new(ns::iso::broader_instantial(), other)
    }
    pub fn broader_partitive(other: IRI) -> Self {
        Self::new(ns::iso::broader_partitive(), other)
    }
    pub fn narrower_generic(other: IRI) -> Self {
        Self::new(ns::iso::narrower_generic(), other)
    }
    pub fn narrower_instantial(other: IRI) -> Self {
        Self::new(ns::iso::narrower_instantial(), other)
    }
    pub fn narrower_partitive(other: IRI) -> Self {
        Self::new(ns::iso::narrower_partitive(), other)
    }

    pub fn predicate(&self) -> &IRI {
        &self.predicate
    }

    pub fn other(&self) -> &IRI {
        &self.other
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------
