use std::time::{Duration, Instant};

use tfhe::shortint::parameters::v1_5::{
    V1_5_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
    V1_5_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
    V1_5_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
    V1_5_PARAM_MESSAGE_4_CARRY_4_KS_PBS_GAUSSIAN_2M64,
};
use tfhe::shortint::parameters::ClassicPBSParameters;
use tfhe::shortint::{gen_keys, Ciphertext};

// Running multiple bitwise ANDs reduces jitter on the timing measurements.
const BENCH_ITERATIONS: u32 = 16;

#[derive(Clone, Copy)]
struct Config {
    label: &'static str,
    message_bits: u32,
    carry_bits: u32,
    params: ClassicPBSParameters,
}

struct BenchmarkRecord {
    label: &'static str,
    message_bits: u32,
    carry_bits: u32,
    plaintext_a: u64,
    plaintext_b: u64,
    avg_duration: Duration,
    result: u64,
    server_message_modulus: u64,
    server_carry_modulus: u64,
}

fn main() {
    println!("Starting Shortint AND benchmark...\n Please be patient, this may take a few minutes.\n");
    let configs = [
        Config {
            label: "1_1",
            message_bits: 1,
            carry_bits: 1,
            params: V1_5_PARAM_MESSAGE_1_CARRY_1_KS_PBS_GAUSSIAN_2M64,
        },
        Config {
            label: "2_2",
            message_bits: 2,
            carry_bits: 2,
            params: V1_5_PARAM_MESSAGE_2_CARRY_2_KS_PBS_GAUSSIAN_2M64,
        },
        Config {
            label: "3_3",
            message_bits: 3,
            carry_bits: 3,
            params: V1_5_PARAM_MESSAGE_3_CARRY_3_KS_PBS_GAUSSIAN_2M64,
        },
        Config {
            label: "4_4",
            message_bits: 4,
            carry_bits: 4,
            params: V1_5_PARAM_MESSAGE_4_CARRY_4_KS_PBS_GAUSSIAN_2M64,
        },
    ];

    let records: Vec<BenchmarkRecord> = configs.iter().copied().map(benchmark_config).collect();

    print_summary(&records);
    print_details(&records);
}

fn benchmark_config(config: Config) -> BenchmarkRecord {
    let (client_key, server_key) = gen_keys(config.params);

    // Choose plaintexts that span the entire message modulus without overflowing.
    let plaintext_a = (1u64 << config.message_bits) - 1;
    let plaintext_b = if config.message_bits == 0 {
        0
    } else {
        1u64 << (config.message_bits - 1)
    };

    let base_ct_a = client_key.encrypt(plaintext_a);
    let base_ct_b = client_key.encrypt(plaintext_b);

    let mut last_ciphertext: Option<Ciphertext> = None;
    let start = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        // Clone the inputs so each AND works on a fresh ciphertext pair.
        let mut tmp_a = base_ct_a.clone();
        let mut tmp_b = base_ct_b.clone();
        last_ciphertext = Some(server_key.unchecked_bitand(&mut tmp_a, &mut tmp_b));
    }
    let avg_duration = start.elapsed()  / BENCH_ITERATIONS;

    let final_ciphertext = last_ciphertext.expect("bench loop executed at least once");
    let result = client_key.decrypt(&final_ciphertext);

    BenchmarkRecord {
        label: config.label,
        message_bits: config.message_bits,
        carry_bits: config.carry_bits,
        plaintext_a,
        plaintext_b,
        avg_duration,
        result,
        server_message_modulus: server_key.message_modulus.0,
        server_carry_modulus: server_key.carry_modulus.0,
    }
}

fn print_summary(records: &[BenchmarkRecord]) {
    const DIVIDER: &str = "+--------+--------------+-------------+----------------+--------+";
    println!("\nShortint AND benchmark (message == carry)");
    println!("{:}", DIVIDER);
    println!(
        "| {:^6} | {:^12} | {:^11} | {:^14} | {:^6} |",
        "Config", "Message bits", "Carry bits", "Avg AND", "Result"
    );
    println!("{:}", DIVIDER);
    for record in records {
        println!(
            "| {:^6} | {:^12} | {:^11} | {:>14} | {:^6} |",
            record.label,
            record.message_bits,
            record.carry_bits,
            format_duration(record.avg_duration),
            record.result,
        );
    }
    println!("{:}", DIVIDER);
}

fn print_details(records: &[BenchmarkRecord]) {
    for record in records {
        println!(
            "\nConfig {} (message={}, carry={})",
            record.label, record.message_bits, record.carry_bits
        );
        println!(
            "  Transmitted plaintexts: 0b{:b} & 0b{:b}",
            record.plaintext_a, record.plaintext_b
        );
        println!(
            "  Server moduli (message/carry): {}/{}",
            record.server_message_modulus, record.server_carry_modulus
        );

    }
}

fn format_duration(duration: Duration) -> String {
    format!("{:.2} ms", duration.as_secs_f64() * 1_000.0)
}
