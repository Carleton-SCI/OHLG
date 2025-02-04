# OHLG (Oblivious Homomorphic Logic Gates)
## Overview
This repo provides an unoptimized prototype for the Oblivious Homomorphic Logic Gates scheme [1]. The system allows executing an obfuscated circuit on encrypted data, and producing encrypted output. So it basically hides the inputs, the process, and the output from the operator server who does the processing. The system is visualized in the figure below.
![System Overview figure](Sys_overview.png)
The system consists of two parties, the client and the server:  
* **Client**: The client is the owner of the data and the circuit, but with low computing and storage power. It encrypts the data to be processed, encrypts the gates' parameters, serializes the data flow description, and sends them along with the HE bootstrapping key to the server side. The client then decrypts the evaluation results sent back from the server.
* **Server**: The server, which has more computing power, executes the circuit, built with obfuscated gates according to the data flow description, on the encrypted data using the encrypted gates' parameters and sends the encrypted results to the client.

##Repo structure
There are two main directories:
1. **ohlg**: A rust crate that contains the implementation of the oblivious logic gates and an oblivious matching application.
2. **tfhe-rs**: A modified-APi version of the TFHE implementation by Zama. The original TFHE by Zama can be found [here](https://github.com/zama-ai/tfhe-rs "Zama TFHE-rs")

## Getting started
The code was tested on Ubuntu 24.04.01 LTS and Windows 11
### General steps
1. Make sure to have Rust and Cargo installed in your system [https://doc.rust-lang.org/cargo/getting-started/installation.html]
2. Clone the repo
```
git clone https://github.com/Carleton-SCI/OHLG.git
```
3. Change directory to ohlg
```
cd ohlg
```
4.* **For Windows**, open ```ohlg/Cargo.toml``` and modify the following line in the dependencies
```
tfhe = { path = "../tfhe-rs/tfhe", features = [ "boolean", "x86_64-unix" ] }
```
to be
```
tfhe = { path = "../tfhe-rs/tfhe", features = [ "boolean", "x86_64" ] }
```
For Linux, leave it as is.

### Gate benchmarking
The benchmarking of a single gate performance is implemented in the main binary file ```main.rs```, which can be run through:
```
cargo run --bin ohlg --release
```
The benchmarking tests an obfuscated "XNOR" operation with bith inputs are zeros. The additive parameter is 6/8=-2/8 and the multiplicative parameter is 2. You can change these parameters to implement other gates by changing the following values in ```main.rs```
```Rust
//----Perform an Obfuscated logic operation (XNOR) between two ciphertexts----
    //m1 and m2 are the Boolean plaintext values to be encrypted
    //M is the plaintext value of the multiplicative parameter (=2 for XNOR operation)
    //d is the plaintext value of the additive parameter (=6/8=-2/8 for XNOR operation)
    
    let m1:bool = false;
    let m2:bool = false;
    let M:u32 = 2;
    let d: u32 = (1 << (32-3))*6 ;
    let opr_str = "XNOR";
```
The implementation also allows for choosing the TFHE-io parameters ([link](https://tfhe.github.io/tfhe/security_and_params.html)) or the default Boolean TFHE-rs parameters ([link](https://github.com/zama-ai/tfhe-rs)). You can choose between them by editing the value of the ```param_choice``` variable in ```main.rs``` to be ```TFHE_RS``` or ```TFHE_IO```
```Rust
let param_choice = "TFHE_RS";
```
### ODM (Oblivious Direct Matching) Application
The application works as follows:
1. **Client**
   * Prepare and serialize the data flow description of the matching circuit.
   * Encrypt the parameters of the gates.
   * Encrypt the query data, if it is not already stored on the server.
   * Send the serialized data flow description, encrypted gate parameters, encrypted query data, and the bootstrapping key to the server.
2. **Server**
   * Execute the matching circuit, as defined by the received data flow description and using the encrypted gate parameters, on the encrypted query data and the encrypted search corpus.
   * Return the encrypted result to the client.
3. **Client**
   * Decrypt the result to interpret the matching outcome.

To run the experiment, first execute:
```
cargo run --bin client_odm --release
```
This creates and seralizes the keys, the search character ciphertext, the search corpus ciphertxt, and the encrypted gate parameters. The code asks the user to enter the search character should be encrypted and sent to the server. The code also contains a sample search corpus of "ABCDEFGHIJ" which can be changed in ```ohlg/bin/client_odm.rs```
```Rust
let plain_search_corpus = "ABCDEFGHIJ";
```
Changing the length of the search corpus (originally 10 bytes) is not tested.  

The second step is to run the server process by:
```
cargo run --bin server_odm --release
```
which does the obfuscated processing. **Note** that reading the encrypted gate parameters might take a while because the data serialization technique (save and read from disk) is not optimized, this is just a prototype.

The third step is to run the verification process, which is typically at the client side, by:
```
cargo run --bin verif_odm --release
```
which decrypts and prints the matching result.

# Licensing
This software is built in and on top of Zama's TFHE implementation, for which we include there license as [requested](https://github.com/zama-ai/tfhe-rs/blob/main/LICENSE).

BSD 3-Clause Clear License

Copyright © 2024 ZAMA.
All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this
list of conditions and the following disclaimer in the documentation and/or other
materials provided with the distribution.

3. Neither the name of ZAMA nor the names of its contributors may be used to endorse
or promote products derived from this software without specific prior written permission.

NO EXPRESS OR IMPLIED LICENSES TO ANY PARTY'S PATENT RIGHTS ARE GRANTED BY THIS LICENSE.
THIS SOFTWARE IS PROVIDED BY THE ZAMA AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL
ZAMA OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY,
OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED ANDON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF
ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


# References
[1] Mahmoud Abdelhafeez Sayed and Mostafa Taha, Oblivious Homomorphic Logic Gates, Journal of Cryptographic Engineering, 2025

---

