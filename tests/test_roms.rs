use std::sync::mpsc::{TryRecvError, RecvError};
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

fn test_to_buffer(rom_path: String, n_redraws: u16) -> Result<bool, TryRecvError> {
    let test_path = rom_path
        .replacen("roms", "tests", 1)
        .replace(".gb", ".json");
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

    for _ in 0..n_redraws - 1 {
        &rx.recv()?;
    };
    match &rx.recv() {
        Ok(pixels) => {
            let mut abort = abort.lock().unwrap();
            *abort = true;
            let pixels = screen_buffer_to_vec(pixels);

            let buffer = fs::File::open(dbg!(test_path));
            let x: Vec<u8> = serde_json::from_reader(buffer.unwrap()).unwrap();
            return Ok(pixels.eq(&x));
        }
        Err(_) => panic!("Failed to communicate over pixel-channel")
    }
}

#[test]
fn gb_test_roms_cpu_instrs_individual_01_special() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/01-special.gb".to_string(),
        200,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_02_interrupts() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/02-interrupts.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_03_op_sp_hl() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb".to_string(),
        200,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_04_op_r_imm() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/04-op r,imm.gb".to_string(),
        200,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_05_op_rp() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/05-op rp.gb".to_string(),
        250,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_06_ld_r_r() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/06-ld r,r.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_07_jr_jp_call_ret_rst() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_08_misc_instrs() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/08-misc instrs.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_09_op_r_r() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/09-op r,r.gb".to_string(),
        550,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_10_bit_ops() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/10-bit ops.gb".to_string(),
        850,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_cpu_instrs_individual_11_op_a_hl() {
    let res = test_to_buffer(
        "roms/gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb".to_string(),
        1050,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_instr_timing_instr_timing() {
    let res = test_to_buffer(
        "roms/gb-test-roms/instr_timing/instr_timing.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_mem_timing_individual_01_read_timing() {
    let res = test_to_buffer(
        "roms/gb-test-roms/mem_timing/individual/01-read_timing.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_mem_timing_individual_02_write_timing() {
    let res = test_to_buffer(
        "roms/gb-test-roms/mem_timing/individual/02-write_timing.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_mem_timing_individual_03_modify_timing() {
    let res = test_to_buffer(
        "roms/gb-test-roms/mem_timing/individual/03-modify_timing.gb".to_string(),
        50,
    );
    assert!(res.unwrap());
}

#[test]
fn gb_test_roms_mem_timing_2_mem_timing() {
    let res = test_to_buffer(
        "roms/gb-test-roms/mem_timing-2/mem_timing.gb".to_string(),
        250,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim00() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim00.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim00_div_trigger() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/timer/tim00_div_trigger.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim01() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim01.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim01_div_trigger() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/timer/tim01_div_trigger.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim10() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim10.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim10_div_trigger() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/timer/tim10_div_trigger.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim11() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/tim11.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tim11_div_trigger() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/timer/tim11_div_trigger.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_tima_reload() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/timer/tima_reload.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_timer_div_write() {
    let res = test_to_buffer("roms/mooneye/acceptance/timer/div_write.gb".to_string(), 50);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_intr_timing() {
    let res = test_to_buffer("roms/mooneye/acceptance/intr_timing.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_if_ie_registers() {
    let res = test_to_buffer("roms/mooneye/acceptance/if_ie_registers.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_ppu_intr_1_2_timing_GS() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/ppu/intr_1_2_timing-GS.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_ppu_intr_2_0_timing() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/ppu/intr_2_0_timing.gb".to_string(),
        10,
    );
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_bits_reg_f() {
    let res = test_to_buffer("roms/mooneye/acceptance/bits/reg_f.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_oam_dma_basic() {
    let res = test_to_buffer("roms/mooneye/acceptance/oam_dma/basic.gb".to_string(), 10);
    assert!(res.unwrap());
}

#[test]
fn mooneye_acceptance_oam_dma_reg_read() {
    let res = test_to_buffer(
        "roms/mooneye/acceptance/oam_dma/reg_read.gb".to_string(),
        10,
    );
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
//Todo: fn mooneye_acceptance_oam_dma_sources_GS() {
//    let res = test_to_buffer("roms/mooneye/acceptance/oam_dma/sources-GS.gb".to_string());
//    assert!(res.unwrap());
//}
