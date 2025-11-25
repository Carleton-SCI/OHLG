#![allow(non_snake_case)]
use ohlg::oblivious_gates::*;
use ohlg::tgsw::*;
use std::time::Duration;
use std::time::Instant;
use tfhe::boolean::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    For TFHE-io parameters, the Decomposition base B = 16 and level l=3 are used. Obfuscated gate error probability is 10^-18
    For TFHE-rs default Boolean parameters, the Decomposition base B = 64 and level l=2 are used. Obfuscated gate error probability is 10^-25
    */
    let param_choice = "TFHE_RS";
    let tgsw_params: TgswParams;

    match param_choice {
        "TFHE_IO" => {
            //tgsw_decomposition_base = 16;
            //tgsw_decomposition_level = 3;
            //selected_parameters = TFHE_LIB_PARAMETERS;
            tgsw_params = TgswParams::new(TFHE_LIB_PARAMETERS, 16, 3);
        }
        "TFHE_RS" => {
            //tgsw_decomposition_base = 64;
            //tgsw_decomposition_level = 2;
            //selected_parameters = DEFAULT_PARAMETERS;
            tgsw_params = TgswParams::new(DEFAULT_PARAMETERS, 64, 2);
        }
        _ => {
            //tgsw_decomposition_base = 64;
            //tgsw_decomposition_level = 2;
            //selected_parameters = DEFAULT_PARAMETERS;
            tgsw_params = TgswParams::new(DEFAULT_PARAMETERS, 64, 2);
        }
    }

    //Create the client and server keys
    let client_key = ClientKey::new(&tgsw_params.tfhe_params);
    let server_key = ServerKey::new(&client_key);

    //----Perform an Obfuscated logic operation (NAND) between two ciphertexts----
    //m1 and m2 are the Boolean plaintext values to be encrypted
    //M is the plaintext value of the multiplicative parameter (=1 for NAND operation)
    //d is the plaintext value of the additive parameter (=3/8 for NAND operation)

    let m1: bool = false;
    let m2: bool = false;
    let M: u32 = 1;
    let d: u32 = (1 << (32 - 3)) * 3;
    let opr_str = "NAND";

    //Check the operation correctness (The result of the obfuscated NAND gate should be true if both m1 and m2 are false)
    //The counters for the true and false results (the result of the obfuscated gate, depends on the input values)
    //All the results should be true or all should be false
    let mut true_counter = 0;
    let mut false_counter = 0;
    let trials = 100;

    //Time accumulator
    let mut time_accumulator: Duration = Duration::new(0, 0);

    //Multiple trials for time averaging
    for _i in 0..trials {
        let ct_1 = client_key.encrypt(m1);
        let ct_2 = client_key.encrypt(m2);
        let ct_d = client_key.encrypt_abs(d);
        let tgsw_ct = TgswCiphertext::new_encrypt(M, &tgsw_params, &client_key);

        //Capture start time
        let start = Instant::now();

        //Execute the obfuscated gate
        let ct_res = ob_gate_2op(&ct_1, &ct_2, &tgsw_ct, &ct_d, &tgsw_params, &server_key);

        //Capture end time
        let duration = start.elapsed();
        time_accumulator += duration;

        //Decrypt the result and check the correctness.
        let dec_val = client_key.decrypt(&ct_res);
        if dec_val == false {
            false_counter += 1;
        } else {
            true_counter += 1;
        }
    }

    println!(
        "(Obfuscated) {} op.: between {} and {}. True counter = {}, false counter = {}",
        opr_str, m1, m2, true_counter, false_counter
    );
    println!(
        "Average time of an obfuscated gate: {:?}",
        time_accumulator / trials
    );

    //Benchmarking the execution time of a typical TFHE gate
    let mut time_accumulator_tfhe: Duration = Duration::new(0, 0);
    for _i in 0..trials {
        let ct_1 = client_key.encrypt(m1);
        let ct_2 = client_key.encrypt(m2);

        let start = Instant::now();
        let _ct_nand = server_key.nand(&ct_1, &ct_2);
        let duration = start.elapsed();

        time_accumulator_tfhe += duration;
    }
    println!(
        "Average time of a typical NAND TFHE gate: {:?}",
        time_accumulator_tfhe / trials
    );

    Ok(())
}
