//! JUNK
//! JUNK
//! JUNK
//! JUNK
//!
//!
//!
//! # Requirements
//!
//! ## Definitions
//!
//! * buffer index: 1-based index of lines from the underlying buffer
//!   (i.e., original file)
//! * return index: 1-based index of requested line; lines corresponding to a
//!   given return index could correspond to a matched line, a context line,
//!   or a context gap; thus in the case of context gaps, the returned value
//!   may not correspond to a line in the actual file. Also return indexes need
//!   not match the buffer indexes when filters are applied.
//!
//! * without a filter:
//!   * given a return index, return the line from the underlying buffer
//! * with a filter:
//!   * given a return index, return a line subject to filtering. The retuned
//!     line may be a match, a context line, or a gap in the results.
//!
//! ## Example
//!
//! Filter on: foobar, context lines: 1
//!
//! Underlying buffer:
//!
//! buf idx     line
//! ----------------
//! 1           a
//! 2           foobar
//! 3           a
//! 4           foobar
//! 5           a
//! 6           a
//! 7           a
//! 8           a
//! 9           foobar
//! 10          a
//! 11          a
//!
//! ret idx     buf idx     line    type
//! ------------------------------------
//! 1           1           a       ctx
//! 2           2           foobar  match
//! 3           3           a       ctx
//! 4           4           foobar  match
//! 5           5           a       ctx
//! 6           ---         ---     gap
//! 7           8           a       ctx
//! 8           9           foobar  match
//! 9           11          a       ctx
//! 10          ---         ---     gap
//!
//! match_cache:
//!
//! 2  -> (2, foobar)
//! 4  -> (4, foobar)
//! 9  -> (9, foobar)
//!
//! ##################################################
//! ##################################################
//! ##################################################
//!
//! buf idx     line
//! ----------------
//! 1           a
//! 2           foobar
//! 3           a
//! 4           a
//! 5           a
//! 6           a
//! 7           a
//! 8           a
//! 9           a
//! 10          a
//! 11          a
//! 12          a
//! 13          a
//! 14          a
//! 15          a
//! 16          a
//! 17          a
//! 18          a
//! 19          a
//! 20          foobar
//! 21          a
//! 22          a
//! 23          a
//! 24          foobar
//!
//! ret idx     buf idx     line    type
//! ------------------------------------
//! 1           1           a       ctx
//! 2           2           foobar  match
//! 3           3           a       ctx
//! 4           ---         ---     gap
//! 5           19          a       ctx
//! 6           20          foobar  match
//! 7           21          a       ctx
//! 8           ---         ---     gap
//! 9           23          a       ctx
//! 10          24          foobar  match
//!
//!
//!
//! get(ret_idx = 8):
//!
//!
//!
//! ## Notes
//! * if context_lines = 0, you just search the match_cache for matches
//! * if every buffer line contains a match, ret idx = buf idx
//!
//! ## Questions
//! * If we know where all the matches are, can we determine in constant time
//!   which lines are gaps and context?
//! * given context_lines and a ret_idx, what's the maximum/minimum number of
//!   matches that could already have occurred?
//!   * maximum = ret_idx
//!   * minimum = round(ret_idx / (context_lines * 2 + 2))
//! * given context_lines and a ret_idx, what's the maximum/minimum buffer idx
//!   of possible matches?
//!   * maximum = buffer idx of ret_idx'th match - context_lines (unlimited)
//!   * minimum = ret_idx (match on every line)
//!


use std::collections::BTreeMap;
use std::io::BufRead;

use iter::{FilterPredicate, LineBuffer, NumberedLine};


fn invert(arg: bool) -> bool {
    return !arg;
}



#[cfg(test)]
mod test {

    use super::invert;

    #[test]
    fn test1() {
        assert!(invert(true) == false);
    }
}
