# Registry snapshot — the workspace index without a CRDT

**Status: shipped as the local `registry1` snapshot; replaces the `workspace2` Loro workspace doc.**

## Why

The workspace doc (sidebar index: spaces, chats, session status) was a Loro CRDT
that kept *history* and had to be replayed and re-exported under a CPU budget. Three
incidents in one week (2026-07-30, 2026-08-04, 2026-08-05) traced to the same root: a
large full-snapshot export could fail where it succeeded on smaller data, so the doc
could not self-compact — the log only grew and cold replays crept toward the limit.
All of that machinery bought convergence-without-an-authority that the topology never
uses: every write already flows through one place, which can simply *be* the authority.

The sidebar's data is a keyed set of rows with independently-updatable scalar fields —
a replicated table, not a document. The registry stores **current state only**:

- 10,000 sessions ≈ a few MB of rows, bounded forever; history is discarded the moment it
  stops being true.
- The registry cold-starts by reading a SQLite table. No replay, no compaction, no
  fold budgets, no wedge class.
- Conflict semantics are unchanged in practice: the Loro schema resolved same-field
  conflicts by last-write-wins map sets; the registry runs the same rule as ~40 lines of
  auditable code (per-field hybrid logical clocks) instead of a black-box CRDT library.

Session docs (transcripts) stay on Loro — concurrent text has no "newest wins" answer,
and that side is measured and healthy.

## Topology

- **RegistryDoc** (`crates/doc/src/registry.rs`): the local table. Authoritative rows
  (engine truth) + a pending-op queue (offline writes, replayed as an overlay for reads).
  Typed API is a drop-in for the old `WorkspaceDoc`. Serialized whole into `DocsStore`
  (`registry1` snapshot row) — restarts keep full state, cursor, and queue.

## Row and op shapes

`Op = {kind, id, op: "upsert"|"update"|"delete", set?: {field: value|null}, hlc, clocks?}`
`Row = {kind, id, seq, deleted, delHlc?, fields, clocks}`

## Merge rules

- HLC strings `"{ms:013}-{counter:06}-{device}"` — lexicographic order = causal order,
  device id breaks ties totally.
- A field set applies iff its clock > the stored clock for that field. `null` deletes the
  field (still a clocked write).
- `update` ops never create or revive rows (the old "never invent rows" discipline);
  `upsert` creates, and revives a tombstone iff newer than `delHlc`.
- `delete` tombstones iff newer than every… no: iff newer than `delHlc`; fields/clocks are
  cleared. Tombstones GC after 30 days; `gcFloor` forces a full reload for cursors older
  than the horizon.
- Re-applying any op is a no-op (`>` compare) — re-pushes are idempotent by construction.

## Cursor + recovery

- `seq` bumps once per accepted batch; every touched row is stamped with it. Delta updates are
  `seq > cursor`.
- Engine behind the client (`seq < cursor`, e.g. wiped local storage): the client keeps
  its rows and re-seeds from them with **original per-field clocks** (`clocks` on the op)
  — the earlier manual repair recipe, automated and lossless.
- Local-only rows on a full reload re-seed the same way; unpushed writes always live in
  the pending queue.

## Migration (instant, on first boot after update)

`WorkspaceHost::open`: if no `registry1` snapshot exists, read the legacy `workspace2`
Loro snapshot, convert every row into pending upsert ops (HLCs derived from the rows' own
timestamps, so genuinely-newer live writes beat migrated values), and save. The UI reads
the overlay immediately — zero visible gap. The old Loro snapshot is no longer opened;
it is retained for rollback.

## Sidebar sections

Signed-in profiles store section metadata in `sidebarSections` rows and session
placement in `sidebarLocations` rows in the same per-user registry as pins.
A session's single `location` field is either a section ID, `pinned`, or empty.
Concurrent placements converge by field clock; renaming and collapsing a section
update separate fields. Deletion marks the section deleted without deleting chats.
Archived chats retain membership. Section IDs are never reused or revived.

Sections and their members keep a placed order through `orderKey` fields, minted
between the neighbours' keys as pins are. A section's key lives on its own
`sidebarSections` row, so concurrent moves of one section converge by clock and
moves of different sections both survive. Sections no move has keyed yet (all of
them before the first reorder, and any an older app creates) follow the keyed
ones by creation. Moving sections is desktop-only and needs an engine advertising
`sidebar-section-order-v1`; apps that predate it keep showing creation order.

The existing sidebar preferences watch includes sections. Pin and section intents
share the UI's ordered optimistic queue; a rejected write reveals the latest
confirmed state. Later pin toggles from older apps still take effect through their
existing `pinned` field clocks. Section ordering itself requires an updated app.

Legacy per-profile UI sections import after an authoritative registry snapshot.
Import skips existing section IDs (including deleted ones) and existing explicit
placements, so replay does not overwrite newer edits. The engine persists the
registry snapshot and outbox before acknowledging migration; only then does the
UI remove its legacy copy. A failed migration retains that copy and can retry on
a new engine attachment/restart. Local workspaces continue using local settings.
The local registry already accepts these row kinds.
