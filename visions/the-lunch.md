# Vision: the-lunch — the daily convening ritual

> Sub-vision of [roundtable](roundtable.md). The Round Table met for lunch
> every day at the Algonquin, 1919–1929 — a *standing* engagement no one had
> to be invited to. `the-lunch` is wintermute's version of that standing
> engagement: the layer that pulls the day's solo creative work onto one table,
> seats the right voices, sets an agenda, keeps the minutes, and wires the
> cadence so the gathering happens without being asked.

## TL;DR

wintermute's creative wing is a pile of **write-only soloists**. `day-haiku`
writes a haiku into its `summary/content.json` and stops. `conversations-zine`
extracts moments to a file. `letters-we-never-sent` drafts a monthly letter by
design no one reads. `self-portrait` watches `CLAUDE_SELF` diffs in silence.
`ambient` emits telemetry cues into an empty room. Nothing ever *convenes*
them — there is no table, no noon, no record of a gathering.

`the-lunch` builds the convening foundation: a structured **table** object that
gathers the day's artifacts, a **seating** decision that picks which personas
attend based on what's actually on the table, a **menu** (agenda) that orders
and time-boxes the discussion, **minutes** that keep the durable transcript,
and the **standing invitation** — a systemd-user timer at noon plus a hook —
so the lunch recurs on its own. The critique ensemble (`vicious-circle`) and
the wit games (`bon-mot`, `thanatopsis`) all *run inside* the table this
vision sets.

## End-state

At 12:00 local a `the-lunch.timer` fires (after the Round Table's real
cadence). `the-lunch convene` walks the creative repos, pulls today's
artifacts onto a `Table` — a single typed, serializable object: the haiku, any
zine excerpt, the current unsent-letter draft, the latest self-portrait diff,
the day's ambient cues — each as a `Dish` with a kind, source, content, and
provenance. `the-lunch seat` reads the table and selects attending personas
(a letter draws the realist Ferber; a haiku draws Parker; a self-portrait diff
draws Woollcott's enthusiasm). `the-lunch menu` sets the agenda: artifact
order, what gets discussed, a per-dish time-box, and a total table budget.
Downstream, `vicious-circle` runs its critique round *against this menu*;
`the-lunch minutes` records who said what into a durable, append-only
transcript (the gathering's record). The table, seating, menu, and minutes are
the substrate every other roundtable sub-vision reads and writes.

The end-state: a creative day on this laptop has a *place it goes* — a table
that convenes at noon, seats the right voices, and leaves a written record that
the lunch happened.

## Decision: one table per day (with an on-demand override)

The umbrella open question — *one table per day, or convene on-demand when N
new artifacts accumulate?* — is decided here: **one table per day is the
default.** The Round Table's defining property was its *cadence*, not its
volume; a once-daily noon gathering is what makes it a ritual and keeps the
record (and any `--lavish` API cost downstream) bounded and legible. The table
is keyed by local date (`table-YYYY-MM-DD.json`); a second `convene` on the
same day is idempotent — it refreshes dishes in place, it does not open a
second table.

On-demand convening is supported but **opt-in**, not the trigger: `the-lunch
convene --now` opens (or refreshes) today's table immediately regardless of the
timer, for when the user wants to gather mid-day. The "N new artifacts" model
is explicitly rejected as the default — it would make the ritual bursty and
the minutes non-comparable day to day.

## Components (one per PRD)

- **the-lunch-convene** (`rust-lib` + thin CLI) — the foundation type. Defines
  the `Table` and `Dish` model and the artifact adapters that pull the day's
  creative outputs onto the table. Idempotent, date-keyed. Everything else
  depends on this crate.
- **the-lunch-seating** (`rust-cli`) — given a table, select attending
  personas via declarative rules keyed on dish kinds. Emits a `Seating` the
  menu and the critique ensemble consume.
- **the-lunch-menu** (`rust-cli`) — set the agenda: order the dishes, mark what
  is discussed, assign per-dish and total time-boxes. Emits a `Menu`.
- **the-lunch-minutes** (`rust-cli`) — append-only transcript of the gathering:
  who (persona) said what, against which dish, in menu order. The durable
  record `vicious-circle` writes its verdicts into.
- **the-lunch-standing-invitation** (`mixed` — shell + config) — the cadence:
  a `the-lunch.timer`/`.service` at noon and a SessionStart-style hook, the
  "standing invitation" that makes the lunch happen unprompted. Extends the
  proven `daily-receipt.timer` / `claude-dream.timer` pattern.

## Order

```
the-lunch-convene  (Table + Dish foundation type)
      │
      ├──> the-lunch-seating  (who attends)
      │           │
      │           └──> the-lunch-menu  (agenda over the seating)
      │                       │
      └────────────────────────┴──> the-lunch-minutes  (record)
                                          │
                                  the-lunch-standing-invitation
                                  (timer + hook wires it all to noon)
```

- `the-lunch-convene` ships first; it is the type the others import.
- `seating` and `menu` layer on top (menu reads seating).
- `minutes` records over a convened+seated+menu'd table.
- `standing-invitation` ships last — it wires the whole chain to the noon timer
  and the hook; nothing to wire until the chain exists.

## Why this is honest, not fan-fiction

- Real write-only soloists it convenes:
  `~/wintermute/day-haiku/` (haiku → `summary/content.json`),
  `~/wintermute/conversations-zine/`, `~/wintermute/letters-we-never-sent/`,
  `~/wintermute/self-portrait/`, `~/wintermute/ambient/`.
- Real cadence precedent it extends: `~/wintermute/daily-receipt/` plus the
  installed `~/.config/systemd/user/daily-receipt.timer`
  (`OnCalendar=*-*-* 21:30:00`) and `claude-dream.timer` — the exact
  systemd-user timer pattern `standing-invitation` reuses for noon.

## Open questions

- Does the table persist as a single JSON file per day under
  `$XDG_STATE_HOME/the-lunch/`, or one dir per day holding table + minutes?
  Lean: one dir per day (`.../the-lunch/2026-06-08/{table,seating,menu,minutes}.json`)
  so the gathering's whole record co-locates. Decide in `the-lunch-convene`.
- When an artifact source repo is absent or has no output today, is its dish
  *omitted* or recorded as an explicit *empty seat*? Lean: explicit empty seat,
  so the minutes show what was missing. Decide in `the-lunch-convene`.
- Should `standing-invitation` also publish a notification (peon-ping) when the
  table convenes, or stay silent like the soloists? Defer; default silent.
