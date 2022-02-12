use std::str::FromStr;

use serenity::model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
};
use sp_core::crypto::Ss58Codec;

use ethereum_types;

pub fn register(command: &ApplicationCommandInteraction) -> Result<(), String> {
    let address_type = command
        .data
        .options
        .get(0)
        .expect("Expected wallet type")
        .resolved
        .as_ref()
        .expect("Expected wallet type object");
    let address = command
        .data
        .options
        .get(1)
        .expect("Expected address")
        .resolved
        .as_ref()
        .expect("Expected address object");

    if let ApplicationCommandInteractionDataOptionValue::String(address_type) = address_type {
        if let ApplicationCommandInteractionDataOptionValue::String(address) = address {
            match verify(address_type, address) {
                Ok(_) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    Err("Something went wrong while processing the command.".to_string())
}

fn verify(address_type: &String, address: &String) -> Result<(), String> {
    if address_type.eq("Moonriver") {
        return check_h160(address);
    } else if address_type.eq("Kusama") {
      return check_ss58(address);
    }
    Err("The provided wallet address is invalid.".to_string())
}

fn check_h160(address: &String) -> Result<(), String> {
    match ethereum_types::H160::from_str(address) {
        Ok(_) => return Ok(()),
        Err(e) => {
            print!("Error while saving moonriver address: {}", e.to_string());
            return Err("Invalid H160 address provided".to_string());
        }
    }
}

fn check_ss58(address: &String) -> Result<(), String> {
    if let Ok(_) = sp_core::crypto::AccountId32::from_ss58check(&address) {
        return Ok(());
    }
    
    return Err("Invalid ss58 address provided".to_string());
}
