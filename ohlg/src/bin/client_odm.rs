#![allow(non_snake_case)]
use std::io::Write;
use std::time::Instant;
use tfhe::boolean::prelude::*;
use ohlg::tgsw::*;


fn main()
{
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
            
        },
        "TFHE_RS" => {
            //tgsw_decomposition_base = 64;
            //tgsw_decomposition_level = 2;
            //selected_parameters = DEFAULT_PARAMETERS;
            tgsw_params = TgswParams::new(DEFAULT_PARAMETERS, 64, 2);
        },
        _ => {
            //tgsw_decomposition_base = 64;
            //tgsw_decomposition_level = 2;
            //selected_parameters = DEFAULT_PARAMETERS;
            tgsw_params = TgswParams::new(DEFAULT_PARAMETERS, 64, 2);
        }
    }

    let chars_num = 10;
    let bits_per_char = 8;

    let num_xnor_gates:u32 = chars_num * bits_per_char; //XNORing stage, 1 XNOR per bit
    let num_and_gates:u32 = (bits_per_char - 1) * chars_num; //ANDing stage, aggregate all bits/char using AND gates
    let num_or_gates:u32 = chars_num - 1; //ORing stage, aggregate all chars results using OR gates

    let num_total_gates:u32 = num_xnor_gates + num_and_gates + num_or_gates;

    print!("number of search characters: {}\n\r", chars_num);
    print!("number of bits per character: {}\n\r", bits_per_char);
    print!("number of XNOR gates = chars_num * bits_per_char : {}\n\r", num_xnor_gates);
    print!("number of AND gates = (bits_per_char - 1) * chars_num : {}\n\r", num_and_gates);
    print!("number of OR gates = chars_num - 1 : {}\n\r", num_or_gates);
    print!("number of total gates = num_xnor_gates + num_and_gates + num_or_gates : {}\n\r", num_total_gates);




    let client_key = ClientKey::new(&tgsw_params.tfhe_params);
    let server_key = ServerKey::new(&client_key);

    //-----------------Gate Parameters Encryption-----------------
    //Create an empty vector to store num_total_gates number of TGSW ciphertexts
    let mut C_gate_params:Vec<TgswCiphertext> = Vec::with_capacity(num_total_gates as usize);

    //Create an empty vector to store num_total_gates number of Ciphertexts
    let mut cd_gate_params:Vec<Ciphertext> = Vec::with_capacity(num_total_gates as usize);

    //Start measuring time
    let start = Instant::now();
    for c in 0..chars_num 
    {
        for b in 0..bits_per_char 
        {
            let A:u32 = 2; //Multiplicative parameter for XNOR = 2
            let d:u32 = (1 << (32-3))*6; //Additive parameter for XNOR = 6/8 = -2/8

            let C_gate = TgswCiphertext::new_encrypt(A, &tgsw_params, &client_key);
            let cd_gate = client_key.encrypt_abs(d);

            C_gate_params.push(C_gate);
            cd_gate_params.push(cd_gate);

            if b != 0 
            {
                let A:u32 = 1; //Multiplicative parameter for AND = 1
                let d:u32 = (1 << (32-3))*7; //Additive parameter for AND = 1

                let C_gate = TgswCiphertext::new_encrypt(A, &tgsw_params, &client_key);
                let cd_gate = client_key.encrypt_abs(d);

                C_gate_params.push(C_gate);
                cd_gate_params.push(cd_gate);
            }

        }
        if c != 0 
        {
            let A:u32 = 1; //Multiplicative parameter for OR = 1
            let d:u32 = (1 << (32-3))*1; //Additive parameter for OR = 1

            let C_gate = TgswCiphertext::new_encrypt(A, &tgsw_params, &client_key);
            let cd_gate = client_key.encrypt_abs(d);

            C_gate_params.push(C_gate);
            cd_gate_params.push(cd_gate);
        }

    }
    //End measuring gates parameters encryption time
    let duration = start.elapsed();
    print!("Gates Parameters Encryption Time: {:?}\n\r", duration);
    //--------------------End of Gate Parameters Encryption--------------------

    //-----------------Search character encryption-----------------
    //--------Get the search character from the user
    println!("Enter a search character: ");
    let mut search = String::new();
    std::io::stdin().read_line(&mut search).unwrap();

    // Get the first byte as u8
    let plain_search_byte = search.bytes().next().unwrap_or(0);
    println!("So we will be searching for: {}", plain_search_byte as char);

    //Measre search character encryption time
    let mut search_ciphertexts = encrypt_u8(&client_key, plain_search_byte);
    
    //Do the encryption 1000 times and measure the time (averaging)
    let start = Instant::now();
    for _ in 0..1000
    {
        search_ciphertexts = encrypt_u8(&client_key, plain_search_byte);
    }
    let duration = start.elapsed();
    print!("Search Character Encryption Time: {:?}\n\r", duration/1000);
    //------------------End of search character encryption------------------------------


    //--------Create a search corpus and encrypt it--------------------------
    //The search corpus is a string of characters, bitwise encrypted using the client key. It is expected to exist on the server side, but it is done here for the experiment.
    let plain_search_corpus = "ABCDEFGHIJ";
    let search_corpus_ciphertexts = encrypt_string(&client_key, plain_search_corpus);
    //-------------------------------------------------------------------------

    //--------Export server key to a file
    let mut file = std::fs::File::create("server_key_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &server_key).unwrap();
    file.write(&serialized_data).unwrap();

    //--------Export client key to a file
    let mut file = std::fs::File::create("client_key_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &client_key).unwrap();
    file.write(&serialized_data).unwrap();

    //--------Export search ciphertexts to a file
    let mut file = std::fs::File::create("search_ciphertexts_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &search_ciphertexts).unwrap();
    file.write(&serialized_data).unwrap();

    //--------Export search corpus ciphertexts to a file
    let mut file = std::fs::File::create("search_corpus_ciphertexts_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &search_corpus_ciphertexts).unwrap();
    file.write(&serialized_data).unwrap();

    //--------Export C_gate_params to a file
    let mut file = std::fs::File::create("C_gate_params_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &C_gate_params).unwrap();
    file.write(&serialized_data).unwrap();

    //--------Export cd_gate_params to a file
    let mut file = std::fs::File::create("cd_gate_params_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &cd_gate_params).unwrap();
    file.write(&serialized_data).unwrap();

    //--------Export tgsw_params to a file
    let mut file = std::fs::File::create("tgsw_params_ODM.txt").unwrap();
    let mut serialized_data = Vec::new();
    bincode::serialize_into(&mut serialized_data, &tgsw_params).unwrap();
    file.write(&serialized_data).unwrap();


}

//Encrypt a u8 value into 8 ciphertexts
fn encrypt_u8(client_key: &ClientKey, value: u8) -> Vec<Ciphertext> {
    let mut ciphertexts = Vec::with_capacity(8);
    for i in 0..8 {
        ciphertexts.push(client_key.encrypt((value >> i) & 1 == 1));
    }
    ciphertexts
}

//Encrypt a string into a vector of vectors of ciphertext bits
fn encrypt_string(client_key: &ClientKey, value: &str) -> Vec<Vec<Ciphertext>> {
    let mut ciphertexts = Vec::with_capacity(value.len());
    for c in value.chars() {
        ciphertexts.push(encrypt_u8(client_key, c as u8));
    }
    ciphertexts
}