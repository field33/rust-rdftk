/*!
Provides the `TurtleWriter` implementation of the `GraphWriter` trait.

# Example

```rust
use rdftk_io::turtle::writer::{TurtleOptions, TurtleWriter};
use rdftk_io::write_graph_to_string;
use rdftk_iri::{IRIRef, IRI};
use std::str::FromStr;
# use rdftk_memgraph::MemGraph;
# fn make_graph() -> MemGraph { MemGraph::default() }

let mut options = TurtleOptions::default();
options.use_sparql_style = true;
options.nest_blank_nodes = false;
let writer = TurtleWriter::with_base(
    IRIRef::from(IRI::from_str("http://en.wikipedia.org/wiki/").unwrap()),
    options,
);

let result = write_graph_to_string(&writer, &make_graph());
```

*/

use crate::common::Indenter;
use crate::GraphWriter;
use rdftk_core::graph::{Graph, Prefix, PrefixMappings};
use rdftk_core::{Literal, SubjectNode};
use rdftk_iri::IRIRef;
use std::io::Write;
use std::rc::Rc;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct TurtleOptions {
    pub nest_blank_nodes: bool,
    pub use_sparql_style: bool,
}

#[derive(Debug)]
pub struct TurtleWriter {
    base: Option<String>,
    options: TurtleOptions,
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Default for TurtleOptions {
    fn default() -> Self {
        Self {
            nest_blank_nodes: true,
            use_sparql_style: false,
        }
    }
}

// ------------------------------------------------------------------------------------------------

impl Default for TurtleWriter {
    fn default() -> Self {
        Self {
            base: None,
            options: Default::default(),
        }
    }
}

impl GraphWriter for TurtleWriter {
    fn write(&self, w: &mut impl Write, graph: &impl Graph) -> crate::error::Result<()> {
        //
        // Write out the graph base IRI
        //
        if let Some(base) = &self.base {
            if self.options.use_sparql_style {
                writeln!(w, "BASE <{}>", base)?;
            } else {
                writeln!(w, "@base <{}> .", base)?;
            }
        }
        writeln!(w)?;
        //
        // Write all prefix mappings
        //
        let mappings = graph.prefix_mappings();
        for prefix in mappings.prefixes() {
            let namespace = mappings.get_namespace(prefix).unwrap();
            let prefix = match prefix {
                Prefix::Default => String::new(),
                Prefix::Some(prefix) => prefix.clone(),
            };
            if self.options.use_sparql_style {
                writeln!(w, "PREFIX {}: <{}>", prefix, namespace)?;
            } else {
                writeln!(w, "@prefix {}: <{}> .", prefix, namespace)?;
            }
        }
        writeln!(w)?;
        //
        // Write statements, start with those where subject is an IRI
        //
        let mut blanks_to_write: Vec<&SubjectNode> = Default::default();
        let mut blanks_written: Vec<SubjectNode> = Default::default();
        for subject in graph.subjects() {
            if subject.is_blank() {
                blanks_to_write.push(subject);
            } else {
                let mut inner_written =
                    self.write_sub_graph(w, subject, graph, Indenter::default())?;
                blanks_written.append(&mut inner_written);
            }
            writeln!(w)?;
        }
        //
        // Write statements where subject is a blank node
        //
        blanks_to_write.retain(|subject| !blanks_written.contains(subject));
        for subject in blanks_to_write {
            self.write_sub_graph(w, subject, graph, Indenter::default())?;
        }
        Ok(())
    }
}

impl TurtleWriter {
    ///
    /// Create a new writer with the provided options, this is used to override the default
    /// options that are used when calling `Default::default`.
    ///
    pub fn new(options: TurtleOptions) -> Self {
        Self {
            base: None,
            options,
        }
    }
    pub fn with_base(base: IRIRef, options: TurtleOptions) -> Self {
        Self {
            base: Some(base.to_string()),
            options,
        }
    }

    fn write_sub_graph(
        &self,
        w: &mut impl Write,
        subject: &SubjectNode,
        in_graph: &impl Graph,
        indenter: Indenter,
    ) -> std::io::Result<Vec<SubjectNode>> {
        write!(w, "{}", indenter)?;
        let mut indenter = indenter;
        let mut blanks_written = Vec::default();
        let mappings = in_graph.prefix_mappings();
        if subject.is_blank() && indenter.depth() == 0 {
            write!(w, "_:{} ", subject.as_blank().unwrap())?;
        } else if subject.is_iri() {
            self.write_iri(w, subject.as_iri().unwrap(), &mappings)?;
        }
        let predicates = in_graph.predicates_for(subject);
        indenter = indenter.indent();
        let mut p_iter = predicates.iter().peekable();
        while let Some(predicate) = p_iter.next() {
            self.write_iri(w, predicate, &mappings)?;
            let objects = in_graph.objects_for(subject, predicate);
            if objects.len() > 1 {
                indenter = indenter.indent();
            }
            let mut o_iter = objects.iter().peekable();
            while let Some(object) = o_iter.next() {
                if object.is_blank() && self.options.nest_blank_nodes {
                    write!(w, "[\n{}", indenter.one())?;
                    let inner_subject = object.as_subject().unwrap();
                    let mut inner_written =
                        self.write_sub_graph(w, &inner_subject, in_graph, indenter.clone())?;
                    blanks_written.push(inner_subject);
                    blanks_written.append(&mut inner_written);
                    write!(w, "{}]", indenter)?;
                } else if object.is_blank() && !self.options.nest_blank_nodes {
                    write!(w, "_:{}", object.as_blank().unwrap())?;
                } else if object.is_iri() {
                    self.write_iri(w, object.as_iri().unwrap(), &mappings)?;
                } else {
                    self.write_literal(w, object.as_literal().unwrap(), &mappings)?;
                }
                if o_iter.peek().is_some() {
                    writeln!(w, ",")?;
                }
            }
            if p_iter.peek().is_some() {
                write!(w, ";\n{}", indenter)?;
            }
            if objects.len() > 1 {
                indenter = indenter.outdent();
            }
        }
        indenter = indenter.outdent();
        if indenter.depth() == 0 {
            writeln!(w, ".")?;
        } else {
            writeln!(w)?;
        }
        Ok(blanks_written)
    }

    fn write_iri<W: Write>(
        &self,
        w: &mut W,
        iri: &IRIRef,
        mappings: &Rc<dyn PrefixMappings>,
    ) -> std::io::Result<()> {
        if let Some(base) = &self.base {
            let iri = iri.to_string();
            if iri.starts_with(base) {
                return write!(w, "<{}> ", &iri[base.len()..]);
            }
        }
        write!(
            w,
            "{} ",
            match mappings.compress(&iri) {
                None => format!("<{}>", iri),
                Some(qname) => qname.to_string(),
            }
        )
    }

    fn write_literal<W: Write>(
        &self,
        w: &mut W,
        literal: &Literal,
        _mappings: &Rc<dyn PrefixMappings>,
    ) -> std::io::Result<()> {
        // TODO: compress data type IRIs
        write!(w, "{} ", literal)
    }
}