<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8">
		<meta name="viewport" content="width=device-width,initial-scale=1">
		<title>memol language overview</title>
		<link href="https://fonts.googleapis.com/css?family=Inconsolata|Source+Sans+Pro:700|Source+Serif+Pro&display=swap" rel="stylesheet">
		<style>
			* {
				font: inherit;
				margin:  0;
				padding: 0;
			}
			body {
				font: 100%/1.5 "Source Serif Pro", serif;
				margin: 0 auto;
				padding: 0 1em;
				max-width: 50rem;
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
			li ul {
				margin: 0;
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
				font-family: "Source Sans Pro", sans-serif;
				font-weight: bold;
			}
			pre, code {
				font-family: "Inconsolata", monospace;
				word-break: break-all;
				background-color: #f8f8f8;
			}
			pre {
				white-space: pre-wrap;
				text-align: left;
				padding: 0.25rem 0.5rem;
				border: solid 1px #e0e0e0;
			}
			address {
				margin-top:    2em;
				margin-bottom: 2em;
				font-size: small;
				text-align: right;
			}
			img {
				max-width: 100%;
			}
		</style>
	</head>
<body>

<h1>memol language overview</h1>
<p><strong style="color: #e02020">WARNING: The documentation is very incomplete
and different from the latest implementation.</strong>

<p>memol is a music description language which features:
<dl>
	<dt>Well-structured
	<dd>Essentially, a score is described as recursive compositions of two
	constructs: group <code>"[...]"</code> and chord <code>"(...)"</code>.
	<dt>Orthogonal
	<dd>Some musical elements like scale, chord, and backing pattern can be
	described independently and composite them each other.  <code>"with"</code>
	syntax enables (some of) them in a unified form.  Expressions (note
	velocity, control change, ...) are also described separately.
	<dt>Focused on musical composition
	<dd>Language design and implementation help trial-and-error of musical
	composition well (in the future).  Unlike score typesetting languages,
	memol also focused on describing time-dependent value used for MIDI control
	changes, etc.
	<dt>Extendable with arbitrary programming languages
	<dd>(Planned. Not implemented yet.)
</dl>
<p>memol does <strong>not</strong> aim to have:
<dl>
	<dt>Complete ability to describe music typesetting
	<dd>Staff notation generation may be implemented in the future, but memol
	never will be a complete score typesetting language.
	<a href="http://lilypond.org/">Lilypond</a> is awesome for this purpose (In
	fact, the sheet musics on this page are rendered by Lilypond!).
</dl>
<p>Here is an example written in (current) memol language.
<pre style="font-size: 87.5%">
/* Gymnopedie No. 1, Erik Satie */

score $melody_common() = {
    _  | _    | _    | _  &lt; | _FA | gfc  | bCD  | a    |
    f^ | f^   | f^   | f    | _FA | gfc  | bCD  | a    |
    C  | F &gt;  | e^   | e^   | e   | ABC- | Edb  | Dc-b |
    D^ | D:2D | EF-G | Ac-D | Edb | D^   | D:2D | G
}

score $melody() = [
    $melody_common() { &lt; F  | baB   | CDE  | cDE  | f:2G  | C- | D }
    $melody_common() { &lt; F- | bC-F- | edc- | Edc- | f-:2G | C- | D }
]

score $chord_common() = [
    repeat 8 { (gBDF_)  | (dACF_) } {
    (fACF_)  | (bBDF_)  | (EGB__)   | (EBDG_)  | (dF-AD_) | (aAC-E_) | (DGBE_)  | (DDGBE) |
    (Dc-EAD) | (Dc-FAD) | (DAC-F-_) | (DAC-E_) | (DGBE_)  | (DDGBE)  | (Dc-EAD) | (EBEG_) }
]

score $chord() = [
    $chord_common()
    { (fACF_)  | (bBDF_)   | (ECEA_)  | (EACFA)   | E (EbAD) (EEBD) | (AgC-EA_) | (daD&lt;DFA)  }
    $chord_common()
    { (eADF-A) | (eAC-F-_) | (eC-EA_) | (eAC-F-A) | e (ebAD) (eeBD) | (AgC-EA_) | (daD&lt;DF-A) }
]

score $pattern() = [
    repeat 36 { @Q0 q0 ^ (/ @Q1 Q1 Q2 Q3 Q4):2 }
    { (@Q0 q0 [_ (@Q1 Q1 Q2 Q3) /]) | (@Q0 q0 @Q1 Q1 Q2 @Q3 Q3 Q4 Q5) | / }
]

score $out.0() = [
    _ ( $melody() repeat 2 $pattern() with q = $chord() ) with * = repeat 78 { (ABC+DEF+G) } _
]

value $out.0.offset()   = $gauss() / 512
value $out.0.velocity() = $gauss() / 64 + (if $note.nth() == 0 then 4/8 else 3/8)
value $out.0.cc64()     = [ repeat 79 { 0 1:23 } { 0 } ]
value $out.tempo()      = 2/5
</pre>

<p>Although the core idea of the language is considered for many years,
the development begun recently so both the language specification and the
implementation are still in a very early stage.  Currently they lack many
features for practical use.

<h2>Current status</h2>
<ul>
<li>70% of primitive/low-level features are implemented.
<li>0% of middle-level features are implemented.
	<ul>
		<li>chord notation, auto-voicing, auto-articulation, language
		extension API, etc.
	</ul>
<li>0% of syntax is stabilized.
<li>20% of the documentation is completed.
<li>10% of non-language features are implemented.
</ul>

<h2>Download pre-built binaries</h2>
<p><a href="https://github.com/y-fujii/memol-rs/releases">See the GitHub Release page.</a>
<p>Note that macOS binaries are never tested since I don't have a Mac...

<h2>Build and install</h2>
<p>This section is only necessary for users who want to build memol themselves.
<p>Although memol can run potentially on any platforms which support Rust and
JACK, I develop it primarily on Linux and sometimes test it on Windows
(<code>x86_64-pc-windows-gnu</code> target).  Please make sure that the
following programs are installed and configured properly.
<ul>
	<li><a href="http://rust-lang.org/">Rust</a> (build dependency)
	<li><a href="http://jackaudio.org/">JACK</a> (runtime dependency, optional but strongly recommended)
</ul>
<p>Building and installing memol are quite simple thanks to Cargo; Just type
<pre>
$ cargo install --git <a href="https://github.com/y-fujii/memol-rs/">https://github.com/y-fujii/memol-rs/</a> memol_cli
</pre>
<p>and everything should be done.
<p>A recent version of memol has an experimental GUI program.
<a href="https://clang.llvm.org/">Clang</a> must be installed to build one.
<pre>
$ cargo install --git <a href="https://github.com/y-fujii/memol-rs/">https://github.com/y-fujii/memol-rs/</a> memol_gui
</pre>
<p style="text-align: center"><img src="memol_gui.png" style="width: 50%; border: 1px solid #e0e0e0">
<p>Building <code>memol_gui</code> on Windows requires a workaround due to
<a href="https://github.com/rust-lang/rust/issues/47048">issue #47048</a> for
now.  I recommended using prebuild binaries above.

<h2>Run</h2>
<pre>
$ memol_cli
Usage: memol_cli [options] FILE
  -v, --verbose       Be verbose.
  -b, --batch         Generate a MIDI file and exit.
  -j, --jack          Use JACK.
  -p, --plugin        Use plugins.
  -a, --any           Accept remote connections.
  -c, --connect PORT  Connect to specified ports.

$ memol_gui
Usage: memol_gui [options] [FILE]
  -w, --wallpaper FILE  Set a background image.
  -j, --jack            Use JACK.
  -p, --plugin          Use plugins.
  -a, --any             Accept remote connections.
  -c, --connect PORT    Connect to specified ports.
</pre>
<p>There are three ways to interact with other applications.
<ul>
	<li>Generates MIDI file.
	<li>Use JACK (recommended if available).
	<li>Use the VST plugin.
</ul>
<p>XXX
<p>XXX
<p>PORT can be specified multiple times and then the memol output port is being
connected to them.
<p>memol keeps watching the change of the file and reflects it immediately.  If
<code>$out.begin</code>, <code>$out.end</code> (see below) are specified, memol
automatically seeks and starts playing each time the file has changed.
<p>Since memol supports JACK transport, start/stop/seek operations are synced
with other JACK clients.  Personally I
use <a href="https://github.com/falkTX/Carla/">Carla</a> to manage JACK
connections, plugins, etc.  Many JACK supported DAW like
<a href="http://ardour.org/">Ardour</a> can be used, of course.

<h2>Hello, twinkle little star</h2>

<pre>
score $out.0() = { c c G G | A A g _ | f f e e | d d c _ }
</pre>
<lilypond relative="1">
	{ c c g' g a a g r f f e e d d c r }
</lilypond>
<p>memol language structure is roughly divided into two layers: inside of
<code>{...}</code> and outside of.  Both layers have similar syntax and similar
semantics, but different.  Inside <code>{...}</code>, a sequence is split by
<code>"|"</code> and each part gets the unit time length regardless of the
number of the elements.
<p>XXX
<p>Outside <code>{...}</code>, on the other hand, all elements have specific
lengths.
<p>XXX

<h2>Token</h2>
<p>Unlike common programming languages, newline and whitespace characters have
no meanings at most locations.  The exception is the one before or after the
registered words (<code>"score"</code>, <code>"value"</code>, etc.), symbol
names (<code>"$name"</code>) and numbers.  For example, <code>"(cEGB)"</code>
and <code>"( c E G B )"</code> have the same meaning, <code>"scoreabc"</code>
is different from <code>"score abc"</code>.

<h2>Comments</h2>
<pre>
/* This is a comment */
</pre>

<h2>Octave</h2>
<p>memol has a mechanism to avoid annoying octave changing.  If you write a
note in the upper case, it has a higher pitch than the previous one within an
octave.  If in the lower case, it has a lower pitch within an octave.
<code>"&lt;"</code> and <code>"&gt;"</code> can be used to make the current
octave +1 and -1
respectively.
<pre>
score $out.0() = { c D E d | &gt; D E &lt; c _ }
</pre>
<lilypond relative="1">
	{ c d e d d' e c, r }
</lilypond>

<h2>Accidental</h2>
<p>Sharp and flat pitches are represented as <code>"+"</code>, <code>"-"</code>
respectively.  they must specified every time.  A key signature can be
specified with <code>"with"</code> syntax explained later.
<pre>
score $out.0() = { c D+ E++ F- }
</pre>
<lilypond relative="1">
	{ c dis eisis fes }
</lilypond>

<h2>Group</h2>
<p>Grouping is one of the unique features of memol.  Unlike other languages,
absolute duration values are never specified in memol.  Grouping is noted as
<code>"[...]"</code> and it divides the duration equally into child notes and
serializes them.  Group notation can be nested oneself and other notation.
Each child note has an optional number prefix, which represents a relative
ratio.  For example, <code>"[e:3 c:2]"</code> gives the duration 3/5 to "e" and
2/5 to "c".
<pre>
score $out.0() = { c | c c | c c c | c [c c c c] [c:3 c] [c:2 c:3 [c c]] }
</pre>
<lilypond relative="1">
	{ c1 c2 c2 \tuplet 3/2 { c2 c2 c2 } c4 c16 c16 c16 c16 c8. c16 \tuplet 3/2 { c8 c8. c32 c32 } }
</lilypond>

<h2>Chord</h2>
<p>Chord is noted as <code>"(...)"</code> and child notes are located in
parallel.  Chord can be nested oneself and other notation.  The note pitch to
determine the next note octave is the first child of the chord, not the last
child.
<pre>
score $out.0() = { (c E G) | (c E G [B C b]) (c E F A) }
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
<p><strong style="color: #e02020">Recently, tie related specifications are
fundamentally changed.  It might be buggy for now.</strong>
<p>Tie is noted by adding <code>"^"</code> after the note from which the tie
begins.  Composite notes such as group and chord also can be tied.  A tied
chord means all child notes are tied.  A tied group means the last note is
tied.
<pre>
score $out.0() = { [c:3 c]^c [c:3 c^] c | (c E G)^(c E G) | (c^ E^ G) (c E G) }
</pre>
<lilypond relative="1">
	c8. c16 ~ c4 c8. c16 ~ c4 <c e g>2~ <c e g>2 <c~ e~ g>2 <c e g>2
</lilypond>

<h2>Repeat</h2>
<p><code>"/"</code> is semantically equivalent to the previous note, the most
recent simple note or chord in postordered depth-first traversal.  The ties of
child notes are inherited if a target is composite (the tie attached to itself
is not inherited).
<pre>
score $out.0() = { (c E G) / | (c [E /]) | ([c:3 E]) / | c^(/ E)^(/ G) }
</pre>
<lilypond relative="1">
	\set tieWaitForNote = ##t
	<c e g>2 <c e g>2
	<<
		\new Voice = "one" { \voiceOne e e }
		\new Voice = "two" { \voiceTwo c1 }
	>>
	c4. e8 c4. e8
	c4~ e4~ g4~ <c, e g>4
</lilypond>

<h2>Score level composition</h2>
<p>Score elements can be composited by <code>"[...]"</code> and
<code>"(...)"</code>, which looks similar to group and chord syntax;
<code>"[...]"</code> serializes its child elements and <code>"(...)"</code>
locates its child elements in parallel.  Additionally,
<code>repeat N element</code> syntax is used for repeating,
<code>stretch N/M element</code> for stretching time.
<pre>
score $out.0() = [ repeat 2 { c D E d } ( { E F G A | c c c c } stretch 3/4 { D E F } ) ]
</pre>

<h2>Score symbols</h2>
<p>Score symbols is similar to constant variables in common programming
languages and probably works as you expected.  It is possible to use symbols
defined after their location.  Defining the same name symbol more than once
causes error.
<pre>
score $part_a() = { e F G A }
score $part_b() = { c D E F }
score $out.0()  = ( $part_a() $part_b() )
</pre>

<h2><code>"with"</code> syntax</h2>
<p><code>"with"</code> syntax is one of the unique feature of memol that
enables high level music description.
<p>XXX
<p>XXX
<pre>
score $chord()   = { (c E G B) (D F G B) | (c E G B) }
score $pattern() = { [@q0 q0 Q1 Q2 q1] (@q0 q0 Q1 Q2 Q3) }
score $out.0()   = repeat 2 $pattern() with q = $chord()
</pre>
<lilypond relative="1">
	c8 e8 g8 e8 <d f g b>2 c8 e8 g8 e8 <c e g b>2
</lilypond>
<p>
<p>The special symbol <code>"_"</code> corresponds to the note symbols
<code>"abcdefg"</code>.  It can be used to change a key signature.  
<pre>
score $a_major() = { (c+DEF+G+AB) }
score $out.0()   = { ... } with * = $a_major()
</pre>

<h2>Value track</h2>
<p>Value track has the similar syntax to score track and it describes the
time-dependent value.
<p>XXX
<p>Outside <code>"{...}"</code>, arithmetic operators
(<code>+, -, *, /</code>), comparison operators
(<code>==, !=, &lt;=, &gt;=</code>), logical operators
(<code>||, &amp;&amp;, !</code>) and a branch syntax
(<code>"if A then B else C"</code>) can be applied.
<p>XXX
<pre>
value $out.tempo()      = 1 / 2
value $out.0.velocity() = { [3:3 4] 3 2 | 2..4 3 } / 8 + { 0..1 | 1..2 } / 4
value $out.0.offset()   = $note.nth() / 32 + $gauss() / 256
value $out.0.duration() = $note.len() * 6 / 8 + 1 / 8
value $out.0.cc11()     = { 3..* | 4..* | *..* | *..1 } / 4
</pre>
<p>There are some special symbols: <code>$note.len(), $note.cnt(), $note.nth()</code>.
<p>XXX
<p>XXX
<pre>
score $top_notes()  = filter $note.nth() == 0 { (cEGB) | (cEFA) }
score $transposed() = transpose 3 { (cEGB) | (cEFA) }
score $sliced()     = slice 0 3/2 { (cEGB) | (cEFA) }
</pre>

<h2>MIDI channels</h2>
<p>WARNING: This specification will be changed.
<p>Although this is out of the language specification, current implementation
maps the score to MIDI outputs by variable names: <code>$out.0</code> ..
<code>$out.15</code> are mapped to MIDI channel 1 .. 16.

<h2>Begin/end position</h2>
<p>XXX
<pre>
value $out.begin() =  0
value $out.end()   = 24
</pre>

<h2>Import</h2>
<pre>
import "other_file.mol"
</pre>


<address>Yasuhiro Fujii &lt;y-fujii at mimosa-pudica.net&gt;</address>

</body>
</html>
