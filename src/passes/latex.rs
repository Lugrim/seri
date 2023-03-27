//! Latex frontends

use std::{ops::Add, str::FromStr};

use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use thiserror::Error;

use crate::{event::Event, passes::CompilingPass};

/// Frontend outputing events to a standalone LaTeX document containing a Tikz timetable
pub struct TikzFrontend {}

/// Bounding box of event.
///
/// This structure contains datetimes that allows to draw a box containing all the events from which
/// it was built in a calendar view.
pub struct BoundingBox {
    up_left: DateTime<Local>,
    down_right: DateTime<Local>,
}

impl BoundingBox {
    /// Get a datetime of the first day at 00:00 of the bounding box.
    #[must_use]
    pub fn first_day(&self) -> Option<DateTime<Local>> {
        self.up_left.with_hour(0)?.with_minute(0)
    }

    /// Get a datetime of the last day at 00:00 of the bounding box.
    #[must_use]
    pub fn last_day(&self) -> Option<DateTime<Local>> {
        self.down_right.with_hour(0)?.with_minute(0)
    }
}

/// TODO Do not assume everything is in the same month
/// Will find the bounding box (date, times) to generate a timetable
fn find_bounding_box(events: &Vec<Event>) -> Option<BoundingBox> {
    events.get(0).and_then(|first| {
        let mut up_left = first.start_date;
        let mut down_right = first.start_date;
        for e in events {
            if e.start_date.time() < up_left.time() {
                up_left = up_left.with_hour(e.start_date.hour())?;
                up_left = up_left.with_minute(e.start_date.minute())?;
            }
            if e.start_date.date_naive() < up_left.date_naive() {
                up_left = up_left.with_day(e.start_date.day())?;
            }
            if (e.start_date + Duration::minutes(i64::from(e.duration))).time() > down_right.time()
            {
                down_right = down_right
                    .with_hour(e.start_date.hour())?
                    .with_minute(e.start_date.minute())
                    .map(|h| h.add(Duration::minutes(i64::from(e.duration))))?;
            }
            if e.start_date.date_naive() > down_right.date_naive() {
                down_right = down_right
                    .with_day(e.start_date.day())?
                    .with_month(e.start_date.month())?
                    .with_year(e.start_date.year())?;
            }
        }

        Some(BoundingBox {
            up_left,
            down_right,
        })
    })
}

/// Error occuring when compiling an event list to tikz.
#[derive(Debug, Error)]
pub enum TikzBackendCompilationError {
    /// The event could not be parsed.
    #[error(transparent)]
    CouldNotParseEvent(#[from] <Event as FromStr>::Err),
    #[error("no event was provided")]
    /// The list of events was empty.
    NoEventProvided,
}

impl CompilingPass<Vec<Event>, String, TikzBackendCompilationError> for TikzFrontend {
    fn apply(events: Vec<Event>) -> Result<String, TikzBackendCompilationError> {
        const POSTAMBLE: &str = r"
\end{tikzpicture}
\end{document}";

        let bb = find_bounding_box(&events).ok_or(TikzBackendCompilationError::NoEventProvided)?;

        let first_hour = bb.up_left.hour();
        let last_hour = bb.down_right.hour() + 1;

        let day_count = (bb.last_day().unwrap() - bb.first_day().unwrap()).num_days() + 1;
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

    % First print a list of times."
            .to_owned();

        let foreach = r"
    \foreach \time   [evaluate=\time] in "
            .to_owned()
            + &format!("{{{first_hour},...,{last_hour}}}");

        r += &foreach;
        r += r"
            \node[anchor=east] at (1,\time) {\time:00};";

        r += r"
    % Draw some day dividers.
    \foreach \day   [evaluate=\day] in ";
        r += &format!("{{1,...,{day_end}}}");

        r += r"
        \draw (\day,";
        r += &format!("{}", first_hour - 1);
        r += r") -- (\day,";
        r += &format!("{}", last_hour);
        r += ");";

        r += r"
	% Draw some hours dividers.";
        r += &foreach;
        r += r"
        \draw (1,\time) -- (";
        r += &format!("{}", day_end);
        r += r", \time);";

        for i in 0..day_count {
            let col = i + 1;
            r += r"
        \node[anchor=south] at (";
            r += &format!("{col}");
            r += r".5, 8.5) {";
            r += &format!("{}", (bb.up_left + Duration::days(i)).format("%A, %B %e"));
            r += r"};";
        }

        for e in events {
            r += r"
    \node[";
            r += &format!("{}", e.event_type);
            r += "={";
            r += &format!("{:.2}", f64::from(e.duration) / 60.);
            r += "}{";
            r += "1"; // TODO Compute simultaneous events
            r += "}] at (";
            r += &format!("{}", e.start_date.day() - bb.up_left.day() + 1);
            r += ",";
            r += &format!(
                "{}.{:.2}",
                e.start_date.format("%H"),
                e.start_date.minute() * 5 / 3
            );
            r += ") {";
            let (short_title, title_overflow) = e.title.split_at(std::cmp::min(25, e.title.len()));
            r += short_title;
            if !title_overflow.is_empty() {
                r += "...";
            }
            r += "};";
        }

        r += POSTAMBLE;
        Ok(r)
    }
}
