// Verifier program that reads the client key and the search result from the files named "client_key.txt" and "encrypted_result.txt" respectively.
use std::io::Cursor;
use std::time::Instant;
use tfhe::boolean::prelude::*;
fn main() {
    //read the clinet key from the file named "client_key_ODM.txt"
    let serialized_data =
        std::fs::read("client_key_ODM.txt").expect("Failed to open client_key_ODM.txt");
    let mut cursor = Cursor::new(&serialized_data);
    let client_key: ClientKey = bincode::deserialize_from(&mut cursor).unwrap();

    //read the search result from the file named "final_result_ODM.txt"
    let serialized_data =
        std::fs::read("final_result_ODM.txt").expect("Failed to open final_result_ODM.txt");
    let mut cursor = Cursor::new(&serialized_data);
    let encrypted_result: Ciphertext = bincode::deserialize_from(&mut cursor).unwrap();

    //Decrypt the search result
    let mut decrypted_result = client_key.decrypt(&encrypted_result);

    //Do the decryption 1000 times (averaging) and measure the time
    let start = Instant::now();
    for _ in 0..1000 {
        decrypted_result = client_key.decrypt(&encrypted_result);
    }
    let duration = start.elapsed();
    println!(
        "Time taken to decrypt the search result: {:?}",
        duration / 1000
    );

    //Print the decrypted ODM result
    println!("Decrypted result: {}", decrypted_result);
}
