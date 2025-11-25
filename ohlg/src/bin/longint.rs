use std::time::{Duration, Instant};

use tfhe::integer::{gen_keys_radix, RadixCiphertext, ServerKey};
use tfhe::shortint::parameters::v1_5::{
    V1_5_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
    V1_5_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
    V1_5_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
    V1_5_PARAM_MESSAGE_4_CARRY_4_KS_PBS_GAUSSIAN_2M64,
};
use tfhe::shortint::parameters::ClassicPBSParameters;

const BENCH_ITERATIONS: u32 = 8;
const MAX_BLOCKS: usize = 4;
const PATTERN_A: u64 = 0xAA55_AA55_AA55_AA55;
const PATTERN_B: u64 = 0xCC33_CC33_CC33_CC33;

#[derive(Clone, Copy)]
struct ParamConfig {
    label: &'static str,
    message_bits: u32,
    params: ClassicPBSParameters,
}

#[derive(Clone, Copy)]
struct ThreadingMode {
    label: &'static str,
    uses_parallel: bool,
}

struct BenchmarkRecord {
    param_label: &'static str,
    message_bits: u32,
    blocks: usize,
    threading_label: &'static str,
    threaded: bool,
    plaintext_a: u64,
    plaintext_b: u64,
    avg_duration: Duration,
    decrypted: u64,
    ciphertext_blocks: usize,
    server_message_modulus: u64,
    server_carry_modulus: u64,
}

fn main() {
    println!("TFHE integer AND benchmark (radix representation)\n");
    println!("Please be patient, this may take a few minutes (up to 10 minutes).\n");

    let param_configs = [
        ParamConfig {
            label: "1_1",
            message_bits: 1,
            params: V1_5_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
        },
        ParamConfig {
            label: "2_2",
            message_bits: 2,
            params: V1_5_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
        },
        ParamConfig {
            label: "3_3",
            message_bits: 3,
            params: V1_5_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
        },
        ParamConfig {
            label: "4_4",
            message_bits: 4,
            params: V1_5_PARAM_MESSAGE_4_CARRY_4_KS_PBS_GAUSSIAN_2M64,
        },
    ];

    let threading_modes = [
        ThreadingMode {
            label: "sequential",
            uses_parallel: false,
        },
        ThreadingMode {
            label: "parallel",
            uses_parallel: true,
        },
    ];

    let mut records = Vec::new();

    for config in param_configs {
        print!("Benchmarking parameter set: {}...", config.label);
        for mode in threading_modes {
            for blocks in 1..=MAX_BLOCKS {
                let (client_key, server_key) = gen_keys_radix(config.params, blocks);
                let (plaintext_a, plaintext_b) = select_plaintexts(config.message_bits, blocks);
                let base_ct_a = client_key.encrypt(plaintext_a);
                let base_ct_b = client_key.encrypt(plaintext_b);
                
                let (avg_duration, result_ct) =
                    run_benchmark(&server_key, &base_ct_a, &base_ct_b, mode);
                let decrypted: u64 = client_key.decrypt(&result_ct);

                records.push(BenchmarkRecord {
                    param_label: config.label,
                    message_bits: config.message_bits * blocks as u32,
                    blocks,
                    threading_label: mode.label,
                    threaded: mode.uses_parallel,
                    plaintext_a,
                    plaintext_b,
                    avg_duration,
                    decrypted,
                    ciphertext_blocks: blocks,
                    server_message_modulus: server_key.message_modulus().0,
                    server_carry_modulus: server_key.carry_modulus().0,
                });
            }
        }
        println!("done.");
    }

    print_summary(&records);
    print_details(&records);
}

fn run_benchmark(
    server_key: &ServerKey,
    ct_a: &RadixCiphertext,
    ct_b: &RadixCiphertext,
    mode: ThreadingMode,
) -> (Duration, RadixCiphertext) {
    let mut last_ciphertext = None;
    let start = Instant::now();

    for _ in 0..BENCH_ITERATIONS {
        let result = if mode.uses_parallel {
            server_key.bitand_parallelized(ct_a, ct_b)
        } else {
            server_key.unchecked_bitand(ct_a, ct_b)
        };
        last_ciphertext = Some(result);
    }

    let avg_duration = start.elapsed() / BENCH_ITERATIONS;
    (avg_duration, last_ciphertext.expect("benchmark loop executed"))
}

fn select_plaintexts(message_bits: u32, num_blocks: usize) -> (u64, u64) {
    let total_bits = (message_bits * num_blocks as u32).min(64);
    if total_bits == 0 {
        return (0, 0);
    }

    let mask = if total_bits == 64 {
        u64::MAX
    } else {
        (1u64 << total_bits) - 1
    };

    let fallback = if mask == 0 { 0 } else { 1.min(mask) };
    let a = PATTERN_A & mask;
    let b = PATTERN_B & mask;
    let plaintext_a = if a == 0 { fallback } else { a };
    let plaintext_b = if b == 0 { mask } else { b };

    (plaintext_a, plaintext_b)
}

fn print_summary(records: &[BenchmarkRecord]) {
    const DIVIDER: &str =
        "+-------------------+--------+-----------+----------------+-------------+-------------+-------------+-------------+";
    println!("\nSummary ({} samples per cell)", BENCH_ITERATIONS);
    println!("{:}", DIVIDER);
    println!(
        "| {:^17} | {:^6} | {:^9} | {:^14} | {:^11} | {:^11} | {:^11} | {:^11}",
        "Param (Msg_Carry)", "Blocks", "Threaded", "Total Msg bits", "Avg AND", "Plain A", "Plain B", "Decrypted"
    );
    println!("{:}", DIVIDER);

    for record in records {
        println!(
            "| {:^17} | {:^6} | {:^9} | {:^14} | {:>11} | {:>11} | {:>11} | {:>11} |",
            record.param_label,
            record.blocks,
            if record.threaded { "yes" } else { "no" },
            record.message_bits,
            format_duration(record.avg_duration),
            format_value(record.plaintext_a),
            format_value(record.plaintext_b),
            format_value(record.decrypted),
        );
            if record.blocks == MAX_BLOCKS {
                println!("{:}", DIVIDER);
                if record.threaded{
                    println!("{:}", DIVIDER);
                }
            }
    }
}

fn print_details(records: &[BenchmarkRecord]) {
    let mut current_param = "";
    for record in records {
        if record.param_label != current_param {
            current_param = record.param_label;
            println!(
                "\nParameter {} (message bits per block = {}, carry bits per block = {})",
                record.param_label, record.message_bits, record.message_bits
            );
            println!(
                "  server moduli (message/carry): {}/{}",
                record.server_message_modulus, record.server_carry_modulus
            );
        }

        println!(
            "  - {:>2} block(s), {:<10}: avg {}, result {} (ciphertext blocks = {})",
            record.blocks,
            record.threading_label,
            format_duration(record.avg_duration),
            format_value(record.decrypted),
            record.ciphertext_blocks,
        );
    }
}

fn format_duration(duration: Duration) -> String {
    if duration.as_micros() < 1_000 {
        format!("{:.2}us", duration.as_secs_f64() * 1_000_000.0)
    } else if duration.as_millis() < 1_000 {
        format!("{:.2}ms", duration.as_secs_f64() * 1_000.0)
    } else {
        format!("{:.2}s", duration.as_secs_f64())
    }
}

fn format_value(value: u64) -> String {
    if value <= 0xFFFF {
        format!("0x{:04X}", value)
    } else {
        format!("0x{:08X}", value)
    }
}
