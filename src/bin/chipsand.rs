use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

use structopt::StructOpt;

use chipsandlib::{cpu, save_screen_buffer};
use chipsandlib::display::Display;
use chipsandlib::input::{from_sdl2_event, Control};
use chipsandlib::mmu::MMU;
use anyhow::{Context, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "chipsand")]
struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    rom: PathBuf,
}

fn emulation_loop(mut cpu: cpu::CPU) {
    loop {
        cpu.cycle();
        if cpu.cycles % 4096 == 0 {
            thread::sleep(time::Duration::from_millis(1));
        }
    }
}

fn main() -> Result<()> {
    let opt:Opt = Opt::from_args();
    let data = fs::read(&opt.rom).context(format!("unable to open '{}'", opt.rom.display()))?;
    let sdl_context= sdl2::init().map_err(|s|anyhow::anyhow!(s))?;
    let mut display = Display::new(&sdl_context);
    let (tx, rx) = mpsc::sync_channel(0);
    let (tx_events, rx_events) = mpsc::channel();
    thread::spawn(move || {
        let mmu = MMU::new(data, tx, rx_events);
        let mut cpu = cpu::CPU::new(mmu);
        cpu.reset();
        emulation_loop(cpu);
    });
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut redraws = 0u64;
    loop {
        for event in event_pump.poll_iter() {
            match from_sdl2_event(event) {
                None => {},
                Some(Control::Quit) => std::process::exit(0),
                Some(x) => tx_events.send(x)?
            }
        }
        match &rx.try_recv() {
            Ok(pixels) => {
                redraws += 1;
                display.draw(pixels);
                                if redraws == 200 {
                                    save_screen_buffer(pixels, "test.json".to_string());
                                    std::process::exit(0);
                                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        }
        thread::sleep(time::Duration::from_millis(5));
    }
}
