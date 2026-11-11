//! `env-file`, against the real runtime.
//!
//! It needs `stat`, so the granted and refused cases are both here, and
//! the directories are real ones made by the test — the plugin is
//! genuinely asking the filesystem through a program, which is the only
//! way it can.

use haven_plugin_host::testing::{plain_load, with_exec, Session};
use haven_plugin_host::{text_of, Command, Event};

/// A directory, with or without a `.env` in it, cleaned up after.
struct Dir(std::path::PathBuf);

impl Dir {
    fn new(tag: &str, with_env: bool) -> Self {
        let path = std::env::temp_dir().join(format!(
            "haven-envfile-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("dir");
        if with_env {
            std::fs::write(path.join(".env"), "TOKEN=hunter2\n").expect("write");
        }
        Self(path)
    }

    fn as_str(&self) -> String {
        self.0.to_string_lossy().to_string()
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_directory_with_an_env_file_is_announced_once() {
    let dir = Dir::new("has", true);
    let session = Session::new().pane(1, "sh", &dir.as_str());
    let Some(mut p) = with_exec("env-file", &["stat"], session) else {
        return;
    };

    let said = p.on_event(&Event::Started).expect("started");
    let notices: Vec<&str> = said
        .iter()
        .filter_map(|c| match c {
            Command::Notice(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(notices.len(), 1, "{said:?}");
    assert!(notices[0].contains(".env here"), "{notices:?}");
    // The command it gives has to be one that actually loads a file.
    assert!(notices[0].contains("set -a"), "{notices:?}");

    assert_eq!(text_of(&p.render().expect("render")), ".env");

    // Arriving again at the same directory says nothing: a notice you
    // have already read is noise.
    let again = p.on_event(&Event::PaneFocus(1)).expect("focus");
    assert!(
        again.is_empty(),
        "it announced the same directory twice: {again:?}"
    );
    assert_eq!(
        text_of(&p.render().expect("render")),
        ".env",
        "but the segment stays"
    );
}

#[test]
fn a_directory_without_one_says_nothing_at_all() {
    let dir = Dir::new("hasnt", false);
    let session = Session::new().pane(1, "sh", &dir.as_str());
    let Some(mut p) = with_exec("env-file", &["stat"], session) else {
        return;
    };
    assert!(p.on_event(&Event::Started).expect("started").is_empty());
    assert_eq!(text_of(&p.render().expect("render")), "");
}

#[test]
fn walking_into_one_announces_it_and_walking_out_clears_it() {
    let empty = Dir::new("out", false);
    let project = Dir::new("in", true);

    // Start somewhere with nothing.
    let Some(mut p) = with_exec(
        "env-file",
        &["stat"],
        Session::new().pane(1, "sh", &empty.as_str()),
    ) else {
        return;
    };
    assert!(p.on_event(&Event::Started).expect("started").is_empty());
    assert_eq!(text_of(&p.render().expect("render")), "");

    // Walk into the project. The snapshot is what moves the pane.
    p.refresh(Session::new().pane(1, "sh", &project.as_str()).into());
    let said = p
        .on_event(&Event::CwdChanged(haven_plugin_host::CwdChange {
            pane: 1,
            cwd: project.as_str(),
        }))
        .expect("cwd");
    assert_eq!(said.len(), 1, "{said:?}");
    assert_eq!(text_of(&p.render().expect("render")), ".env");

    // And back out: the segment goes with it.
    p.refresh(Session::new().pane(1, "sh", &empty.as_str()).into());
    p.on_event(&Event::CwdChanged(haven_plugin_host::CwdChange {
        pane: 1,
        cwd: empty.as_str(),
    }))
    .expect("cwd");
    assert_eq!(
        text_of(&p.render().expect("render")),
        "",
        "it kept announcing an old directory"
    );
}

#[test]
fn without_the_grant_it_draws_nothing_rather_than_claiming_there_is_none() {
    // Refused is not the same as "there is no .env", and the difference
    // matters: this directory has one.
    let dir = Dir::new("refused", true);
    let Some(mut p) = plain_load("env-file", Session::new().pane(1, "sh", &dir.as_str())) else {
        return;
    };
    assert!(p.on_event(&Event::Started).expect("started").is_empty());
    assert_eq!(text_of(&p.render().expect("render")), "");
    // And it says why where somebody can find it.
    assert!(
        p.log().iter().any(|line| line.contains("stat")),
        "nothing logged about the refusal: {:?}",
        p.log()
    );
}

#[test]
fn a_pane_that_does_not_know_where_it_is_is_not_guessed_at() {
    // No `OSC 7`, so no directory. Guessing would mean answering about
    // the daemon's directory, which is nowhere near the pane.
    let Some(mut p) = with_exec("env-file", &["stat"], Session::new().pane(1, "sh", "")) else {
        return;
    };
    assert!(p.on_event(&Event::Started).expect("started").is_empty());
    assert_eq!(text_of(&p.render().expect("render")), "");
}
