//! Latex frontends

use std::ops::Add;

use chrono::{Timelike, Local, Datelike, DateTime, Duration};

use crate::{
    passes::CompilingPass,
    event::Event
};

/// Frontend outputing events to a standalone LaTeX document containing a Tikz timetable
pub struct TikzFrontend {
}

/// TODO Do not assume everything is in the same month
/// Will find the bounding box (date, times) to generate a timetable
fn find_bounding_box(events: &Vec<Event>) -> (DateTime<Local>, DateTime<Local>) {
    assert!(!events.is_empty(), "{}", "Can't generate Tikz diagram from empty event list");
    let mut up_left = events.get(0).unwrap().start_date;
    let mut down_right = events.get(0).unwrap().start_date;
    for e in events {
        if e.start_date.time() < up_left.time() {
            up_left = up_left.with_hour(e.start_date.hour()).unwrap();
            up_left = up_left.with_minute(e.start_date.minute()).unwrap();
        }
        if e.start_date.day() < up_left.day() {
            up_left = up_left.with_day(e.start_date.day()).unwrap();
        }
        if (e.start_date + Duration::minutes(i64::from(e.duration))).time() > down_right.time() {
            down_right = down_right
                .with_hour(e.start_date.hour())
                .unwrap()
                .with_minute(e.start_date.minute())
                .map(|h| h.add(Duration::minutes(i64::from(e.duration))))
                .unwrap();
        }
        if e.start_date.day() > down_right.day() {
            down_right = down_right.with_day(e.start_date.day()).unwrap();
        }
    }

    (up_left, down_right)
}

impl CompilingPass<Vec<Event>, String, ()> for TikzFrontend {
    fn apply(events: Vec<Event>) -> Result<String, ()> {
    const POSTAMBLE: &str = r"
\end{tikzpicture}
\end{document}";
	
    let (up_left, down_right) = find_bounding_box(&events);

    let first_hour = up_left.hour();
    let last_hour = down_right.hour() + 1;

    let day_count = (down_right
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        -
        up_left
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()).num_days() + 1;

    let day_end = day_count + 1;
    let mut r: String = r"\documentclass{standalone}
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

    % First print a list of times.".to_owned();

    let foreach = r"
    \foreach \time   [evaluate=\time] in ".to_owned()
        + &format!("{{{first_hour},...,{last_hour}}}");

    r += &foreach;
    r += r"
            \node[anchor=east] at (1,\time) {\time:00};";
    
    r += r"
    % Draw some day dividers.
    \foreach \day   [evaluate=\day] in ";
    r += &format!("{{1,...,{day_end}}}");

    r +=r"
        \draw (\day,";
    r += &format!("{}", first_hour - 1);
    r += r") -- (\day,";
    r += &format!("{}", last_hour);
    r += ");";

    r += r"
	% Draw some hours dividers.";
    r += &foreach;
    r +=r"
        \draw (1,\time) -- (";
    r += &format!("{}", day_end);
    r += r", \time);";


    eprintln!("{}, {}", up_left, down_right);

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
        r += &format!("{}.{:.2}",
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
