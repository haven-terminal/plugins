# Haven plugins

The plugins for [Haven](https://github.com/haven-terminal/Haven), and the
signed repository they are published as.

```console
$ haven plugin repo add https://github.com/haven-terminal/plugins.git \
      --key '2F10 059C F663 6B15 0B95 8486 7D04 983E 5D12 A3E4'
$ haven plugin repo update
$ haven plugin search
```

That fingerprint is the thing worth checking, and checking it against
*this page* is weaker than checking it anywhere else — it is also in
`haven-repo.pub` on every Haven release and in the [pacman repository
instructions](https://github.com/haven-terminal/Haven/blob/master/packaging/README.md),
because a fingerprint that only ever appears beside the thing it signs is
one nobody has verified.

Leave `--key` out and Haven imports the key from here and shows it to you
to accept. It is pinned from then on, so only the first contact was ever
exposed — but nothing checked it was us, and `haven plugin repo list`
will keep saying `(never checked)`.

## What is here

| Plugin | Shows | Needs |
|---|---|---|
| `clock` | `17:20` | nothing |
| `cwd-crumbs` | `~/P/H/c/haven-client` | nothing |
| `python-venv` | `venv:haven` | nothing |
| `ssh-marker` | `⇥ box`, when the pane is elsewhere | nothing |
| `command-timer` | `cargo build took 2m14s`, past ten seconds | nothing |
| `fail-watch` | `mke2fs: not found` | nothing |
| `pomodoro` | `work 23:41` | nothing |
| `gitline` | `master* ↑5` | `git` |
| `git-wip` | `+736-46` | `git` |
| `git-stash` | `⚑2` | `git` |
| `load-average` | `7.22`, bold past the core count | `uptime`, `nproc` |
| `disk-free` | `205G 54%` | `df` |
| `docker-status` | `docker 7` | `docker` |
| `url-hints` | `3 links` | your screen |
| `build-errors` | `12 errors, 3 warnings` | your screen |
| `secret-guard` | warns when a credential was just printed | your screen |

Each directory under `useful/` holds the plugin, its `MANIFEST.yaml` —
the index entry it wants — and a README saying what it does and what it
cannot. The last three ask to **read what your panes are showing**, which
is the capability Haven names first in every prompt; each of their
READMEs says in plain words what it reads and why it cannot work without
it.

Installed, several of them look like this, between the tabs and the
clock:

```
 haven-1   1:1   17:20 /P/HavenTerminal +795-55 master* ↑5   1 panes · 00:10
```

## Building

```console
$ cargo build --release --target wasm32-wasip2
```

A plugin is a WebAssembly component, and since Rust 1.82 that target
emits one directly — no `cargo-component`, no `wasm-tools`. The
components land in `target/wasm32-wasip2/release/` at 50–90 KB each.

The SDK and the WIT world come from the main repository as a git
dependency, because the runtime that has to agree with them lives there.

## Testing

```console
$ cd checks && HAVEN_PLUGINS=.. cargo test
```

72 tests. Each one loads the real component into the real runtime with
the real capability checks and shows it a made-up session — there is no
mock, because a mock of a sandbox proves nothing. `$HAVEN_PLUGINS` points
the runtime's test helper at this workspace.

The negative tests are the ones that matter: a fast command notices
nothing, `grep` exiting 1 is not a failure, a clean build counts nothing,
a screen full of `password=` and a git object id produces no warning, and
`secret-guard`'s notice is asserted not to contain the token — not the
token, not a 16-character slice of it, not its prefix.

## Publishing

[`publish.sh`](publish.sh) turns this directory into a repository Haven
will accept: reproducible tarballs, a SHA-256 per artifact, an index
generated from each plugin's own manifest, and a detached signature if
you hand it a fingerprint. It never goes looking for a key — one that
quietly picks a key is one that signs with the wrong one.

```console
$ ./publish.sh /tmp/out                     # unsigned; tells you what is left
$ ./publish.sh /tmp/out YOUR_FINGERPRINT    # signed
```

In practice [`.github/workflows/publish.yml`](.github/workflows/publish.yml)
does it: it builds, signs with the key in the secret store, **verifies the
result the way a user would** — `repo add` and `search` against a
throwaway configuration, failing if the index lists nothing — and only
then commits `plugins.yaml`, `plugins.yaml.sig`, `key.asc` and `pool/`
here.

It needs `GPG_PRIVATE_KEY` and `GPG_PASSPHRASE` visible to this
repository. Until it has run, this repository has plugins and no signed
index, and `haven plugin repo add` will say so rather than pretend.

## Writing one

See [writing and publishing
them](https://github.com/haven-terminal/Haven/blob/master/site/src/writing-plugins.md).
A plugin is a `cdylib` implementing one trait, with three entry points:
`init`, `on_event` returning commands the host applies, and `render`
returning styled text for the status bar. It gets no filesystem, no
network and no environment, and only the permissions its manifest asked
for and the user granted.
