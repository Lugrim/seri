//! Latex frontends
use crate::{
    passes::CompilingPass,
    event::Event
};

/// Frontend outputing events to a standalone LaTeX document containing a Tikz timetable
pub struct TikzFrontend {
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

    let mut i = 0; // TODO Temp
    for e in events {
        i += 1;
        r += r"\node[";
        r += &format!("{}", e.event_type);
        r += "={";
        r += "0.5"; // TODO Duration
        r += "}{";
        r += "1"; // TODO Compute simultaneous events
        r += "}] at (";
        r += "1"; // TODO Compute day
        r += ",";
        r += & (9 + i).to_string(); // TODO Compute time
        r += ") {";
        let (short_title, title_overflow) = e.title.split_at(std::cmp::min(50, e.title.len()));
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
