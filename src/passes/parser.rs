//! Parsing compilation passes

use crate::{event::Event, passes::CompilingPass};

use std::str::FromStr;

/// Parses a string slice into an Event vector.
/// For now, the grammar is defined as follow in EBNF
/// ```ebnf
/// line return = "\n" ;
///
/// new paragraph = line return , line return ;
///
/// S = { " " | "\n" | "\t" } ;
///
/// delimiter = "---" , line return ;
///
/// key = litteral ;
/// value = litteral ;
/// pair = S , key , S , ":" , value , S , line return ;
///
/// event header = { pair } ;
///
/// event description = (TODO) ;
///
/// event = event header , new paragraph , event description ;
///
/// timetable = event , ( delimiter , event ) * ;
/// ```
pub struct ParseTimetable {}

impl CompilingPass<&str, Vec<Event>, <Event as FromStr>::Err> for ParseTimetable {
    fn apply(s: &str) -> Result<Vec<Event>, <Event as FromStr>::Err> {
        s.split("---").map(Event::from_str).collect()
    }
}
