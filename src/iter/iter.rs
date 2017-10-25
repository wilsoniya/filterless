use std::fmt;

/// Parameters used when creating a filtering iterator
#[derive(Clone)]
pub struct FilterPredicate {
    /// Search string which must be included in a line to be considered a match
    pub filter_string: String,
    /// Number of non-match lines above and below a match line to include in
    /// the lines returned by the iterator
    pub context_lines: usize ,
}

pub type NumberedLine = (usize, String);

/// Representation of a line that might be returned from a filtering iterator.
#[derive(Clone, Debug, PartialEq)]
pub enum FilteredLine {
    /// a gap between context groups (i.e., groups of context lines
    /// corresponding to distinct match lines)
    Gap,
    /// a line which provides context before or after a matched line
    ContextLine(NumberedLine),
    /// a line matched by a filter string
    MatchLine(NumberedLine),
    /// a line emitted when no filter predicate is in use
    UnfilteredLine(NumberedLine),
}

#[derive(Clone, Debug)]
/// Representation of a line returned from a ContextBuffer.
pub enum ContextLine {
    /// the line matched a given filter string
    Match(NumberedLine),
    /// the line did not match the filter string
    NoMatch(NumberedLine),
}

/// Representation of an iterator's encounter with a context gap.
pub enum Gap {
    /// When the current value in the iterator would produce a context gap
    Current,
    /// When the previous value in the iterator produced a context gap
    Previous,
    /// When a context gap has not been produced in the past two iterations
    None,
}

impl ContextLine {
    /// Creates a `ContextLine` instance by consuming a `NumberedLine`.
    pub fn from_numbered_line(numbered_line: NumberedLine, filter_string: &String) -> ContextLine {
        if numbered_line.1.contains(filter_string) {
            ContextLine::Match(numbered_line)
        } else {
            ContextLine::NoMatch(numbered_line)
        }
    }

    /// Creates a `FilteredLine` by cloning the inner `NumberedLine`.
    pub fn to_filtered_line(&self, pred: &Option<FilterPredicate>) -> FilteredLine {
        match self {
            &ContextLine::Match(ref numbered_line) => {
                FilteredLine::MatchLine(numbered_line.to_owned())
            },
            &ContextLine::NoMatch(ref numbered_line) => {
                match pred {
                    &Some(_) => FilteredLine::ContextLine(numbered_line.to_owned()),
                    &None => FilteredLine::UnfilteredLine(numbered_line.to_owned()),
                }
            },
        }
    }

}

impl fmt::Display for FilteredLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &FilteredLine::Gap => {
                write!(f, "-----")
            },
            &FilteredLine::ContextLine((line_num, ref line)) => {
                write!(f, "C {:05}: {}", line_num, line)
            },
            &FilteredLine::MatchLine((line_num, ref line)) => {
                write!(f, "M {:05}: {}", line_num, line)
            },
            &FilteredLine::UnfilteredLine((line_num, ref line)) => {
                write!(f, "U {:05}: {}", line_num, line)
            },
        }
    }
}
