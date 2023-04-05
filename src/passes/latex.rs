//! Latex frontends

use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use thiserror::Error;

use crate::{event::Event, passes::CompilingPass};

/// Backend outputing events to a standalone LaTeX document containing a Tikz timetable
pub struct TikzBackend {}

/// Bounding box of event.
///
/// This structure contains datetimes that allows to draw a box containing all the events from which
/// it was built in a calendar view.
pub struct BoundingBox {
    up_left: DateTime<Local>,
    down_right: DateTime<Local>,
}

/// The datetime is not valid.
#[derive(Debug, Error)]
#[error("the datetime created is invalid")]
pub struct InvalidDatetime;

impl BoundingBox {
    /// Get a datetime of the first day at 00:00 of the bounding box.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDatetime`] if it is not possible to build the datetime.
    pub fn first_day(&self) -> Result<DateTime<Local>, InvalidDatetime> {
        self.up_left
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .ok_or(InvalidDatetime)
    }

    /// Get a datetime of the last day at 00:00 of the bounding box
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidDatetime`] if it is not possible to build the datetime.
    pub fn last_day(&self) -> Result<DateTime<Local>, InvalidDatetime> {
        self.down_right
            .with_hour(0)
            .and_then(|dt| dt.with_minute(0))
            .ok_or(InvalidDatetime)
    }

    /// Create a datetime that represent the day of the earlier of both datetimes and at the time of
    /// day of the earlier of them.
    ///
    /// # Example
    ///
    /// If first is the November, 06 at 9:00 and the second is November, 08 at 8:00, the result is
    /// the November, 06 at 8:00.
    #[must_use]
    pub fn most_top_left(
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let mut res = *first;

        if second.time() < first.time() {
            res = res.with_hour(second.hour())?.with_minute(second.minute())?;
        }

        if second.date_naive() < first.date_naive() {
            res = res
                .with_day(second.day())?
                .with_month(second.month())?
                .with_year(second.year())?;
        }

        Some(res)
    }

    /// Create a datetime that represent the day of the later of both datetimes and at the time of
    /// day of the later of them.
    ///
    /// # Example
    ///
    /// If first is the November, 06 at 9:00 and the second is November, 08 at 8:00, the result is
    /// the November, 08 at 9:00.
    #[must_use]
    pub fn most_bottom_right(
        first: &DateTime<Local>,
        second: &DateTime<Local>,
    ) -> Option<DateTime<Local>> {
        let mut res = *first;

        if first.time() < second.time() {
            res = res.with_hour(second.hour())?.with_minute(second.minute())?;
        }

        if first.date_naive() < second.date_naive() {
            res = res
                .with_day(second.day())?
                .with_month(second.month())?
                .with_year(second.year())?;
        }

        Some(res)
    }
}

/// TODO Do not assume everything is in the same month
/// Will find the bounding box (date, times) to generate a timetable
fn find_bounding_box(events: &Vec<Event>) -> Option<BoundingBox> {
    let first = events.get(0)?;
    let mut up_left = first.start_date;
    let mut down_right = first.start_date;

    for e in events {
        let end_of_event = e.start_date + Duration::minutes(i64::from(e.duration));

        up_left = BoundingBox::most_top_left(&up_left, &e.start_date)?;
        down_right = BoundingBox::most_bottom_right(&down_right, &end_of_event)?;
    }

    Some(BoundingBox {
        up_left,
        down_right,
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
    #[error(transparent)]
    /// The datetime of either the first day or last day of the bounding box is not valid.
    InvalidDatetime(#[from] InvalidDatetime),
}

const LATEX_INTRO: &str = r"\documentclass{standalone}
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

    % First print a list of times.";

fn speaker_string(e: &Event) -> String {
    match e.speakers.len() {
        0 => {
            let (short, overflow) = e.title.split_at(std::cmp::min(25, e.title.len()));
            let rest = if overflow.is_empty() { "" } else { "..." };
            short.to_owned() + rest
        }
        1 => e.speakers[0].clone(),
        _ => format!("{} et~al.", e.speakers[0]),
    }
}

impl CompilingPass<Vec<Event>> for TikzBackend {
    type Residual = String;
    type Error = TikzBackendCompilationError;

    // TODO: Split this huge function into smaller ones.
    fn apply(events: Vec<Event>) -> Result<Self::Residual, Self::Error> {
        const POSTAMBLE: &str = r"
\end{tikzpicture}
\end{document}";

        let bb = find_bounding_box(&events).ok_or(TikzBackendCompilationError::NoEventProvided)?;

        let first_hour = bb.up_left.hour();
        let last_hour = bb.down_right.hour() + 1;

        let day_count = (bb.last_day()? - bb.first_day()?).num_days() + 1;
        let day_end = day_count + 1;

        let mut r: String = LATEX_INTRO.to_owned();

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
        r += &format!("{last_hour}");
        r += ");";

        r += r"
	% Draw some hours dividers.";
        r += &foreach;
        r += r"
        \draw (1,\time) -- (";
        r += &format!("{day_end}");
        r += r", \time);";

        for i in 0..day_count {
            let col = i + 1;
            r += r"
        \node[anchor=south] at (";
            r += &format!("{col}");
            r += r".5, ";
            r += &format!("{}", first_hour - 1);
            r += ".5) {";
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
            r += &speaker_string(&e);
            r += "};";
        }

        r += POSTAMBLE;
        Ok(r)
    }
}
