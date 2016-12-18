# memol - a music markup language

memol is a music markup language which features:

* **Well-structured** - Essentially, memol describes a score as recursive
  composition of only two constructs: group "[...]" and chord "(...)".
* **Orthogonal** - Some musical elements like scale, chord, voicing, backing
  pattern and expression (note velocity, control change, ...) can be described
  separately and composite them each other. "with" syntax enables (some of)
  them in a unified form.
* **Focused on musical composition** - Language design and implementation help
  trial-and-error of musical composition well.

## Example

	score 'out.0 = { (c E G) | (c E G [B C b]) (c E F A) }

![sample](doc/sample.png)

## Documentation

http://mimosa-pudica.net/memol/tutorial/
