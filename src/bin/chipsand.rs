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

#[derive(StructOpt, Debug)]
#[structopt(name = "chipsand")]
struct Opt {
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

fn emulation_loop(mut cpu: cpu::CPU) {
    loop {
        cpu.cycle();
        if cpu.cycles % 4096 == 0 {
            thread::sleep(time::Duration::from_millis(1));
        }
    }
}

fn main() {
    //    let opt = Opt::from_args();
    //    println!("{:#?}", opt);
    ////            let data = fs::read("roms/mooneye/acceptance/timer/tima_write_reloading.gb").unwrap();
    //            let data = fs::read("roms/mooneye/acceptance/timer/rapid_toggle.gb").unwrap();
    //        let data = fs::read("roms/mooneye/acceptance/ppu/intr_2_mode0_timing.gb").unwrap();
    //        let data = fs::read("roms/mooneye/acceptance/oam_dma/sources-GS.gb").unwrap();
    //            let data = fs::read("roms/mooneye/misc/ppu/vblank_stat_intr-C.gb").unwrap();
//    let data = fs::read("roms/tetris.gb").unwrap();
        let data = fs::read("roms/DRMARIO.GB").unwrap();
//                let data = fs::read("roms/gb-test-roms/interrupt_time/interrupt_time.gb").unwrap();
    let sdl_context = sdl2::init().unwrap();
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
            let x = from_sdl2_event(event);
            if Some(Control::Quit) == x {
                std::process::exit(0);
            }
            if let Some(x) = x {
                tx_events.send(x);
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
