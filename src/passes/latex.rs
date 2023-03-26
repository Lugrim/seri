//! Latex frontends

use chrono::{Timelike, Local, Datelike, DateTime};

use crate::{
    passes::CompilingPass,
    event::Event
};

/// Frontend outputing events to a standalone LaTeX document containing a Tikz timetable
pub struct TikzFrontend {
}

/// TODO Use duration too
/// TODO Do not assume everything is in the same month
/// Will find the bounding box (date, times) to generate a timetable
fn find_bounding_box(events: &Vec<Event>) -> (DateTime<Local>, DateTime<Local>) {
    assert!(!events.is_empty(), "{}", "Can't generate Tikz diagram from empty event list");
    let mut up_left = events.get(0).unwrap().start_date;
    let mut down_right = events.get(0).unwrap().start_date;
    for e in events {
        if e.start_date.hour() < up_left.hour() {
            up_left = up_left.with_hour(e.start_date.hour()).unwrap();
        }
        if e.start_date.day() < up_left.day() {
            up_left = up_left.with_day(e.start_date.day()).unwrap();
        }
        if e.start_date.hour() + e.duration /60 > down_right.hour() {
            down_right = down_right.with_hour(e.start_date.hour()).unwrap();
        }
        if e.start_date.day() > down_right.day() {
            down_right = down_right.with_day(e.start_date.day()).unwrap();
        }
    }

    (up_left, down_right)
}

impl CompilingPass<Vec<Event>, String, ()> for TikzFrontend {
    fn apply(events: Vec<Event>) -> Result<String, ()> {
    const PREAMBLE: &str = r"
\documentclass{standalone}
\usepackage{tikz}

\begin{document}

% These set the width of a day and the height of an hour.
\newcommand*\daywidth{6cm}
\newcommand*\hourheight{4em}

% The entry style will have two options:
% * the first option sets how many hours the entry will be (i.e. its height);
% * the second option sets how many overlapping entries there are (thus
%   determining the width).
\tikzset{entry/.style 2 args={
    xshift=(0.5334em+0.8pt)/2,
    draw,
    line width=0.8pt,
    font=\sffamily,
    rectangle,
    rounded corners,
    fill=blue!20,
    anchor=north west,
    inner sep=0.3333em,
    text width={\daywidth/#2-1.2em-1.6pt},
    minimum height=#1*\hourheight,
    align=center
}}

\tikzset{talk/.style 2 args={
		entry={#1}{#2},
		fill=red!40
	}
}

\tikzset{meal/.style 2 args={
		entry={#1}{#2},
		fill=green!40
	}
}

\tikzset{fun/.style 2 args={
		entry={#1}{#2},
		fill=blue!40
	}
}

\tikzset{transport/.style 2 args={
		entry={#1}{#2},
		fill=gray!20
	}
}

% Start the picture and set the x coordinate to correspond to days and the y
% coordinate to correspond to hours (y should point downwards).
\begin{tikzpicture}[y=-\hourheight,x=\daywidth]

    % First print a list of times.
	\foreach \time   [evaluate=\time] in {8,...,22}
        \node[anchor=north east] at (1,\time) {\time:00};

    % Draw some day dividers.
    \draw (1,6.5) -- (1,23);
    \draw (2,6.5) -- (2,23);
    \draw (3,6.5) -- (3,23);

	% Draw some hours dividers.
	\foreach \time   [evaluate=\time] in {8,...,22}
        \draw (1,\time) -- (3,\time);

";

    const POSTAMBLE: &str = r"
\end{tikzpicture}
\end{document}";

    let mut r: String = PREAMBLE.to_owned();

    let (up_left, down_right) = find_bounding_box(&events);

    for e in events {
        r += r"\node[";
        r += &format!("{}", e.event_type);
        r += "={";
        r += &format!("{:.2}",f64::from(e.duration) / 60.);
        r += "}{";
        r += "1"; // TODO Compute simultaneous events
        r += "}] at (";
        r += &format!("{}",
                      e.start_date.day() - up_left.day() + 1);
        r += ",";
        r += &format!("{}.{}",
                      e.start_date.format("%H"),
                      e.start_date.minute() * 5 / 3);
        r += ") {";
        let (short_title, title_overflow) = e.title.split_at(std::cmp::min(25, e.title.len()));
        r += short_title;
        if !title_overflow.is_empty() {
            r += "...";
        }
        r += "};\n";
    }

    r += POSTAMBLE;
    Ok(r)
    }
}
