BUILD/Owner4 reverse-engineering proofs

Run the seven ZV43*.java scripts through the existing read-only Ghidra
instance. Each takes one absolute output-file path. Projection probes need
the local GOG SINCOS.DAT; no game data is bundled here. Numeric raw callback
outputs remain private under Z_NATIVE_TRACE_DIR.

cargo test original_world_contact_game::trail::
cargo test original_world_contact_game::crane::
cargo test original_world_contact_game::trail_blast::

These execute deterministic generated inputs and public numeric hash goldens
without private traces. Set Z_NATIVE_TRACE_DIR to the private audit folder
to add full raw-record comparisons. Crane queues and playback fragment-
template comparisons additionally use the actual private original tables.
Eight explicitly ignored campaign diagnostics require the local resource
package and can be invoked with --ignored. Their names distinguish an
injected M1 crane from the authored M14 neutral-crane takeover fixture.

Recorded leaf boundaries are explicit in each Java header: draw/audio/world
queries, the four no-effect common callees and terrain raster leaves are
not claimed as complete runtime ports by these numerical comparisons.
No second Ghidra database/owner is required or permitted by this workflow.

Frontend endpoint checks use the actual Rust diagnostic logs:
 node tools/check-native-building-trails.cjs <TRAIL_ROWS.log> <INHERITED_FX_ROWS.log>
 node tools/check-native-crane-work.cjs <CRANE_WORK_ROWS.log>
Set Z_TRAIL_ROWS=1 for those Rust diagnostics. The RPC harness uses the
existing Wasm only for a valid state envelope; the new row payloads come
from the tested Rust callbacks. It does not pretend an old Wasm executed
the new source path.

Integrated check-all reads actual Rust callback logs through:
Z_NATIVE_TRAIL_ROWS_LOG, Z_NATIVE_INHERITED_FX_ROWS_LOG,
Z_NATIVE_CRANE_ROWS_LOG. The explicit CLI arguments remain supported.
Generate them with Z_TRAIL_ROWS=1 and --nocapture on, respectively,
late_reader_emits_1472_rows (--ignored), retained_source250_branch,
and authored_mission14_neutral_crane_takeover (--ignored).
These checks require the separately authorized private campaign/sprite pack.
No synthetic or stale-log fallback is substituted when inputs are absent.
