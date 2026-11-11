# env-file

Says when the directory you have just walked into has a `.env` in it.

```
 haven  1:shell        17:20 ~/P/myapp .env        1 panes · 00:12
```

and, the first time you arrive there:

```
env-file: .env here — load it with: set -a; . ./.env; set +a
```

## What it needs

`exec: [stat]`, and nothing else.

A plugin has no filesystem — the sandbox is a WASI context with nothing
in it — so asking whether a file exists means asking a program. `stat`
is the narrowest one that answers: its exit status *is* the answer, so
nothing is parsed and the differences between GNU and BSD `stat` never
come up.

Be aware of what granting it means. An `exec` allowlist names `argv[0]`
and nothing more, so this plugin could in principle `stat` any path on
the machine, not only the one it is looking for. That is a small thing
to be able to learn — whether a path exists — but it is not nothing, and
it is the honest limit of the capability as it stands.

## What it does not do

**It does not load the file.** A plugin cannot, and that is deliberate:
there is no way for one to put text into your shell. A plugin that could
type into a pane could run anything, which would make every other
capability in Haven decorative — the `exec` allowlist would be a
formality if any plugin could simply type `curl … | sh` at your prompt.

So auto-activation is not a thing this can grow into without Haven
growing a new, separately granted way to send text to a shell, with its
own prompt and its own entry in the manifest. Until then the honest
shape is: it tells you the file is there, and you decide.

## What it looks for

`.env` only. Not `.envrc`, which is `direnv`'s and is a *script* — the
advice "source this" is safe for an inert list of `KEY=value` and is not
safe for an arbitrary program somebody put in a directory you happened
to `cd` into.

## Limits

- It knows where a pane is from `OSC 7`, which needs shell integration
  loaded. Without it the pane reports no directory and this draws
  nothing — rather than guessing, and guessing wrong.
- It says the file is there, never whether it is already loaded. A
  plugin cannot see your shell's environment, so "already active" is not
  a question it can answer.
- One notice per directory, so walking out and back in is quiet. A
  notice you have already read is noise.
