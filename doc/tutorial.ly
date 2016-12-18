<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8">
		<title>memol language tutorial</title>
		<style>
			* {
				font: 100%/1.5 serif;
				margin:  0;
				padding: 0;
			}
			body {
				margin: 2em auto;
				max-width: 48em;
				text-align: justify;
				   -moz-hyphens: auto;
				-webkit-hyphens: auto;
						hyphens: auto;
			}
			p, ul, dl, h1, h2, h3 {
				margin: 1em 0;
			}
			dd {
				margin-left: 2em;
			}
			li {
				margin-left: 1em;
			}
			h1 {
				font-size: 200%;
				text-align: center;
			}
			h2, h3 {
				font-size: 150%;
			}
			h1, h2, h3, strong, dt {
				font-weight: bold;
			}
			pre, code {
				font-family: monospace, monospace;
			}
		</style>
	</head>
<body>

<h1>memol language Tutorial (under construction)</h1>

<p>memol is a music markup language which features:
<dl>
    <dt>Well-structured
	<dd>Essentially, memol describes a score as recursive composition of only
	two constructs: group <code>"[...]"</code> and chord <code>"(...)"</code>.
    <dt>Orthogonal
	<dd>Some musical elements like scale, chord, voicing and backing pattern
	can be described independently and composite them each other.
	<code>"with"</code> syntax enables (some of) them in a unified form.
	Separate descriptions of expressions (note velocity, control change, ...)
	are also planned.
    <dt>Focused on musical composition
	<dd>Language design and implementation help trial-and-error of musical
	composition well.
</dl>
<p>memol does <strong>not</strong> aim to have:
<dl>
	<dt>Complete ability to describe sheet music
	<dd>Sheet music generation may be implemented in the future, but memol
	never will be a complete sheet music description language.  <a
	href="http://lilypond.org/">Lilypond</a> is awesome for this purpose (In
	fact, the sheet musics in this page are rendered by Lilypond!).
</dl>
<p>Although the core idea of the language is considered for many years,
the development begun recently so both the language specification and the
implementation are still in a very early stage.  Currently they lack many
features for practical use.

<h2>Build, install and run</h2>

<p>Current supported platform is Linux only.  Please make sure that following
programs are installed.
<ul>
    <li><a href="http://rust-lang.org/">Rust</a>
    <li><a href="http://crates.io/">Cargo</a>
    <li><a href="http://jackaudio.org/">Jack</a>
</ul>
<p>Using <a href="https://www.rustup.rs/">rustup</a> is an easiest way to
install Rust and Cargo.  Building and installing memol are quite simple thanks
to Cargo; Just type
<pre>
hg clone https://bitbucket.org/ysfujii/memol-rs/
cd memol-rs
cargo install
</pre>
<p>and everything should be done.
<p>Current implementation of memol is a simple command line program which emits
MIDI messages to Jack.
<pre>
memol [-c JACK_PORT] [-s SEEK_TIME] FILE
</pre>
<p>XXX
<p>memol keeps watching the change of the file and reflects it immediately.
Since memol supports Jack transport, start/stop/seek operations can be done by
external programs.

<h2>Hello, twinkle little star</h2>

<p>XXX
<pre>
score 'out.0 = { c c G G | A A g _ | f f e e | d d c _ }
</pre>
<lilypond relative="1">
    { c c g' g a a g r f f e e d d c r }
</lilypond>

<h2>Octave</h2>
<p>memol has a mechanism to avoid annoying octave changing.  If you write a
note in upper case, it has higher pitch than previous one within a octave.  If
in lower case, it has lower pitch within a octave.  <code>"&lt;"</code> and
<code>"&gt;"</code> can be used to make the current octave +1 and -1
respectively.
<pre>
score 'out.0 = { c D E d | < D E > c _ }
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
<p>Grouping is one of the charasteristic features of memol.  Unlike other
language, absolute duration values are never specified in memol.  Group
notation divides the duration equally into child notes and serializes them.
Group notation can be nested itself and other notation. Each child note have an
optional number prefix, which represents a relative ratio. For example,
<code>"[3e 2c]"</code> gives "e" to the duration 5/3 and "c" to 2/5.
<pre>
score 'out.0 = { c | c c | c c c | c [c c c c] [3c c] [2c 3c [c c]] }
</pre>
<lilypond relative="1">
    { c1 c2 c2 \tuplet 3/2 { c2 c2 c2 } c4 c16 c16 c16 c16 c8. c16 \tuplet 3/2 { c8 c8. c32 c32 } }
</lilypond>

<h2>Chord</h2>
<p>XXX
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
<p>XXX
<pre>
score 'out.0 = { [3c c]^c [3c c^] c | (c E G)^(c E G) | (c^ E^ G) (c E G) | c^ E^ G^ (c E G) }
</pre>
<lilypond relative="1">
	\set tieWaitForNote = ##t
	{ c8. c16 ~ c4 c8. c16 ~ c4 <c e g>2~ <c e g>2 <c~ e~ g>2 <c e g>2 c4~ e4~ g4~ <c, e g>4 }
</lilypond>

<h2>Repeat</h2>
<p>XXX
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
<p>XXX
<pre>
score 'out.0 = [ { c D E d } ( { E F G A | c c c c } 2 { c D E F } ) ]
</pre>

<h2>Variable</h2>
<p>XXX
<pre>
score 'part_a = { e F G A }
score 'part_b = { c D E F }
score 'out.0 = ('part_a 'part_b)
</pre>

<h2>"With" syntax</h2>
<pre>
score 'chord   = { (c E G B) (D F G B) | (c E G B) }
score 'pattern = { [$q0 Q1 Q2 q1] ($q0 Q1 Q2 Q3) }
score 'out.0   = 2'pattern with q = 'chord
</pre>
<lilypond relative="1">
	c8 e8 g8 e8 <d f g b>2 c8 e8 g8 e8 <c e g b>2
</lilypond>

<h2>Value</h2>
<p>XXX
<pre>
value 'vel = { ... }

</body>
</html>
