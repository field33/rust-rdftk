/*!
One-line description.

More detailed description, with

# Example

*/

#![allow(clippy::upper_case_acronyms)] // << generated by pest.

use crate::common::parser_error::ParserErrorFactory;
use pest::iterators::Pair;
use pest::Parser;
use rdftk_core::error::{ErrorKind, Result};
use rdftk_core::model::graph::{GraphFactoryRef, GraphRef};
use rdftk_core::model::literal::{DataType, LanguageTag, LiteralFactoryRef, LiteralRef};
use rdftk_core::model::statement::{
    ObjectNodeRef, StatementFactoryRef, StatementRef, SubjectNodeRef,
};
use rdftk_iri::{IRIRef, IRI};
use regex::Regex;
use std::str::FromStr;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Parser)]
#[grammar = "nt/nt.pest"]
struct NTripleParser;

// ------------------------------------------------------------------------------------------------
// Private Types
// ------------------------------------------------------------------------------------------------

const ERROR: ParserErrorFactory = ParserErrorFactory { repr: super::NAME };

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

pub(super) fn parse_graph(input: &str, factory: GraphFactoryRef) -> Result<GraphRef> {
    let mut parsed = NTripleParser::parse(Rule::ntriplesDoc, input).map_err(|e| ERROR.parser(e))?;
    let top_node = parsed.next().unwrap();
    ntriples_doc(top_node, factory)
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
// Private Functions
// ------------------------------------------------------------------------------------------------

fn ntriples_doc(input_pair: Pair<'_, Rule>, factory: GraphFactoryRef) -> Result<GraphRef> {
    trace!("ntriples_doc({:?})", &input_pair.as_rule());

    let graph = factory.graph();

    if input_pair.as_rule() == Rule::ntriplesDoc {
        for inner_pair in input_pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::triple => {
                    let mut graph = graph.borrow_mut();
                    let st = triple(
                        inner_pair,
                        &graph.statement_factory(),
                        &graph.literal_factory(),
                    )?;
                    graph.insert(st);
                }
                Rule::EOI => {
                    trace!("Done.")
                }
                _ => {
                    unexpected!("ntriples_doc", inner_pair)
                }
            }
        }
    } else {
        unexpected!("ntriples_doc", input_pair);
    }

    Ok(graph)
}

fn triple(
    input_pair: Pair<'_, Rule>,
    statements: &StatementFactoryRef,
    literals: &LiteralFactoryRef,
) -> Result<StatementRef> {
    trace!("triple({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::triple {
        let mut inner_pairs = input_pair.into_inner();
        let subject = subject(inner_pairs.next().unwrap(), statements)?;
        let predicate = predicate(inner_pairs.next().unwrap())?;
        let object = object(inner_pairs.next().unwrap(), statements, literals)?;
        statements.statement(subject, predicate, object)
    } else {
        unexpected!("triple", input_pair);
    }
}

fn subject(input_pair: Pair<'_, Rule>, factory: &StatementFactoryRef) -> Result<SubjectNodeRef> {
    trace!("subject({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::subject {
        let inner_pair = input_pair.into_inner().next().unwrap();
        match inner_pair.as_rule() {
            Rule::IRIREF => Ok(factory.named_subject(iri_ref(inner_pair)?)),
            Rule::BlankNode => {
                let node = inner_pair.as_str().to_string();
                // strip the leading '_:'
                let node = &node[2..];
                factory.blank_subject_named(node)
            }
            _ => {
                unexpected!("subject", inner_pair)
            }
        }
    } else {
        unexpected!("subject", input_pair);
    }
}

fn predicate(input_pair: Pair<'_, Rule>) -> Result<IRIRef> {
    trace!("predicate({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::predicate {
        let inner_pair = input_pair.into_inner().next().unwrap();
        if inner_pair.as_rule() == Rule::IRIREF {
            Ok(iri_ref(inner_pair)?)
        } else {
            unexpected!("subject", inner_pair);
        }
    } else {
        unexpected!("subject", input_pair);
    }
}

fn object(
    input_pair: Pair<'_, Rule>,
    factory: &StatementFactoryRef,
    literals: &LiteralFactoryRef,
) -> Result<ObjectNodeRef> {
    trace!("object({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::object {
        let inner_pair = input_pair.into_inner().next().unwrap();
        match inner_pair.as_rule() {
            Rule::IRIREF => Ok(factory.named_object(iri_ref(inner_pair)?)),
            Rule::BlankNode => {
                let node = inner_pair.as_str().to_string();
                // strip the leading '_:'
                let node = &node[2..];
                Ok(factory.blank_object_named(node)?)
            }
            Rule::literal => {
                let literal = literal(inner_pair, literals)?;
                Ok(factory.literal_object(literal))
            }
            _ => {
                unexpected!("object", inner_pair)
            }
        }
    } else {
        unexpected!("object", input_pair);
    }
}

fn literal(input_pair: Pair<'_, Rule>, literals: &LiteralFactoryRef) -> Result<LiteralRef> {
    trace!("literal({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::literal {
        let inner_pair = input_pair.into_inner().next().unwrap();
        rdf_literal(inner_pair, literals)
    } else {
        unexpected!("literal", input_pair);
    }
}

fn rdf_literal(input_pair: Pair<'_, Rule>, literals: &LiteralFactoryRef) -> Result<LiteralRef> {
    trace!("literal({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::rdfLiteral {
        let mut inner_pair = input_pair.into_inner();
        let lexical_form = string(inner_pair.next().unwrap())?;

        if let Some(other) = inner_pair.next() {
            match other.as_rule() {
                Rule::iri => {
                    let data_type = DataType::Other(iri(other)?);
                    Ok(literals.with_data_type(&lexical_form, data_type))
                }
                Rule::LANGTAG => {
                    let lang_tag = lang_tag(other)?;
                    Ok(literals.with_language(&lexical_form, lang_tag))
                }
                _ => {
                    unexpected!("literal", other);
                }
            }
        } else {
            Ok(literals.literal(&lexical_form))
        }
    } else {
        unexpected!("literal", input_pair);
    }
}

fn string(input_pair: Pair<'_, Rule>) -> Result<String> {
    trace!("string({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::String {
        let inner_pair = input_pair.into_inner().next().unwrap();
        match inner_pair.as_rule() {
            Rule::STRING_LITERAL_QUOTE => {
                let inner_pair = inner_pair.into_inner().next().unwrap();
                if inner_pair.as_rule() == Rule::QUOTE_INNER {
                    Ok(inner_pair.as_str().to_string())
                } else {
                    unexpected!("string", inner_pair);
                }
            }
            _ => {
                unexpected!("string", inner_pair)
            }
        }
    } else {
        unexpected!("string", input_pair);
    }
}

fn iri(input_pair: Pair<'_, Rule>) -> Result<IRIRef> {
    trace!("iri({:?})", &input_pair.as_rule());

    if input_pair.as_rule() == Rule::iri {
        let inner_pair = input_pair.into_inner().next().unwrap();
        if inner_pair.as_rule() == Rule::IRIREF {
            iri_ref(inner_pair)
        } else {
            unexpected!("iri", inner_pair);
        }
    } else {
        unexpected!("iri", input_pair);
    }
}

fn iri_ref(input_pair: Pair<'_, Rule>) -> Result<IRIRef> {
    trace!("iri_ref({:?})", &input_pair.as_rule());
    if input_pair.as_rule() == Rule::IRIREF {
        let iri = input_pair.as_str().to_string();
        // strip the '<' and '>' characters.
        let iri_str = unescape_iri(&iri[1..iri.len() - 1]);
        let iri = IRIRef::new(IRI::from_str(&iri_str)?);
        if !iri.is_relative_reference() {
            Ok(iri)
        } else {
            Err(ErrorKind::AbsoluteIriExpected(iri_str).into())
        }
    } else {
        unexpected!("iri_ref", input_pair);
    }
}

fn lang_tag(input_pair: Pair<'_, Rule>) -> Result<LanguageTag> {
    trace!("lang_tag({:?})", &input_pair.as_rule());
    if input_pair.as_rule() == Rule::LANGTAG {
        let tag = input_pair.as_str().to_string();
        // strip the leading '@'
        let tag = &tag[1..];
        Ok(LanguageTag::from_str(tag)?)
    } else {
        unexpected!("lang_tag", input_pair);
    }
}

// ------------------------------------------------------------------------------------------------

lazy_static! {
    static ref UNICODE_ESC: Regex =
        Regex::new(r"(\\U[[:xdigit:]]{8})|(\\u[[:xdigit:]]{4})").unwrap();
}

fn unescape_iri(iri: &str) -> String {
    let (new_iri, end) =
        UNICODE_ESC
            .captures_iter(iri)
            .fold((String::new(), 0), |(so_far, start), cap| {
                let cap = cap.get(0).unwrap();
                (
                    format!(
                        "{}{}{}",
                        so_far,
                        &iri[start..cap.start()],
                        unescape_uchar(cap.as_str())
                    ),
                    cap.end(),
                )
            });

    format!("{}{}", new_iri, &iri[end..])
}

fn unescape_uchar(uchar: &str) -> char {
    use std::char;
    let uchar = &uchar[2..];
    let uchar_u32 = u32::from_str_radix(uchar, 16).unwrap();
    char::from_u32(uchar_u32).unwrap()
}

// ------------------------------------------------------------------------------------------------
// Modules
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nt::writer::NTripleWriter;
    use crate::GraphWriter;
    use rdftk_core::simple::graph::graph_factory;

    fn write_graph(graph: &GraphRef) {
        let writer = NTripleWriter::default();
        let _ = writer.write(&mut std::io::stdout(), graph);
    }

    #[test]
    fn parse_simple() {
        let result: Result<GraphRef> = parse_graph(
            r###"
<http://example.org/show/218> <http://www.w3.org/2000/01/rdf-schema#label> "That Seventies Show"^^<http://www.w3.org/2001/XMLSchema#string> . # literal with XML Schema string datatype
<http://example.org/show/218> <http://www.w3.org/2000/01/rdf-schema#label> "That Seventies Show" . # same as above
<http://example.org/show/218> <http://example.org/show/localName> "That Seventies Show"@en . # literal with a language tag
<http://example.org/show/218> <http://example.org/show/localName> "Cette Série des Années Septante"@fr-be .  # literal outside of ASCII range with a region subtag
<http://example.org/#spiderman> <http://example.org/text> "This is a multi-line\nliteral with many quotes (\"\"\"\"\")\nand two apostrophes ('')." .
<http://en.wikipedia.org/wiki/Helium> <http://example.org/elements/atomicNumber> "2"^^<http://www.w3.org/2001/XMLSchema#integer> . # xsd:integer
<http://en.wikipedia.org/wiki/Helium> <http://example.org/elements/specificGravity> "1.663E-4"^^<http://www.w3.org/2001/XMLSchema#double> .     # xsd:double
"###,
            graph_factory(),
        );
        match result {
            Ok(g) => {
                println!("ok");
                write_graph(&g);
            }
            Err(e) => {
                println!("{:?}", e);
                panic!();
            }
        }
    }

    #[test]
    fn parse_simple_with_blanks() {
        let result: Result<GraphRef> = parse_graph(
            r###"
<http://one.example/subject1> <http://one.example/predicate1> <http://one.example/object1> . # comments here
# or on a line by themselves
_:subject1 <http://an.example/predicate1> "object1" .
_:subject2 <http://an.example/predicate2> "object2" .
"###,
            graph_factory(),
        );
        match result {
            Ok(g) => {
                println!("ok");
                write_graph(&g);
            }
            Err(e) => {
                println!("{:?}", e);
                panic!();
            }
        }
    }
}
