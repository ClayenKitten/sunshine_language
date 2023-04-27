use crate::{input_stream::Location, source::SourceId};

/// Location in code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub source: Option<SourceId>,
    pub start: Location,
    pub end: Location,
}
