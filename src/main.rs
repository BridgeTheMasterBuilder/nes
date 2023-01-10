use std::error::Error;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use argh::FromArgs;
use nes::core::EmulatorCore;

use nes::gui::Gui;
use nes::util::Config;
use nes::Nes;

// I don't know why but running the clock a little bit slower seems to result in good overall audio sync
const CLOCKRATE: u32 = (1789773.0 / (60.1 / 60.0)) as u32;

/// Emulator for the Nintendo Entertainment System
#[derive(FromArgs)]
struct Args {
    /// nes ROM to load
    #[argh(positional)]
    filename: String,
    /// open debugger
    #[argh(switch, short = 'd')]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = argh::from_env();
    let config = Config::new(&args.filename);

    if !args.debug {
        let mut core = EmulatorCore::new(&config, CLOCKRATE)?;

        let core_thread = thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut nes = Nes::new(CLOCKRATE, &config).unwrap_or_else(|error| {
                    eprintln!("Error: {error}");

                    process::exit(1);
                });

                core.cpu.reset();

                loop {
                    if core.request_termination {
                        nes.save_save_states();
                        break;
                    }

                    nes.run(&mut core).unwrap();
                }
            })
            .unwrap();

        core_thread.join().unwrap();
    } else {
        let core = Arc::new(Mutex::new(EmulatorCore::new(&config, CLOCKRATE)?));
        let gui = Gui::new(core.clone());

        thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let mut nes = Nes::new(CLOCKRATE, &config).unwrap_or_else(|error| {
                    eprintln!("Error: {error}");

                    process::exit(1);
                });

                core.lock().unwrap().cpu.reset();

                loop {
                    if core.lock().unwrap().request_termination {
                        nes.save_save_states();

                        core.lock().unwrap().running = false;
                        break;
                    }

                    nes.run(&mut core.lock().unwrap()).unwrap();
                    // In order to not starve the GUI thread
                    sleep(Duration::from_nanos(1));
                }
            })
            .unwrap();

        let options = eframe::NativeOptions::default();

        eframe::run_native("GUI", options, Box::new(|_cc| Box::new(gui)));
    }

    Ok(())
}
