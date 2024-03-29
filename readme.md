# memol - a music description language

![Build Status](https://github.com/y-fujii/memol-rs/actions/workflows/ci.yml/badge.svg)

memol is a music description language which features:

- **Well-structured**
    - Essentially, a score is described as recursive compositions of two
      constructs: group `[...]` and chord `(...)`.
- **Orthogonal**
    - Some musical elements like scale, chord and backing pattern can be
      described independently and composite them each other.  `with` syntax
      enables (some of) them in a unified form.  Expressions (note velocity,
      control change, ...) are also described separately.
- **Focused on musical composition**
    - Language design and implementation help trial-and-error of musical
      composition well (in the future).  Unlike score typesetting languages,
      memol also focuses on describing time-dependent value used for MIDI
      control changes, etc.

## Example

    score $out.0() = { (c E G) | (c E G [B C b]) (c E F A) }

![sample](doc/sample.png)

## Documentation

<https://mimosa-pudica.net/memol/tutorial/>
