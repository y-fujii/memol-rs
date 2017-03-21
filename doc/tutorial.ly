<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8">
		<title>memol language overview</title>
		<style>
			* {
				font: 100%/1.5 serif;
				margin:  0;
				padding: 0;
			}
			body {
				margin: 2rem auto;
				max-width: 48rem;
				text-align: justify;
				   -moz-hyphens: auto;
				-webkit-hyphens: auto;
				        hyphens: auto;
			}
			p, ul, dl, h1, h2, h3 {
				margin: 1rem 0;
			}
			dd {
				margin-left: 2rem;
			}
			li {
				margin-left: 1rem;
			}
			h1 {
				font-size: 200%;
				text-align: center;
			}
			h2, h3 {
				margin-top: 2rem;
				font-size: 125%;
				border-bottom: solid 1px #e0e0e0;
			}
			h1, h2, h3, strong, dt {
				font-weight: bold;
			}
			pre, code {
				font-family: monospace, monospace;
				background-color: #f8f8f8;
			}
			pre {
				padding: 0.25rem 0.5rem;
				border: solid 1px #e0e0e0;
			}
			address {
				margin-top:    2em;
				margin-bottom: 2em;
				font-size: small;
				text-align: right;
			}
		</style>
	</head>
<body>

<h1>memol language overview</h1>
<p>XXX: under construction.

<p>memol is a music description language which features:
<dl>
    <dt>Well-structured
	<dd>Essentially, memol describes a score as recursive composition of two
	constructs only: group <code>"[...]"</code> and chord <code>"(...)"</code>.
    <dt>Orthogonal
	<dd>Some musical elements like scale, chord and backing pattern can be
	described independently and composite them each other.  <code>"with"</code>
	syntax enables (some of) them in a unified form.  Separate descriptions of
	expressions (note velocity, control change, ...) are also planned.
    <dt>Focused on musical composition
	<dd>Language design and implementation help trial-and-error of musical
	composition well (in the future).
</dl>
<p>memol does <strong>not</strong> aim to have:
<dl>
	<dt>Complete ability to describe sheet music
	<dd>Sheet music generation may be implemented in the future, but memol
	never will be a complete sheet music description language.
	<a href="http://lilypond.org/">Lilypond</a> is awesome for this purpose (In
	fact, the sheet musics in this page are rendered by Lilypond!).
</dl>
<p>Although the core idea of the language is considered for many years,
the development begun recently so both the language specification and the
implementation are still in a very early stage.  Currently they lack many
features for practical use.

<h2>Build, install and run</h2>

<p>Although any platform that run Rust and JACK are potentially supported, the
main development platform is Linux.  Please make sure that following programs
are installed before building.
<ul>
    <li><a href="http://rust-lang.org/">Rust nightly</a>
    <li><a href="http://crates.io/">Cargo</a>
    <li><a href="http://jackaudio.org/">JACK</a>
</ul>
<p>Using <a href="https://www.rustup.rs/">rustup</a> is an easiest way to
install Rust nightly and Cargo.  Building and installing memol are quite simple
thanks to Cargo; Just type
<pre>
hg clone <a href="https://bitbucket.org/ysfujii/memol-rs/">https://bitbucket.org/ysfujii/memol-rs/</a>
cd memol-rs
cargo install
</pre>
<p>and everything should be done. Note that Windows target must be
<code>*-gnu</code>, not <code>*-msvc</code> due to JACK DLL linking issue.
<p>Current implementation of memol is a simple command line program which emits
MIDI messages to JACK.
<pre>
memol [-c JACK_PORT] FILE
</pre>
<p>memol keeps watching the change of the file and reflects it immediately.  If
<code>'out.begin</code>, <code>'out.end</code> (see below) are specified, memol
automatically seeks and starts playing each time the file has changed.
<p>Since memol supports JACK transport, start/stop/seek operations are synced
with other JACK clients (Currently Timebase is not supported).  Personally I
use <a href="https://github.com/falkTX/Carla/">Carla</a> to manage JACK
connections, LinuxSampler, LV2 plugins, etc.  Many JACK supported DAW like
<a href="http://ardour.org/">Ardour</a> can be used, of course.
<p>JACK_PORT can be specified multiple times and then the memol output port is
being connected to them.

<h2>Hello, twinkle little star</h2>

<pre>
score 'out.0 = { c c G G | A A g _ | f f e e | d d c _ }
</pre>
<lilypond relative="1">
    { c c g' g a a g r f f e e d d c r }
</lilypond>
<p>memol language structure is roughly divided into two layers: inside
<code>{...}</code> and outside.
<p>XXX

<h2>Token</h2>
<p>Newline and whitespace characters have no meanings except before and after
some registerd words, symbol names and numbers.
<pre>
score 'out = {
	[(cEGB)//] |
	(c E G B)
}
</pre>

<h2>Comments</h2>
<pre>
/* This is a comment */
</pre>

<h2>Octave</h2>
<p>memol has a mechanism to avoid annoying octave changing.  If you write a
note in upper case, it has higher pitch than previous one within a octave.  If
in lower case, it has lower pitch within a octave.  <code>"&lt;"</code> and
<code>"&gt;"</code> can be used to make the current octave +1 and -1
respectively.
<pre>
score 'out.0 = { c D E d | &gt; D E &lt; c _ }
</pre>
<lilypond relative="1">
    { c d e d d' e c, r }
</lilypond>

<h2>Accidental</h2>
<p>Sharp and flat pitches are represented as <code>"+"</code>, <code>"-"</code>
respectively.  they must specified every time.  A key signature can be
specified with <code>"with"</code> syntax explained later.
<pre>
score 'out.0 = { c D+ E++ F- }
</pre>
<lilypond relative="1">
    { c dis eisis fes }
</lilypond>

<h2>Group</h2>
<p>Grouping is one of the unique features of memol.  Unlike other language,
absolute duration values are never specified in memol.  Grouping is noted as
<code>"[...]"</code> and it divides the duration equally into child notes and
serializes them.  Group notation can be nested oneself and other notation.
Each child note have an optional number prefix, which represents a relative
ratio.  For example, <code>"[3e 2c]"</code> gives the duration 3/5 to "e" and
2/5 to "c".
<pre>
score 'out.0 = { c | c c | c c c | c [c c c c] [3c c] [2c 3c [c c]] }
</pre>
<lilypond relative="1">
    { c1 c2 c2 \tuplet 3/2 { c2 c2 c2 } c4 c16 c16 c16 c16 c8. c16 \tuplet 3/2 { c8 c8. c32 c32 } }
</lilypond>

<h2>Chord</h2>
<p>Chord is noted as <code>"(...)"</code> and child notes are located in
parallel.  Chord can be nested oneself and other notation.  The note pitch used
to determine the octave of next note is the first child of the chord, not the
last child.
<pre>
score 'out.0 = { (c E G) | (c E G [B C b]) (c E F A) }
</pre>
<lilypond relative="1">
	<c e g>1
	<<
		\new Voice = "one" { \voiceOne \tuplet 3/2 { b'4 c4 b4 } }
		\new Voice = "two" { \voiceTwo <c, e g>2 }
	>>
	<c e f a>2
</lilypond>

<h2>Tie</h2>
<p>Tie is noted by adding <code>"^"</code> after the note which the tie begins.
Composite notes such as group and chord also can be tied.  A tied chord means
all child notes are tied.  A tied group means the last note is tied.
<pre>
score 'out.0 = { [3c c]^c [3c c^] c | (c E G)^(c E G) | (c^ E^ G) (c E G) | c^ E^ G^ (c E G) }
</pre>
<lilypond relative="1">
	\set tieWaitForNote = ##t
	{ c8. c16 ~ c4 c8. c16 ~ c4 <c e g>2~ <c e g>2 <c~ e~ g>2 <c e g>2 c4~ e4~ g4~ <c, e g>4 }
</lilypond>

<h2>Repeat</h2>
<p><code>"/"</code> is semantically equivalent to the previous note, the most
recent simple note or chord in postordered depth-first traversal.  The ties of
child notes are inherited if a target is composite (the tie attached to itself
is not inherited).
<pre>
score 'out.0 = { (c E G) / | (c [E /]) | ([3c E]) / }
</pre>
<lilypond relative="1">
	<c e g>2 <c e g>2
	<<
		\new Voice = "one" { \voiceOne e e }
		\new Voice = "two" { \voiceTwo c1 }
	>>
	c4. e8 c4. e8
</lilypond>

<h2>Score level composition</h2>
<p>XXX: parallel, sequence, stretch, repeat, ...
<pre>
score 'out.0 = [ 2:{ c D E d } ( { E F G A | c c c c } 3/4 { D E F } ) ]
</pre>

<h2>Score symbols</h2>
<p>Score symbols probably works as you expected.  It is possible to use symbols
defined after their location.  Defining the same name symbol more than once
causes error.
<pre>
score 'part_a = { e F G A }
score 'part_b = { c D E F }
score 'out.0 = ('part_a 'part_b)
</pre>

<h2><code>"with"</code> syntax</h2>
<p><code>"with"</code> syntax is one of the unique feature of memol that
enables high level music description.
<p>XXX
<pre>
score 'chord   = { (c E G B) (D F G B) | (c E G B) }
score 'pattern = { [$q0 Q1 Q2 q1] ($q0 Q1 Q2 Q3) }
score 'out.0   = 2:'pattern with q = 'chord
</pre>
<lilypond relative="1">
	c8 e8 g8 e8 <d f g b>2 c8 e8 g8 e8 <c e g b>2
</lilypond>
<pre>
score 'a_major = { (c+DEF+G+AB) }
score 'out.0   = { ... } with _ = 'a_major
</pre>

<h2>Value track</h2>
<p>XXX: Specification/implementation is not completed.
<pre>
value 'out.0.velocity = { [3 4] 3 2 | 2..4 3 } / {4}
value 'out.0.offset = gaussian / {128}
value 'out.0.cc11 = { 3..4 | 3..1 } / {4}
</pre>

<h2>Articulation, arpeggio, sustain pedal</h2>
<p>XXX: Not implemented yet.
<p>Some special syntax for articulation, arpeggio, sustain pedal may be added
in the future.
<pre>
  @N(XXX) : arpeggio
       X~ : legato
(default) : non-legato
       X' : staccato
</pre>

<h2>MIDI channels</h2>
<p>Although this is out of the language specification, current implementation
maps the score to MIDI outputs by variable names: <code>'out.0</code> ..
<code>'out.15</code> are mapped to MIDI channel 1 .. 16.

<h2>Begin/end position</h2>
<p>XXX
<pre>
'out.begin = { 0}
'out.end   = {24}
</pre>

<address>Yasuhiro Fujii &lt;y-fujii at mimosa-pudica.net&gt;</address>

</body>
</html>
