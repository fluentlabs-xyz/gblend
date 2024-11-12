use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::TransactionRequest;
use ethers::core::utils::hex;
use std::io;
use std::{env, fs, str::FromStr, sync::Arc, error::Error};
use anyhow::{Context, Ok, Result};



pub async fn deploy_wasm_contract(private_key:&str,binary_path:&str) -> Result<()>{

    let provider = Provider::<Http>::try_from(
        "https://rpc.dev.thefluent.xyz/"
    ).expect("could not instantiate HTTP Provider");

    let wallet: LocalWallet =private_key.parse()?;

    let mut client = SignerMiddleware::new(provider, wallet);

    //WASM data 
    let data = wasm_to_string(binary_path)?;
    let tx_data= format!("0x{}", data);


    println!("Binary data : {}", tx_data);
    //I'll need to calculate the gas fee 
    Ok(())
}

fn wasm_to_string(path: &str) -> Result<String> {
    // Read the file as bytes
    let file = fs::read(path)
        .with_context(|| format!("Failed to read file at path: {}", path))?;

    // Encode bytes to hex 
    let data = hex::encode(file);
    
    Ok(data)
}



#[cfg(test)]

mod test{
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_wasm_to_string()-> Result<()>{
        let data = wasm_to_string("src/deployrs/utils/greeting.wasm")?;
        assert!(!data.is_empty(), "The WASM string should not be empty");
        Ok(())
    }
}