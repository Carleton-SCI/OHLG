#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use tfhe::boolean::engine::BooleanEngine;
use tfhe::{
    boolean::prelude::{BooleanParameters, Ciphertext, ClientKey},
    core_crypto::prelude::{CiphertextModulus, LweCiphertext},
};

thread_local! {
    static OB_BOOLEAN_ENGINE: RefCell<BooleanEngine> = RefCell::new(BooleanEngine::new());
}

pub struct GadgetMatrix {
    pub matrix: Vec<Vec<u32>>,
}

impl GadgetMatrix {
    // Method to create the gadget matrix based on the parameters n, B, l, and q
    pub fn new(n: usize, B: u32, l: usize, q: u64) -> Self {
        // Define the dimensions of the matrix: (n+1)*l rows and (n+1) columns
        let rows = (n + 1) * l;
        let cols = n + 1;

        // Create a matrix initialized with zeros
        let mut matrix: Vec<Vec<u32>> = vec![vec![0; cols]; rows];

        // Fill in the matrix GT as a decomposition matrix based on B and l
        for i in 0..rows {
            for j in 0..cols {
                if i >= j * l && i < (j + 1) * l {
                    // Set the element based on the given formula
                    let exponent = (i - j * l + 1) as u32;
                    //let value = ((q as f64 / B.pow(exponent) as f64) as u32);  //This line was working 16-jan-2025
                    let value = (q as u64 / B.pow(exponent) as u64) as u32; //This line is working 16-jan-2025
                    matrix[i][j] = value;
                }
            }
        }

        GadgetMatrix { matrix }
    }

    // Method to display the gadget matrix (optional for debugging)
    pub fn display(&self) {
        for row in &self.matrix {
            println!("{:?}", row);
        }
    }
}

// Use a static initialization to make the gadget matrix globally available
static mut GADGET_MATRIX: Option<GadgetMatrix> = None;
use std::sync::Once;
static INIT: Once = Once::new();

// Function to initialize the gadget matrix globally
pub fn get_gadget_matrix(n: usize, B: u32, l: usize, q: u64) -> &'static GadgetMatrix {
    unsafe {
        INIT.call_once(|| {
            // Create the gadget matrix with the provided parameters
            let gadget = GadgetMatrix::new(n, B, l, q);
            GADGET_MATRIX = Some(gadget);
        });
        GADGET_MATRIX.as_ref().unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct TgswParams {
    pub tfhe_params: BooleanParameters,
    pub decomp_base: u32,
    pub decomp_level: usize,
}

impl TgswParams {
    pub fn new(tfhe_params: BooleanParameters, decomp_base: u32, decomp_level: usize) -> Self {
        Self {
            tfhe_params,
            decomp_base,
            decomp_level,
        }
    }
}

/*A TGSW ciphertext that encryptes a message M in Z/PZ (P = B^L) with dimension l(n+1)*(n+1)
Note that the implementation is done with type u32, which assumes a ciphertext modulus of 2^32
This is a LIMITATION of the current implementation, and needs to be updated with more generic approach for other q values
*/
#[derive(Serialize, Deserialize)]
pub struct TgswCiphertext {
    ct_matrix: Vec<Vec<u32>>,
    //params: BooleanParameters,
    //dec_base: u32,
    //dec_level: usize,
}

impl TgswCiphertext {
    //Create a new TGSW ciphertext that encrypts a message m in Z/PZ
    //The implementation assumes a q value of 2^32
    pub fn new_encrypt(m: u32, tgsw_params: &TgswParams, client_key: &ClientKey) -> Self {
        let n = tgsw_params.tfhe_params.lwe_dimension.0;
        //let B = 1 << params.pbs_base_log.0;
        //let l = params.pbs_level.0;
        let decomp_level = tgsw_params.decomp_level;
        let decomp_base = tgsw_params.decomp_base;

        let q: u64 = 1 << 32;

        //Create a matrix of size (l*(n+1))*(n+1)
        let mut ct_matrix = vec![vec![0u32; n + 1]; decomp_level * (n + 1)];
        let gadget_matrix = get_gadget_matrix(n, decomp_base, decomp_level, q);
        //gadget_matrix.display();

        //TGSW(m) = Z + mGT where Z is matrix of (n+1)*l rows and (n+1) columns with each row being an ecryption of zero and GT is the gadget matrix
        for i in 0..decomp_level * (n + 1) {
            let temp_0 = client_key.encrypt_abs(0u32);

            //Extract the LWE vector from the LWE ciphertext
            let temp_0_lwe = match temp_0 {
                Ciphertext::Encrypted(ct_lwe) => ct_lwe,
                _ => panic!("Expected encrypted ciphertext"),
            };

            //Further extract the raw data vector from the LWE ciphertext object
            let temp_0_vec = temp_0_lwe.into_container();

            //TGSW(m) = Z + mGT
            for j in 0..n + 1 {
                ct_matrix[i][j] =
                    temp_0_vec[j].wrapping_add(m.wrapping_mul(gadget_matrix.matrix[i][j]));
            }
        }
        Self {
            ct_matrix, /*, dec_base, dec_level , params*/
        }
    }

    /*Perform an external product between TGSW and TLWE ciphertexts
    for C1 = TGSW(M1) and c2 = TLWE(m2), the external product is defined as
    cr = GT^-1(c2) * C1 = GT^-1(c2) * (Z+M1.GT) =  GT^-1(c2) * Z + M1 GT^-1(c2) * GT = LWE(0) + M1c2
    Which means cr is an LWE encryption of M1*m2, given a small "limited" round and multiplicative noise
    See: Marc Joye, "Guide to Fully Homomorphic Encryption over the [Discretized] Torus" : https://eprint.iacr.org/2021/1402
    */
    pub fn ext_product(
        &self,
        ct_lwe: &Ciphertext,
        tgsw_params: &TgswParams, /*, server_key:&ServerKey*/
    ) -> Ciphertext {
        let n = tgsw_params.tfhe_params.lwe_dimension.0;

        //let B = 1 << self.params.pbs_base_log.0;
        let B = tgsw_params.decomp_base;

        //let l = self.params.pbs_level.0;
        let l = tgsw_params.decomp_level;

        let q: u64 = 1u64 << 32;

        //Create a vector of size l(n+1) to stored the decomposed LWE ciphertext

        let ct_lwe_obj = match ct_lwe.clone() {
            Ciphertext::Encrypted(ct_lwe) => ct_lwe,
            _ => panic!("Expected encrypted ciphertext"),
        };
        let mut ct_lwe_vec = ct_lwe_obj.into_container();

        //scale the vector by the factor of B^l/q
        let bl = B.pow(l as u32) as u64;
        let q_2 = q / 2;
        for i in 0..ct_lwe_vec.len() {
            //ct_lwe_vec[i] = (ct_lwe_vec[i] as f64 * (B as f64).powi(l as i32) / q as f64).round() as u32; //This line was working 16-jan-2025
            ct_lwe_vec[i] = ((ct_lwe_vec[i] as u64 * bl + q_2) / q) as u32; //This line is working 16-jan-2025 // It rescales to B^l/q while avoding floating point arithmetic
        }

        let decomposed_lwe_ct = vec_decompose(&ct_lwe_vec, B, l);

        //The external product is provided by performing decomposed_lwe_ct * ct_matrix
        let mut result = vec![0u32; n + 1];
        for i in 0..n + 1 {
            for j in 0..l * (n + 1) {
                result[i] =
                    result[i].wrapping_add(decomposed_lwe_ct[j].wrapping_mul(self.ct_matrix[j][i]));
            }
        }
        let ciphertext_modulus = CiphertextModulus::new_native();
        let x = LweCiphertext::from_container(result, ciphertext_modulus);

        //CAUTION: Bootstrapping here can lead to unintended results (i.e. reducess -3/8 to -1/8).
        //The bootstrapping should be done at the end of the computation.

        /*OB_BOOLEAN_ENGINE.with(|engine_cell| {
             let eng = & mut engine_cell.borrow_mut() as &mut BooleanEngine;
             let bootstrapper = &mut eng.bootstrapper;
             bootstrapper.apply_bootstrapping_pattern(x, &server_key)

        })*/
        Ciphertext::Encrypted(x)
    }
}

// Function to decompose a single integer `n` into `l` digits in base `B`
pub fn decompose(n: u32, B: u32, l: usize) -> Vec<u32> {
    let mut n = n; // Create a mutable copy of n
    let mut decomposed: Vec<u32> = Vec::with_capacity(l);

    // Perform the decomposition
    for _ in 0..l {
        decomposed.push(n % B); // Get the remainder
        n = n / B; // Integer division
    }

    // Reverse the list to match SageMath's behavior
    decomposed.reverse();

    decomposed
}

// Function to decompose each element of a vector `v` into base `B` with `l` digits
pub fn vec_decompose(v: &[u32], B: u32, l: usize) -> Vec<u32> {
    let mut decomposed_vec: Vec<u32> = Vec::with_capacity(v.len() * l);

    // Iterate over each element in the input vector `v`
    v.iter().for_each(|&val| {
        let mut decomposed_val = decompose(val, B, l);
        decomposed_vec.append(&mut decomposed_val); // Extend the result with the decomposed value
    });

    decomposed_vec
}

pub fn vec_scale(mut v: Vec<u32>, B: u32, l: usize, q: u64) -> Vec<u32> {
    // Iterate over each element in the input vector `v`
    for i in 0..v.len() {
        v[i] = (v[i] as f64 * (B as f64).powi(l as i32) / q as f64).round() as u32;
    }
    v
}
