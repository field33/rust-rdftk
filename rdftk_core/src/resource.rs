/*!
Implementation of the `Resource` pattern as a kind of statement builder. As a builder type the
interface is only additive, no update or remove methods exist.

Each `Resource` comprises a common subject value and a set of predicates each of which may have
multiple associated values. It should therefore be noted that calling a predicate method will add a
new value, and **not** replace any existing value. New resource instances can be used directly as
the object of a predicate and so _nested_ or _related_ resources can be written inline. This is
particulary useful where the object is a blank node.

Additionally a `Predicate` builder is provided where it is more readable to add only the values
individually rather than repeating the same predicate IRI as the `Resource` interface requires. The
interface for `Predicate` is intended to be a precise sub-set of the `Resource` methods so that the
same look and feel is maintained.

# Example

The following short example shows the use of `Resource`, and a nested resource, to build a small
RDF model. Once a resource is created it can be converted into a vector of `Statement`s for either
writing out or constructing a `Graph` instance.

```rust
use rdftk_core::{Literal, Resource, Statement};
use rdftk_iri::IRI;
use rdftk_names::{dc::elements as dc, foaf, rdf};
use std::str::FromStr;

fn contact(name: &str) -> IRI {
    IRI::from_str(&format!(
        "http://www.w3.org/2000/10/swap/pim/contact#{}",
        name
    ))
    .unwrap()
}

let resource =
    Resource::named(IRI::from_str("http://en.wikipedia.org/wiki/Tony_Benn").unwrap().into())
        .value_of(dc::publisher().clone(), Literal::new("Wikipedia"))
        .value_of(dc::title().clone(), Literal::new("Tony Benn"))
        .resource(
            dc::description().clone(),
            Resource::blank()
                .resource_named(rdf::a_type().clone(), foaf::person().clone())
                .value_of(foaf::name().clone(), Literal::new("Tony Benn"))
                .to_owned(),
        )
        .to_owned();

let sts: Vec<Statement> = resource.into();
assert_eq!(sts.len(), 5);

for st in sts {
    println!("{}", st);
}
```

The output from the example above will look like this, although blank node identifiers _will_
likely be different.

```text
<http://en.wikipedia.org/wiki/Tony_Benn> <http://purl.org/dc/elements/1.1/title> "Tony Benn" .
_:B1 <http://xmlns.com/foaf/0.1/name> "Tony Benn" .
_:B1 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://xmlns.com/foaf/0.1/Person> .
<http://en.wikipedia.org/wiki/Tony_Benn> <http://purl.org/dc/elements/1.1/description> _:B1 .
<http://en.wikipedia.org/wiki/Tony_Benn> <http://purl.org/dc/elements/1.1/publisher> "Wikipedia" .
```

*/

use crate::{Literal, Statement, SubjectNode};
use rdftk_iri::IRIRef;
use rdftk_names::rdf;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// The main resource builder type.
///
#[derive(Clone, Debug)]
pub struct Resource {
    subject: SubjectNode,
    predicates: HashMap<IRIRef, RefCell<Vec<ResourceObject>>>,
}

///
/// A predicate builder type, optionally used for multi-valued predicates.
///
#[derive(Clone, Debug)]
pub struct Predicate {
    name: IRIRef,
    objects: Vec<ResourceObject>,
}

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
enum ResourceObject {
    Resource(Resource),
    Resources(Container<Resource>),
    Literal(Literal),
    Literals(Container<Literal>),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
enum ContainerKind {
    Alt,
    Bag,
    Seq,
    Other(IRIRef),
}

#[derive(Clone, Debug)]
struct Container<T> {
    kind: ContainerKind,
    values: Vec<T>,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Predicate {
    ///
    /// Construct a new Predicate instance with the provided `IRI` name.
    ///
    pub fn new(name: IRIRef) -> Self {
        Self {
            name,
            objects: Default::default(),
        }
    }

    // --------------------------------------------------------------------------------------------

    ///
    /// Add a new literal value to this predicate.
    ///
    pub fn property(&mut self, value: Literal) -> &mut Self {
        self.objects.push(ResourceObject::Literal(value));
        self
    }

    ///
    /// Add a new literal value to this predicate.
    ///
    pub fn value_of(&mut self, value: Literal) -> &mut Self {
        self.objects.push(ResourceObject::Literal(value));
        self
    }

    ///
    /// Add a new literal value to this predicate.
    ///
    pub fn literal(&mut self, value: Literal) -> &mut Self {
        self.objects.push(ResourceObject::Literal(value));
        self
    }

    // --------------------------------------------------------------------------------------------

    pub fn property_alternatives(&mut self, values: &[Literal]) -> &mut Self {
        self.objects.push(ResourceObject::Literals(Container {
            kind: ContainerKind::Alt,
            values: values.to_vec(),
        }));
        self
    }

    pub fn property_bag(&mut self, values: &[Literal]) -> &mut Self {
        self.objects.push(ResourceObject::Literals(Container {
            kind: ContainerKind::Bag,
            values: values.to_vec(),
        }));
        self
    }

    pub fn property_sequence(&mut self, values: &[Literal]) -> &mut Self {
        self.objects.push(ResourceObject::Literals(Container {
            kind: ContainerKind::Seq,
            values: values.to_vec(),
        }));
        self
    }

    pub fn property_container(&mut self, values: &[Literal], kind: IRIRef) -> &mut Self {
        self.objects.push(ResourceObject::Literals(Container {
            kind: ContainerKind::Other(kind),
            values: values.to_vec(),
        }));
        self
    }

    // --------------------------------------------------------------------------------------------

    pub fn resource_blank_named(&mut self, name: &str) -> &mut Self {
        self.objects
            .push(ResourceObject::Resource(Resource::blank_named(name)));
        self
    }
    pub fn resource_named(&mut self, name: IRIRef) -> &mut Self {
        self.objects
            .push(ResourceObject::Resource(Resource::named(name)));
        self
    }
    pub fn resource(&mut self, resource: Resource) -> &mut Self {
        self.objects.push(ResourceObject::Resource(resource));
        self
    }

    // --------------------------------------------------------------------------------------------

    pub fn resource_alternatives(&mut self, values: &[Resource]) -> &mut Self {
        self.objects.push(ResourceObject::Resources(Container {
            kind: ContainerKind::Alt,
            values: values.to_vec(),
        }));
        self
    }

    pub fn resource_bag(&mut self, values: &[Resource]) -> &mut Self {
        self.objects.push(ResourceObject::Resources(Container {
            kind: ContainerKind::Bag,
            values: values.to_vec(),
        }));
        self
    }

    pub fn resource_sequence(&mut self, values: &[Resource]) -> &mut Self {
        self.objects.push(ResourceObject::Resources(Container {
            kind: ContainerKind::Seq,
            values: values.to_vec(),
        }));
        self
    }

    pub fn resource_container(&mut self, values: &[Resource], kind: IRIRef) -> &mut Self {
        self.objects.push(ResourceObject::Resources(Container {
            kind: ContainerKind::Other(kind),
            values: values.to_vec(),
        }));
        self
    }
}

// ------------------------------------------------------------------------------------------------

impl Into<Vec<Statement>> for Resource {
    fn into(self) -> Vec<Statement> {
        let mut sts = Vec::default();
        flatten(&self, &mut sts);
        sts
    }
}

impl Into<Vec<Rc<Statement>>> for Resource {
    fn into(self) -> Vec<Rc<Statement>> {
        let sts: Vec<Statement> = self.into();
        sts.iter().cloned().map(Rc::new).collect()
    }
}

impl Resource {
    ///
    /// Construct a new `Resource` with the subject cloned from an existing
    /// [`SubjectNode`](../statement/struct.SubjectNode.html).
    ///
    pub fn new(subject: SubjectNode) -> Self {
        Self {
            subject,
            predicates: Default::default(),
        }
    }

    ///
    /// Construct a new `Resource` with a new blank node as the subject.
    ///
    pub fn blank() -> Self {
        Self {
            subject: SubjectNode::blank(),
            predicates: Default::default(),
        }
    }

    ///
    /// Construct a new `Resource` with a named blank node as the subject.
    ///
    pub fn blank_named(name: &str) -> Self {
        Self {
            subject: SubjectNode::blank_named(name),
            predicates: Default::default(),
        }
    }

    ///
    /// Construct a new `Resource` with the provided `IRI` as the subject.
    ///
    pub fn named(name: IRIRef) -> Self {
        Self {
            subject: SubjectNode::named(name),
            predicates: Default::default(),
        }
    }

    // --------------------------------------------------------------------------------------------

    ///
    /// Returns `true` if this instance is a resource in web terms, that is it's subject is an `IRI`.
    ///
    #[inline]
    pub fn is_a_resource(&self) -> bool {
        self.subject.is_iri()
    }

    ///
    /// Returns `true` if this instance is an individual in RDFS terms, that is it has at least one
    /// `rdf:type` predicate.
    ///
    #[inline]
    pub fn is_an_individual(&self) -> bool {
        self.predicates.keys().any(|p| p == rdf::a_type())
    }

    // --------------------------------------------------------------------------------------------

    ///
    /// Add the `Predicate` instance to this resource.
    ///
    pub fn predicate(&mut self, predicate: Predicate) -> &mut Self {
        for object in predicate.objects.into_iter() {
            self.insert(predicate.name.clone(), object);
        }
        self
    }

    // --------------------------------------------------------------------------------------------

    ///
    /// Add a new predicate with a literal value to this resource.
    ///
    pub fn property(&mut self, predicate: IRIRef, value: Literal) -> &mut Self {
        self.literal(predicate, value)
    }

    ///
    /// Add a new predicate with a literal value to this resource.
    ///
    pub fn value_of(&mut self, predicate: IRIRef, value: Literal) -> &mut Self {
        self.literal(predicate, value)
    }

    ///
    /// Add a new predicate with a literal value to this resource.
    ///
    pub fn literal(&mut self, predicate: IRIRef, value: Literal) -> &mut Self {
        self.insert(predicate, ResourceObject::Literal(value))
    }

    // --------------------------------------------------------------------------------------------

    pub fn property_alternatives(&mut self, predicate: IRIRef, values: &[Literal]) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Literals(Container {
                kind: ContainerKind::Alt,
                values: values.to_vec(),
            }),
        )
    }

    pub fn property_bag(&mut self, predicate: IRIRef, values: &[Literal]) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Literals(Container {
                kind: ContainerKind::Bag,
                values: values.to_vec(),
            }),
        )
    }

    pub fn property_sequence(&mut self, predicate: IRIRef, values: &[Literal]) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Literals(Container {
                kind: ContainerKind::Seq,
                values: values.to_vec(),
            }),
        )
    }

    pub fn property_container(
        &mut self,
        predicate: IRIRef,
        values: &[Literal],
        kind: IRIRef,
    ) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Literals(Container {
                kind: ContainerKind::Other(kind),
                values: values.to_vec(),
            }),
        )
    }

    // --------------------------------------------------------------------------------------------

    pub fn resource_blank_named(&mut self, predicate: IRIRef, name: &str) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Resource(Resource::blank_named(name)),
        )
    }
    pub fn resource_named(&mut self, predicate: IRIRef, name: IRIRef) -> &mut Self {
        self.insert(predicate, ResourceObject::Resource(Resource::named(name)))
    }
    pub fn resource(&mut self, predicate: IRIRef, resource: Resource) -> &mut Self {
        self.insert(predicate, ResourceObject::Resource(resource))
    }

    // --------------------------------------------------------------------------------------------

    pub fn resource_alternatives(&mut self, predicate: IRIRef, values: &[Resource]) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Resources(Container {
                kind: ContainerKind::Alt,
                values: values.to_vec(),
            }),
        )
    }

    pub fn resource_bag(&mut self, predicate: IRIRef, values: &[Resource]) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Resources(Container {
                kind: ContainerKind::Bag,
                values: values.to_vec(),
            }),
        )
    }

    pub fn resource_sequence(&mut self, predicate: IRIRef, values: &[Resource]) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Resources(Container {
                kind: ContainerKind::Seq,
                values: values.to_vec(),
            }),
        )
    }

    pub fn resource_container(
        &mut self,
        predicate: IRIRef,
        values: &[Resource],
        kind: IRIRef,
    ) -> &mut Self {
        self.insert(
            predicate,
            ResourceObject::Resources(Container {
                kind: ContainerKind::Other(kind),
                values: values.to_vec(),
            }),
        )
    }

    // --------------------------------------------------------------------------------------------

    ///
    /// Set the RDF type (classifier) of this resource.
    ///
    pub fn instance_of(&mut self, name: IRIRef) -> &mut Self {
        self.insert(
            rdf::a_type().clone(),
            ResourceObject::Resource(Resource::named(name)),
        )
    }

    // --------------------------------------------------------------------------------------------

    fn insert(&mut self, predicate: IRIRef, object: ResourceObject) -> &mut Self {
        if !self.predicates.contains_key(&predicate) {
            self.predicates
                .insert(predicate.clone(), RefCell::default());
        }
        let values = self.predicates.get(&predicate).unwrap();
        values.borrow_mut().push(object);
        self
    }
}

// ------------------------------------------------------------------------------------------------

impl ResourceObject {
    pub fn is_container(&self) -> bool {
        matches!(self, ResourceObject::Resources(_) | ResourceObject::Literals(_))
    }
}

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

fn flatten(resource: &Resource, sts: &mut Vec<Statement>) {
    let subject = &resource.subject;
    for (predicate, objects) in &resource.predicates {
        let objects = objects.borrow();
        for object in objects.iter() {
            if object.is_container() {
                // <s> <p> {<kind>, [values]} becomes:
                //
                // <s> <p> _:b .
                // _b: rdf:type <kind>
                // _b: rdf:_1 value[1] .
                // _b: rdf:_n value[n] .
                let kind = match object {
                    ResourceObject::Resources(rc) => &rc.kind,
                    ResourceObject::Literals(lc) => &lc.kind,
                    _ => unreachable!(),
                };
                let container = SubjectNode::blank();
                sts.push(Statement::new(
                    subject.clone(),
                    predicate.clone(),
                    container.clone().into(),
                ));
                sts.push(Statement::new(
                    container.clone(),
                    rdf::a_type().clone(),
                    match kind {
                        ContainerKind::Alt => rdf::alt(),
                        ContainerKind::Bag => rdf::bag(),
                        ContainerKind::Seq => rdf::seq(),
                        ContainerKind::Other(iri) => iri,
                    }
                    .into(),
                ));

                match object {
                    ResourceObject::Resources(rc) => {
                        for (index, resource) in rc.values.iter().enumerate() {
                            flatten(resource, sts);
                            sts.push(Statement::new(
                                container.clone(),
                                rdf::member(index),
                                resource.subject.clone().into(),
                            ));
                        }
                    }
                    ResourceObject::Literals(lc) => {
                        for (index, literal) in lc.values.iter().enumerate() {
                            sts.push(Statement::new(
                                container.clone(),
                                rdf::member(index),
                                literal.clone().into(),
                            ));
                        }
                    }
                    _ => unreachable!(),
                };
            } else {
                let statement = Statement::new(
                    subject.clone(),
                    predicate.clone(),
                    match object {
                        ResourceObject::Resource(resource) => {
                            flatten(resource, sts);
                            resource.subject.clone().into()
                        }
                        ResourceObject::Literal(literal) => literal.clone().into(),
                        _ => unreachable!(),
                    },
                );
                sts.push(statement);
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Unit Tests
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rdftk_iri::IRI;
    use std::str::FromStr;

    fn contact(name: &str) -> IRIRef {
        IRI::from_str(&format!(
            "http://www.w3.org/2000/10/swap/pim/contact#{}",
            name
        ))
        .unwrap()
        .into()
    }

    #[test]
    fn test_wikipedia_example_01() {
        let resource = Resource::named(
            IRI::from_str("http://www.w3.org/People/EM/contact#me")
                .unwrap()
                .into(),
        )
        .literal(contact("fullName"), "Eric Miller".into())
        .resource_named(
            contact("mailbox"),
            IRI::from_str("mailto:e.miller123(at)example")
                .unwrap()
                .into(),
        )
        .literal(contact("personalTitle"), "Dr.".into())
        .instance_of(contact("Person"))
        .to_owned();
        let sts: Vec<Statement> = resource.into();
        assert_eq!(sts.len(), 4);
        for st in sts {
            println!("{}", st);
        }
    }
}
