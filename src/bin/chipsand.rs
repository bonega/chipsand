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
    //    let data = fs::read("roms/DRMARIO.GB").unwrap();
//                    let data = fs::read("../gb-test-roms/cpu_instrs/cpu_instrs.gb").unwrap();
                        let data = fs::read("roms/gb-test-roms/cpu_instrs/individual/01-special.gb").unwrap();
    //    let data = fs::read("../gb-test-roms/cpu_instrs/individual/02-interrupts.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/04-op r,imm.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/05-op rp.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/06-ld r,r.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/08-misc instrs.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/09-op r,r.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/10-bit ops.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb").unwrap();
    //            let data = fs::read("../gb-test-roms/interrupt_time/interrupt_time.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/instr_timing/instr_timing.gb").unwrap();
    //                let data = fs::read("../gb-test-roms/mem_timing/individual/01-read_timing.gb").unwrap();
    //                let data = fs::read("../gb-test-roms/mem_timing/individual/02-write_timing.gb").unwrap();
    //            let data = fs::read("../gb-test-roms/mem_timing/individual/03-modify_timing.gb").unwrap();
    //        let data = fs::read("../gb-test-roms/mem_timing-2/mem_timing.gb").unwrap();
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
