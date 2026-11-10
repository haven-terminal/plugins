# Haven plugins

The plugin repository for [Haven](https://github.com/haven-terminal/Haven).

```console
$ haven plugin repo add https://github.com/haven-terminal/plugins.git \
      --key '2F10 059C F663 6B15 0B95 8486 7D04 983E 5D12 A3E4'
$ haven plugin repo update
$ haven plugin search
```

That fingerprint is the thing worth checking, and checking it against
*this page* is weaker than checking it against somewhere else — it is
also on the [pacman repository
instructions](https://github.com/haven-terminal/Haven/blob/master/packaging/README.md)
and in `haven-repo.pub` on every Haven release, because a fingerprint
that only ever appears beside the thing it signs is a fingerprint nobody
has verified.

Leave `--key` out and Haven imports the key from here and shows it to
you to accept. That pins it from then on, so only the first contact was
ever exposed — but nothing checked it was us, and `haven plugin repo
list` will keep saying `(never checked)`.

## What is in here

| File | What it is |
|---|---|
| `plugins.yaml` | the index: every plugin, with a SHA-256 for its artifact |
| `plugins.yaml.sig` | a detached OpenPGP signature over that index |
| `key.asc` | the public half of the key that signs it |
| `pool/` | the artifacts the index names |

One signature over one small file, and that file carries a hash for
every download — so checking the signature is checking everything.
`gpg --verify plugins.yaml.sig plugins.yaml` works with no Haven
installed.

The plugin **sources** are not here. They live in
[`plugins/`](https://github.com/haven-terminal/Haven/tree/master/plugins)
in the main repository, where they are also what the plugin runtime is
tested against — a plugin that does not load is a failing test there
rather than a bad artifact here. This repository is the published half.

## How it is published

By [`.github/workflows/publish.yml`](.github/workflows/publish.yml),
which checks out the main repository, builds every plugin for
`wasm32-wasip2`, generates the index from each plugin's own manifest,
signs it, and commits the result here. Nothing is built on a
contributor's machine and no private key leaves the secret store.

It needs `GPG_PRIVATE_KEY` and `GPG_PASSPHRASE` visible to this
repository — set them on the organisation, or on this repository, with
the same key the Haven releases are signed with. Then run the workflow.

Until it has run once, this repository has a key and no index, and
`haven plugin repo add` will tell you so rather than pretending.
