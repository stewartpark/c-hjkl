mod kbd;

use clap::Clap;
use std::thread;

#[derive(Clap)]
#[clap(version = "0.1", author = "Stewart J. Park <hello@stewartjpark.com>")]
struct Opts {
    #[clap(short, long)]
    verbose: bool,
}

fn main() {
    let opts: Opts = Opts::parse();
    let verbose = opts.verbose;

    if let Ok(keyboards) = kbd::enumerator::enumerate_keyboards() {
        let threads = keyboards
            .into_iter()
            .map(|kbd| {
                println!("Keyboard \"{}\" detected.", kbd.name);

                thread::spawn(move || {
                    let mut handler = kbd::handler::KeyboardHandler::new(&kbd.device_path, verbose);
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
