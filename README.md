# Engine Lab

Engine Lab is a Rust experiment for modelling the timing and event flow of a music engine. The current project focuses on the core domain objects that sit between arrangement data and final playback events: tempo, transport, clips, scheduling, probability, and playback event mapping.

The code is structured as a library crate with a small example binary in `src/main.rs`.

## Current Shape

- `tempo` owns musical timing configuration: BPM, sample rate, time signature, and conversions between seconds, samples, and beats.
- `transport` owns global playback state and position: playing, paused, stopped, current sample position, beat position, bar position, and elapsed time.
- `clips` models reusable musical content and where it appears in arrangement time. A `Clip` has local length and playback mode, while a `ClipPlacement` maps a clip ID onto a global beat range.
- `scheduler` stores scheduled notes, keeps them sorted, and materializes note-on and note-off edges inside a lookahead window.
- `playback` maps scheduled events into playback-domain events and applies probability gates so rejected note-on events do not emit matching note-off events later.

The intended flow is:

```text
Tempo + Transport
        |
        v
Scheduler lookahead window
        |
        v
Scheduled note events
        |
        v
ProbabilityGate
        |
        v
PlaybackEvent
```

## Design Notes

Transport and clips deliberately describe different parts of playback:

- Transport represents the global playback position and state.
- Clips represent finite musical content with local length and looping behavior.
- The scheduler is responsible for mapping transport time into scheduled occurrences.
- Playback receives definitive events that are ready to execute.

This keeps clip bounds out of the global transport and lets a clip loop while the transport continues moving forward through arrangement time.

## Quick Start

Run the example binary:

```sh
cargo run
```

Run the test suite:

```sh
cargo test
```

## Example

The binary now demonstrates the full end-to-end flow:

- builds notes, clips, placements, and routes through `ClipRouter`
- materializes an `ArrangementView`
- advances `PlaybackPipeline` against `Transport` + `Tempo`
- emits timed playback events (`sample_position` deadlines)
- evaluates all-or-nothing mutation guards using both anonymous and named batch APIs
- performs reset panic-note-off flushing

Run it with:

```sh
cargo run
```

Inspect the source for the complete walkthrough in `src/main.rs`.

## Crate Layout

```text
src/
  lib.rs
  main.rs
  clips/
  playback/
  scheduler/
  tempo/
  transport/
```

The public module exports are collected in `src/lib.rs`, and each domain folder exposes its main types through its own `mod.rs`.

## Crate Docs

- [clips crate docs](src/clips/README.md)
- [playback crate docs](src/playback/README.md)
- [scheduler crate docs](src/scheduler/README.md)
- [tempo crate docs](src/tempo/README.md)
- [transport crate docs](src/transport/README.md)

## Architecture Map

Execution-oriented flow:

1. Timing source: [tempo crate docs](src/tempo/README.md) + [transport crate docs](src/transport/README.md)
2. Arrangement resolution: [clips crate docs](src/clips/README.md)
3. Event materialization window: [scheduler crate docs](src/scheduler/README.md)
4. Playback mapping and gating: [playback crate docs](src/playback/README.md)

Related scheduler detail:

- [Loop iteration slicing strategy](src/scheduler/README.md#loop-iteration-slicing)

Pipeline summary:

```text
Tempo + Transport -> Clips (ArrangementView/ClipRouter) -> Scheduler -> Playback (ProbabilityGate + PlaybackEvent)
```
