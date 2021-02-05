pub mod kbd;

use clap::Clap;
use std::{thread,process};

#[derive(Clap)]
#[clap(version = "0.1", author = "Stewart J. Park <hello@stewartjpark.com>")]
struct Opts {
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    let opts: Opts = Opts::parse();
    let verbose = opts.verbose;

    // HACK: Keyboard detection thread. If it detects change, die and let systemd restart
    thread::spawn(|| {
        kbd::detector::run_forever_until_keyboards_change();
        println!("Keyboard(s) added/removed. Exitting...");

        // Exit code 2 forces systemd to restart this service. Defined in c-hjkl.service
        process::exit(2);
    });

    if let Ok(keyboards) = kbd::enumerator::enumerate_keyboards() {
        let threads = keyboards
            .into_iter()
            .map(|kbd| {
                println!("Keyboard \"{}\" detected.", kbd.name);

                let mut handler = kbd::handler::KeyboardHandler::new(&kbd.device_path, verbose);
                thread::spawn(move || {
                    handler.run_forever();
                })
            })
            .collect::<Vec<_>>();

        for th in threads.into_iter() {
            th.join().unwrap();
        }
    } else {
        panic!("Keyboards cannot be detected.");
    }
}
