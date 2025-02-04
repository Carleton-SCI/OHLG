
use tfhe::boolean::engine::BooleanEngine;
use tfhe::boolean::prelude::*;
use tfhe::core_crypto::entities::lwe_ciphertext::LweCiphertext;
use tfhe::core_crypto::algorithms::lwe_linear_algebra::*;
use std::cell::RefCell;

use crate::tgsw::{TgswCiphertext, TgswParams};

thread_local! {
    static OB_BOOLEAN_ENGINE: RefCell<BooleanEngine> = RefCell::new(BooleanEngine::new());
}

/*
An obfuscated logic function that performs A(c1+c2)+d, where
A:TGSW encryption of a scalar value in Z/PZ where P = B^L
c1,c2: The TLWE ciphertexts to be operated on
d: TLWE ciphertext that defines the logic function along with A
*/
pub fn ob_gate_2op(ct_1: &Ciphertext,
    ct_2: &Ciphertext,
    A:&TgswCiphertext,
    ct_d: &Ciphertext,
    tgsw_params: &TgswParams,
    server_key: &ServerKey
    ) -> Ciphertext 
{

    let ct_1_lwe = match ct_1 {
        Ciphertext::Encrypted(ct_lwe) => ct_lwe,
        _ => panic!("Expected encrypted ciphertext"),
    };

    let ct_2_lwe = match ct_2 {
        Ciphertext::Encrypted(ct_lwe) => ct_lwe,
        _ => panic!("Expected encrypted ciphertext"),
    };

    let ct_d_lwe = match ct_d {
        Ciphertext::Encrypted(ct_lwe) => ct_lwe,
        _ => panic!("Expected encrypted ciphertext"),
    };


    let mut buffer_lwe = LweCiphertext::new(
        0u32,
        ct_1_lwe.lwe_size(),
        ct_1_lwe.ciphertext_modulus(),
    );

    lwe_ciphertext_add(&mut buffer_lwe, &ct_1_lwe, &ct_2_lwe);
    
    let mut buffer_ct = Ciphertext::Encrypted(buffer_lwe);
    
    buffer_ct = A.ext_product(&buffer_ct, &tgsw_params /*server_key*/);
    
    buffer_lwe = match buffer_ct {
        Ciphertext::Encrypted(ct_lwe) => ct_lwe,
        _ => panic!("Expected encrypted ciphertext"),
    };
    
    lwe_ciphertext_add_assign(&mut buffer_lwe, &ct_d_lwe);
    


    OB_BOOLEAN_ENGINE.with(|engine_cell| {
        let eng = & mut engine_cell.borrow_mut() as &mut BooleanEngine;
        let bootstrapper = &mut eng.bootstrapper;
        bootstrapper.apply_bootstrapping_pattern(buffer_lwe, &server_key)
       
   })

}

/*A 1-operan obfuscated logic gate (Buffer or NOT)
Performs the operation ct_1 + ct_arg_1, where ct_1 is the input and ct_arg_1 is either 0(buffer) or 1/2(NOT)
*/
pub fn ob_gate_1op(ct_1: &Ciphertext, ct_arg_1: &Ciphertext, server_key: &ServerKey) -> Ciphertext {
    
    let ct_1_lwe = match ct_1 {
        Ciphertext::Encrypted(ct_lwe) => ct_lwe,
        _ => panic!("Expected encrypted ciphertext"),
    };

    let ct_arg_1_lwe = match ct_arg_1 {
        Ciphertext::Encrypted(ct_lwe) => ct_lwe,
        _ => panic!("Expected encrypted ciphertext"),
    };

    let mut buffer_lwe = LweCiphertext::new(
        0u32,
        ct_1_lwe.lwe_size(),
        ct_1_lwe.ciphertext_modulus(),
    );

    lwe_ciphertext_add(&mut buffer_lwe, &ct_1_lwe, &ct_arg_1_lwe);

    OB_BOOLEAN_ENGINE.with(|engine_cell| {
        let eng = & mut engine_cell.borrow_mut() as &mut BooleanEngine;
        let bootstrapper = &mut eng.bootstrapper;
        bootstrapper.apply_bootstrapping_pattern(buffer_lwe, &server_key)
       
   })

}
