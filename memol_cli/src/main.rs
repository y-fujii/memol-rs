// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use gumdrop::Options;
use memol_util::*;
use std::*;

#[derive(gumdrop::Options)]
struct ArgOptions {
    #[options(free)]
    file: path::PathBuf,
    #[options(help = "Be verbose.")]
    verbose: bool,
    #[options(help = "Generate a MIDI file and exit.")]
    batch: bool,
    #[options(help = "Use JACK.")]
    jack: bool,
    #[options(help = "Use plugins.")]
    plugin: bool,
    #[options(help = "Accept remote connections.")]
    any: bool,
    #[options(help = "Connect to specified ports.", meta = "PORT")]
    connect: Vec<String>,
}

fn compile(path: &path::Path, verbose: bool) -> Option<Vec<memol::midi::Event>> {
    let timer = time::Instant::now();
    let rng = memol::random::Generator::new();
    let result = memol::compile(&rng, &path).and_then(|e| memol::assemble(&rng, &e));
    let elapsed = timer.elapsed();
    if verbose {
        eprintln!(
            "compile time: {} ms",
            elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000
        );
        eprintln!(" event count: {}", result.as_ref().map(|evs| evs.len()).unwrap_or(0));
    }
    match result {
        Err(e) => {
            println!("{}", e);
            None
        }
        Ok(v) => Some(v),
    }
}

fn main() {
    let f = || -> Result<(), Box<dyn error::Error>> {
        let args: Vec<_> = env::args().collect();
        let opts = match ArgOptions::parse_args_default(&args[1..]) {
            Ok(e) => e,
            Err(_) => return Err(ArgOptions::usage().into()),
        };
        if opts.file == path::PathBuf::new() {
            return Err(ArgOptions::usage().into());
        }

        // generate MIDI file.
        if opts.batch {
            if let Some(events) = compile(&opts.file, opts.verbose) {
                let smf = memol::smf::generate_smf(&events, 480);
                fs::write(opts.file.with_extension("mid"), smf)?;
            }
            return Ok(());
        }

        // initialize a player.
        let addr = (
            if opts.any {
                net::Ipv6Addr::UNSPECIFIED
            } else {
                net::Ipv6Addr::LOCALHOST
            },
            27182,
        );
        let mut player: Box<dyn player::Player> = match (opts.jack, opts.plugin) {
            (true, false) => Box::new(player_jack::Player::new("memol")?),
            (false, true) => Box::new(player_net::Player::new(addr)?),
            _ => {
                #[cfg(all(target_family = "unix", not(target_os = "macos")))]
                let player = player_jack::Player::new("memol");
                #[cfg(not(all(target_family = "unix", not(target_os = "macos"))))]
                let player = player_net::Player::new(addr);
                Box::new(player?)
            }
        };
        for port in opts.connect {
            player.connect_to(&port)?;
        }

        // main loop.
        loop {
            if let Some(events) = compile(&opts.file, opts.verbose) {
                let bgn = match events.get(0) {
                    Some(ev) => ev.time.max(0.0),
                    None => 0.0,
                };
                player.set_data(&events);
                player.seek(bgn);
                player.play();
            }

            notify::wait_file(&opts.file)?;
        }
    };
    if let Err(e) = f() {
        eprintln!("{}", e);
    }
}
