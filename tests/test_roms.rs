use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc, Mutex};
use std::{fs, thread, time};

use serde_json;

use chipsandlib::cpu::CPU;
use chipsandlib::mmu::MMU;
use chipsandlib::screen_buffer_to_vec;

fn emulation_loop(mut cpu: CPU, abort: Arc<Mutex<bool>>) {
    loop {
        cpu.cycle();
        let x = *abort.lock().unwrap();
        if x {
            break;
        }
    }
}

fn test_to_buffer(rom_path: String, n_redraws: u8) -> Result<bool, TryRecvError> {
    let test_path = rom_path.replacen("roms", "tests", 1).replace(".gb", ".json");
    let data = fs::read(rom_path).unwrap();
    let (tx, rx) = mpsc::sync_channel(0);
    let (_tx_events, rx_events) = mpsc::channel();
    let abort = Arc::new(Mutex::new(false));
    let cpu_abort = abort.clone();

    thread::spawn(move || {
        let mmu = MMU::new(data, tx, rx_events);
        let mut cpu = CPU::new(mmu);
        cpu.reset();
        emulation_loop(cpu, cpu_abort);
    });

    let mut redraws = 0;
    loop {
        thread::sleep(time::Duration::from_millis(1));
        match &rx.try_recv() {
            Ok(pixels) => {
                redraws += 1;
                if redraws == n_redraws {
                    let mut abort = abort.lock().unwrap();
                    *abort = true;
                    let pixs = screen_buffer_to_vec(pixels);

                    let buffer = fs::File::open(dbg!(test_path));
                    let x: Vec<u8> = serde_json::from_reader(buffer.unwrap()).unwrap();
                    break Ok(pixs.eq(&x));
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Channel disconnect"),
        };
    }
}

#[test]
fn gb_test_roms_cpu_instrs_individual_01_special() {
    let res = test_to_buffer("roms/gb-test-roms/cpu_instrs/individual/01-special.gb".to_string(), 200);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim00() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim00.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim00_div_trigger() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim00_div_trigger.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim01() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim01.gb".to_string(),10);
    assert!(res.unwrap());
}
#[test]
fn mooneye_acceptance_timer_tim01_div_trigger() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim01_div_trigger.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim10() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim10.gb".to_string(),10);
    assert!(res.unwrap());
}
#[test]
fn mooneye_acceptance_timer_tim10_div_trigger() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim10_div_trigger.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim11() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim11.gb".to_string(),10);
    assert!(res.unwrap());
}
#[test]
fn mooneye_acceptance_timer_tim11_div_trigger() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim11_div_trigger.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tima_reload() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tima_reload.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_div_write() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/div_write.gb".to_string(),50);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_intr_timing() {
    let res = test_to_buffer("roms/mooneye/acceptance/intr_timing.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_if_ie_registers() {
    let res = test_to_buffer("roms/mooneye/acceptance/if_ie_registers.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_ppu_intr_1_2_timing_GS() {
    let res = test_to_buffer("roms/mooneye/acceptance/ppu/intr_1_2_timing-GS.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_ppu_intr_2_0_timing() {
    let res = test_to_buffer("roms/mooneye/acceptance/ppu/intr_2_0_timing.gb".to_string(),10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_bits_reg_f() {
    let res = test_to_buffer("roms/mooneye/acceptance/bits/reg_f.gb".to_string(),10);
    assert!(res.unwrap());
}
#[test]
fn test_mooneye_acceptance_oam_dma_basic() {
    let res = test_to_buffer("roms/mooneye/acceptance/oam_dma/basic.gb".to_string(),10);
    assert!(res.unwrap());
}
#[test]
fn test_mooneye_acceptance_oam_dma_reg_read() {
    let res = test_to_buffer("roms/mooneye/acceptance/oam_dma/reg_read.gb".to_string(),10);
    assert!(res.unwrap());
}

//#[test]
//Todo: fn mooneye_acceptance_ppu_intr_2_mode0_timing() {
//    let res = test_to_buffer("roms/mooneye/acceptance/ppu/intr_2_mode0_timing.gb".to_string());
//    assert!(res.unwrap());
//}

//#[test]
//Todo: fn mooneye_acceptance_ppu_vblank_stat_intr_C() {
//    let res = test_to_buffer("roms/mooneye/acceptance/ppu/vblank_stat_intr-C.gb".to_string());
//    assert!(res.unwrap());
//}

//#[test]
//Todo: fn mooneye_acceptance_timer_tima_write_reloading() {
//    let res = test_to_buffer("roms/mooneye/acceptance/timer/tima_write_reloading.gb".to_string());
//    assert!(res.unwrap());
//}

//#[test]
//Todo: fn mooneye_acceptance_timer_rapid_toggle() {
//    let res = test_to_buffer("roms/mooneye/acceptance/timer/rapid_toggle.gb".to_string());
//    assert!(res.unwrap());
//}

//#[test]
//Todo: fn test_mooneye_acceptance_oam_dma_sources_GS() {
//    let res = test_to_buffer("roms/mooneye/acceptance/oam_dma/sources-GS.gb".to_string());
//    assert!(res.unwrap());
//}
