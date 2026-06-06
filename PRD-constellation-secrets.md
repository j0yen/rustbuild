# PRD: constellation-secrets — one root key in, every other secret bootstraps from it

Status: Draft v0.1
build_target: shell
Vision: visions/constellation.md
Depends: PRD-constellation-provision.md (the Ansible control plane this role plugs
  into) — provision must exist; this PRD is the secret layer it pulls from.

## TL;DR

Before a second machine can become a wintermute node it needs secrets it can't
carry in the repo: the NATS leaf/hub credentials the bus bridge connects with, the
Tailscale/Headscale pre-auth key enrollment uses, and `WM_ANTHROPIC_API_KEY` the
cloud brain serves on. `constellation-secrets` is the **bootstrap + service-secret
layer**: a single root key delivered out-of-band to a fresh host, and a
`sops`+`age` encrypted store (version-controlled as ciphertext) that every other
secret decrypts from at provision time. It is the prerequisite the rest of the
fleet silently assumes — appearance's chezmoi-`age` only decrypts *dotfile tokens*
and only *after* the host's age key already exists; nothing today puts that key
there, and nothing manages the *service* secrets that live outside dotfiles.

## Why this exists

This is the vision's own first open question, still unanswered:

> **Secrets bootstrapping** — every host needs one root secret (age key / SSH key /
> Vault password) delivered out-of-band before it can decrypt the rest. What's the
> delivery channel (USB, manual paste, the cloud node's tunnel)?
> — `visions/constellation.md`, Open questions

Evidence the gap is real and load-bearing, not hypothetical:

- **appearance handles only the leaf, not the root.** `PRD-constellation-appearance.md`
  AC6 + §Secrets: *"Any secret in the dotfiles is stored age-encrypted in the source
  … `chezmoi apply` decrypts with the host's age key."* It explicitly assumes the
  host's age key is already present and scopes itself to *dotfile* tokens (a
  status-bar weather key is its example). The **host key delivery** and any
  **non-dotfile / service** secret are out of its scope by construction.
- **mesh can't enroll without a secret it doesn't own.** `PRD-constellation-mesh.md`
  AC1 requires *"a pre-authorized auth key drawn from the encrypted store (no
  plaintext key in the repo)."* That store is this PRD. Mesh consumes it; mesh does
  not create it.
- **the bus is already built and already needs creds.** `~/wintermute/agorabus-nats-bridge`
  (`wm-busbridge` v0.1.1, live on disk) connects a NATS leaf at `127.0.0.1:4222`;
  a real multi-host leaf→hub link needs NATS user/nkey credentials that cannot sit
  in the repo. dispatch's `WM_NODES`/`WM_WORK` JetStream KV rides the same creds.
- **the brain secret is live and singular today.** Memory + journal: `WM_ANTHROPIC_API_KEY`
  is the one secret the cloud brain serves on (and on this laptop it has gone *empty*
  before — `docket wm-anthropic-key-empty`). A fleet needs it delivered to the cloud
  node reproducibly, not pasted by hand each rebuild.

So the secret surface is already three kinds (mesh auth key, NATS creds, API key)
across two-plus hosts, with **no managed store and no bootstrap path**. That is this
PRD.

## What this builds

A `constellation/secrets/` layer: an Ansible role + a thin `constellation-secrets`
helper (shell over `sops`/`age`), integrated so provision pulls secrets at apply
time. **Two halves, kept distinct:**

- **Bootstrap (the root key in).** A documented + scripted out-of-band delivery of
  exactly **one** root `age` identity per host: the recommended default is **manual
  paste over the existing SSH session / a USB stick for an air-gapped first boot**,
  with the cloud node's tunnel as the option once mesh is up. The script installs the
  key to `~/.config/sops/age/keys.txt` (0600), verifies it decrypts a canary, and
  **refuses to proceed if the canary fails** — so a mis-delivered key fails loud at
  bootstrap, never silently mid-provision.
- **Store (everything else out).** A `secrets/` tree of `sops`-encrypted YAML
  (committed as ciphertext; `.sops.yaml` creation-rule maps paths → recipient age
  keys per host role). Holds the **service** secrets: `nats.creds` (leaf + hub),
  `tailscale_authkey` / `headscale_preauth`, `WM_ANTHROPIC_API_KEY`, and a slot for
  future per-service creds. A `constellation-secrets get <name>` /
  `… render <template>` helper decrypts to a tmpfs path or env at provision time;
  **never** writes plaintext into the repo or a world-readable path.

UX / wiring:

- `constellation-secrets bootstrap` — install + verify the host root key (the
  out-of-band step), idempotent.
- `constellation-secrets get <name>` — decrypt one secret to stdout (for `EnvironmentFile`
  generation into a `0600` runtime path under `/run/user`); `--check` exits non-zero
  if a required secret is absent for this host's role.
- `constellation-secrets audit` — assert the repo contains **no plaintext secret**
  (grep-style, the same gate appearance uses) and that every secret declared for a
  role is decryptable by that role's key. Self-review / a later warrant can read it.
- An Ansible role `secrets` that runs `get`→`EnvironmentFile` for the host's role
  before the bus / mesh / brain units start (ordering: secrets-before-consumers).

Non-goals: the dotfile-token path (owned by appearance's chezmoi-`age`, unchanged);
choosing a mesh control plane (mesh/headscale); standing up NATS itself (bus). This
PRD delivers the **root-key bootstrap + the encrypted service-secret store + the
decrypt-at-provision wiring** only.

## Acceptance criteria

1. `constellation-secrets bootstrap` installs a single host root `age` identity to
   `~/.config/sops/age/keys.txt` (mode `0600`), verifies it decrypts a committed
   canary, and **exits non-zero without installing** if the canary fails to decrypt
   (mis-delivered key fails loud).
2. The bootstrap delivery channel is documented with a recommended default
   (manual-paste/USB) and a tunnel option, and the script is idempotent (re-running
   with the key already present is a no-op success).
3. A `sops`-encrypted store under `constellation/secrets/` holds at least
   `nats.creds`, a mesh enrollment key (`tailscale_authkey` or `headscale_preauth`),
   and `WM_ANTHROPIC_API_KEY`; the committed tree contains **no plaintext secret**
   (asserted by `constellation-secrets audit`, grep-style).
4. `.sops.yaml` creation-rules map each secret to the correct host-role recipient
   key(s); a secret encrypted for `role:cloud` is **not** decryptable with a
   `role:laptop`-only key (asserted with two distinct test keys / `FakeRecipient`).
5. `constellation-secrets get <name>` decrypts a named secret to stdout or a `0600`
   path under `/run/user/$UID`; plaintext is never written to the repo, `$HOME`
   dotfiles, or a world-readable path (asserted: the output path's mode + location
   are checked).
6. `constellation-secrets get --check <name>` exits non-zero when a secret required
   for the host's role is missing, so a half-provisioned node fails before its
   consumer (bus/mesh/brain) starts rather than after.
7. An Ansible `secrets` role renders the host-role secrets into an `EnvironmentFile`
   consumed by the bus/mesh/brain units, ordered **before** those units start
   (asserted against the unit/`After=` wiring or a documented role-order test).
8. `constellation-secrets audit` reports, for the host's role, every declared secret
   as decryptable-or-missing and the repo as plaintext-free, with a non-zero exit on
   any plaintext leak or undeclared-but-referenced secret — a single read self-review
   can gate on.
9. `sigpipe::reset()` (or the shell equivalent) is the first action so
   `constellation-secrets get … | head` never panics/`SIGPIPE`-aborts
   (`self_sigpipe_panic_toolkit`).
10. No secret material (root key, decrypted service secret, or canary plaintext) is
    ever committed; only ciphertext, the non-secret `.sops.yaml` policy, and the
    canary *ciphertext* are version-controlled.
