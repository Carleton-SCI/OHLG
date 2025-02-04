#![allow(non_snake_case)]
use std::io::Write;
use std::time::Instant;
use ohlg::oblivious_gates::ob_gate_2op;
use tfhe::boolean::prelude::*;
use ohlg::tgsw::*;

fn main()
{
    //Read the search ciphertexts (the encrypted search character) from file
    let mut file = std::fs::File::open("search_ciphertexts_ODM.txt").unwrap();
    let search_ciphertexts: Vec<Ciphertext> = bincode::deserialize_from(&mut file).unwrap();
    print!("Successfully read the search ciphertexts from the file\n\r");

    //Read the search corpus ciphertexts from the file
    let mut file = std::fs::File::open("search_corpus_ciphertexts_ODM.txt").unwrap();
    let search_corpus_ciphertexts: Vec<Vec<Ciphertext>> = bincode::deserialize_from(&mut file).unwrap();
    print!("Successfully read the search corpus ciphertexts from the file\n\r");

    //Read the server key from the file
    let mut file = std::fs::File::open("server_key_ODM.txt").unwrap();
    let server_key: ServerKey = bincode::deserialize_from(&mut file).unwrap();
    print!("Successfully read the server key from the file\n\r");

    //Read the C_gate_params (vector of encrypted gate multiplicative parameters) from the file
    //This step takes some time.
    let mut file = std::fs::File::open("C_gate_params_ODM.txt").unwrap(); //let mut C_gate_params: Vec<TgswCiphertext> = Vec::with_capacity(159); //This didn't make a difference in time.
    let C_gate_params:Vec<TgswCiphertext> = bincode::deserialize_from(&mut file).unwrap();
    print!("Successfully read the C_gate_params from the file\n\r");

    //Read the cd_gate_params (vector of encrypted gate additive parameters) from the file
    let mut file = std::fs::File::open("cd_gate_params_ODM.txt").unwrap();
    let cd_gate_params: Vec<Ciphertext> = bincode::deserialize_from(&mut file).unwrap();
    print!("Successfully read the cd_gate_params from the file\n\r");

    //Read the tgsw_params (Decomposition base, decomposition level, and the used TFHE parameter set) from the file
    let mut file = std::fs::File::open("tgsw_params_ODM.txt").unwrap();
    let tgsw_params: TgswParams = bincode::deserialize_from(&mut file).unwrap();
    print!("Successfully read the tgsw_params from the file\n\r");

    //Successfull reading of the files
    println!("Successfully read all files\n\r");

    //Get the number of characters in the search corpus
    let N = search_corpus_ciphertexts.len();

    //Get the number of bits in the search ciphertexts
    let B = search_ciphertexts.len();

    //Print the parameters for verification
    print!("search corpus ciphertexts size: {}\n\r", N);
    print!("number of bits in search ciphertexts: {}\n\r", B);
    print!("number of tgsw gate paramaters: {}\n\r", C_gate_params.len());
    print!("number of tlwe gate paramaters: {}\n\r", cd_gate_params.len());

    //---------Oblivious Direct Matching operation------------
    let mut params_counter:usize = 0; //Counter for the gate parameters (159 gates = 159 multiplicative parameters + 159 additive parameters in the given example)
    let mut final_result:Ciphertext = Ciphertext::Trivial(false); //The final result of the ODM operation, to be sent to the client for decryption.
    
    //Start measuring the searching time
    let start = Instant::now();
    for c in 0..N
    {
        let mut char_result:Ciphertext = Ciphertext::Trivial(false);
        for b in 0..B
        {
            let bit_temp = ob_gate_2op(&search_ciphertexts[b], &search_corpus_ciphertexts[c][b],  &C_gate_params[params_counter], &cd_gate_params[params_counter], &tgsw_params, &server_key);
            params_counter += 1;
            if b == 0 //For the first bit, just assign the result to the char_result
            {
                char_result = bit_temp;
            }
            else //For the rest of the bits, do the Obfuscated operation (AND in this example of ODM) to aggregate the bits
            {
                char_result = ob_gate_2op(&char_result, &bit_temp, &C_gate_params[params_counter], &cd_gate_params[params_counter], &tgsw_params, &server_key);
                params_counter += 1;
            }
        }
        
        if c == 0 //For the first character result, just assign the result to the final_result
        {
            final_result = char_result;
        }
        else //For the rest of the character results, aggregate using the Obfuscated operation (OR in the example of ODM)
        {
            final_result = ob_gate_2op(&final_result, &char_result, &C_gate_params[params_counter], &cd_gate_params[params_counter], &tgsw_params, &server_key);
            params_counter += 1;
        }
    }
    //measure time end
    let duration = start.elapsed();
    print!("ODM operation time: {:?}\n\r", duration);
    
    //print the parameters counter, check if it is equal to the number of gate parameters
    print!("params_counter: {}\n\r", params_counter);

    //Write the final result to the file
    let mut file = std::fs::File::create("final_result_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &final_result).unwrap();
    file.write(&serialized_data).unwrap();
    print!("Successfully wrote the final result to the file\n\r");



}