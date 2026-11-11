//! Says when the directory you just walked into has a `.env` in it.
//!
//! The thing `direnv` is for, minus the part that runs it: a project
//! with an `.env` beside it is a project you probably meant to load
//! before running anything, and the moment you find out is usually the
//! moment something fails for want of a variable.
//!
//! # What it does not do
//!
//! It does not load it. A plugin cannot: there is no way to put text
//! into your shell, and that is on purpose — a plugin that could type
//! into a pane could run anything, which would make the whole
//! capability model decorative. So this says the file is there and what
//! to type, and you type it.
//!
//! # Why `stat` and not a filesystem
//!
//! A plugin has no filesystem at all; the sandbox is a WASI context
//! with nothing in it. Asking whether a file exists therefore means
//! asking a program, and `stat` is the narrowest one that answers the
//! question — its exit status alone is the answer, so nothing is parsed.
//!
//! `host::run` has no working directory of its own, so the path is
//! built from the pane's `cwd` and passed absolute. A relative path
//! would be resolved against the daemon's directory, which is nowhere
//! near the pane and would answer about a different `.env` entirely.
//!
//! # Why the work is not in `render`
//!
//! `render` is called whenever Haven draws. Spawning `stat` per frame
//! would be unforgivable, so it runs when the directory changes and
//! `render` formats what is already known.

use std::cell::RefCell;

use haven_plugin_api::{dim, focused, host, Command, Event, Plugin, Run};

haven_plugin_api::plugin!(EnvFile);

struct EnvFile;

/// What the last look found, so `render` can answer without asking.
#[derive(Default)]
struct State {
    /// The directory the answer is about, so moving back and forth does
    /// not re-run `stat` for somewhere already looked at.
    looked_at: String,
    /// Whether that directory has an `.env` in it.
    found: bool,
    /// The last directory announced, so walking out and back in does not
    /// say the same thing again. A notice you have already read is
    /// noise, and a plugin that produces noise gets uninstalled.
    announced: String,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// The file this looks for. `.envrc` is `direnv`'s and means something
/// executable; `.env` is the inert list of `KEY=value` that everything
/// from docker-compose to a dozen frameworks reads, and is the one you
/// can safely tell somebody to source.
const NAME: &str = ".env";

impl Plugin for EnvFile {
    fn on_event(event: Event) -> Vec<Command> {
        // Every event here means "where you are may have changed".
        if !matches!(
            event,
            Event::Started | Event::PaneFocus(_) | Event::CwdChanged(_)
        ) {
            return Vec::new();
        }
        let Some(cwd) = focused().map(|pane| pane.cwd).filter(|cwd| !cwd.is_empty()) else {
            // No pane, or a shell that never reported where it is.
            // Nothing to say, and nothing to blame it for.
            STATE.with(|s| *s.borrow_mut() = State::default());
            return Vec::new();
        };

        let fresh = STATE.with(|s| s.borrow().looked_at != cwd);
        if fresh {
            let found = exists(&path_in(&cwd));
            STATE.with(|s| {
                let mut s = s.borrow_mut();
                s.looked_at = cwd.clone();
                s.found = found;
            });
        }

        let (found, said) = STATE.with(|s| {
            let s = s.borrow();
            (s.found, s.announced == cwd)
        });
        if !found || said {
            return Vec::new();
        }
        STATE.with(|s| s.borrow_mut().announced = cwd.clone());
        vec![Command::Notice(format!(
            "{NAME} here — load it with: set -a; . ./{NAME}; set +a"
        ))]
    }

    fn render() -> Vec<Run> {
        STATE.with(|s| {
            let s = s.borrow();
            if !s.found {
                return Vec::new();
            }
            // Dim, and three characters. It is standing information
            // about where you are, not an answer to something that just
            // happened — the notice already did that.
            vec![dim(NAME)]
        })
    }
}

/// `<cwd>/.env`, with exactly one slash however the directory ends.
fn path_in(cwd: &str) -> String {
    format!("{}/{NAME}", cwd.trim_end_matches('/'))
}

/// Whether a path exists, by its exit status alone.
///
/// Nothing is read from `stat`'s output: the formats differ between GNU
/// and BSD and none of them is needed. A refusal — no `exec` grant —
/// is treated as "do not know", which draws nothing rather than
/// claiming there is no `.env` when nobody looked.
fn exists(path: &str) -> bool {
    match host::run(&["stat".to_string(), path.to_string()]) {
        Ok(out) => out.status == 0,
        Err(why) => {
            // Said once where somebody can find it, rather than on the
            // status bar where it would sit forever.
            host::log(&format!("cannot look for {NAME}: {why}"));
            false
        }
    }
}
