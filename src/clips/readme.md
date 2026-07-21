// A clip should not replace the global transport loop.

// They represent different things:

// Transport loop
//     global playback range
//     e.g. arrangement beats 16–32

// Clip
//     finite musical content
//     e.g. a four-beat MIDI pattern

// A clip can loop while the transport continues forward:

// Transport:  0 ─────────────────────────────── 16

// Clip:       [0 1 2 3][0 1 2 3][0 1 2 3]...

// Or it can play once:

// Transport:  0 ─────────────────────────────── 16

// Clip:       [0 1 2 3] stop

// So I would avoid putting clip bounds directly into Transport.

// Suggested ownership
// Transport
//     owns global position and playing state

// Clip
//     owns local bounds and playback mode

// Scheduler
//     maps transport time into clip-local time
//     materialises note occurrences

// ProbabilityGate
//     evaluates the resulting occurrences

// Playback
//     executes definitive events