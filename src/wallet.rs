use schnorrkel::keys::*;
use schnorrkel::sign::Signature;
use schnorrkel::signing_context;
use std::{str::FromStr, io::Stderr};

use serenity::model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
};
use sp_core::crypto::{AccountId32, Ss58Codec};

pub use crate::data;
use ethereum_types;

use self::data::DbClient;

pub async fn sign(
    command: &ApplicationCommandInteraction,
    db_client: &DbClient,
) -> Result<(), String> {
    let ksm = extract_option_str(command, 0).unwrap();
    let movr = extract_option_str(command, 1).unwrap();
    let signature = extract_option_str(command, 2).unwrap();

    match check_ss58(&ksm) {
        Ok(_) => (),
        Err(_) => return Err("Invalid ksm address".to_string()),
    };

    match check_h160(&movr) {
        Ok(_) => (),
        Err(_) => return Err("Invalid movr address".to_string()),
    };

    match check_signature(&ksm, &movr, &signature) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match insert_signed(db_client, command, ksm, movr).await {
        Ok(_) => (),
        Err(_) => {
            return Err("Something went wrong while trying to record your details".to_string())
        }
    }

    Ok(())
}

pub async fn register(
    command: &ApplicationCommandInteraction,
    db_client: &DbClient,
) -> Result<(), String> {
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
                Ok(_) => match insert_non_signed(
                    db_client,
                    command,
                    address_type.to_string(),
                    address.to_string(),
                ).await
                {
                    None => return Ok(()),
                    Some(_) => return Err("Could not save the record".to_string()),
                },
                Err(e) => return Err(e),
            }
        }
    }

    Err("Something went wrong while processing the command.".to_string())
}

fn extract_option_str(command: &ApplicationCommandInteraction, index: usize) -> Option<String> {
    let val: &ApplicationCommandInteractionDataOptionValue = command
        .data
        .options
        .get(index)
        .expect("Expected value")
        .resolved
        .as_ref()
        .expect("Expected object");

    if let ApplicationCommandInteractionDataOptionValue::String(val) = val {
        return Some(val.to_string());
    }

    None
}

fn check_signature(ss58_add: &String, h160_add: &String, signature: &String) -> Result<(), String> {
    let msg = format!("<Bytes>{}</Bytes>", h160_add);
    let context = signing_context(b"substrate");

    match check_h160(&format!("0x{}", h160_add)) {
        Ok(_) => (),
        Err(_) => return Err("Movr address is not valid".to_string()),
    }

    let sig: Vec<u8> = match hex::FromHex::from_hex(signature) {
        Ok(sign) => sign,
        Err(_) => return Err("Input signature is not a hex.".to_string()),
    };

    let sig = match Signature::from_bytes(sig.as_slice()) {
        Ok(sign) => sign,
        Err(_) => return Err("Input signature could not be parsed.".to_string()),
    };

    let acc = match AccountId32::from_string_with_version(ss58_add) {
        Ok(acc32) => acc32,
        Err(_) => return Err("Input substrate address not valid.".to_string()),
    };

    let pk = match PublicKey::from_bytes(acc.0.as_ref()) {
        Ok(some_pk) => some_pk,
        Err(_) => {
            return Err("Something went wrong while trying to parse substrate address.".to_string())
        }
    };

    match pk.verify(context.bytes(msg.as_bytes()), &sig) {
        Ok(_) => return Ok(()),
        Err(_) => return Err("Signature could not be verified.".to_string()),
    }
}

#[test]
fn test_signature() {
    let ss58_address = &"Fk5WEp12UPQJK7ibjA1SjryQUJHDXYJ1sqavX7kUHzi4nbU".to_string();
    let h160_add = &"b794f5ea0ba39494ce839613fffba74279579268".to_string();
    let signature = &"f0148a9053ad65d80b39b5c8d6957359511cadfe872fabbb3cb14829bd4324081e3f7cf405c3aa4774dbf50dd7e2dcb9a227298f386a331659fbc9b9d2fe478d".to_string();

    assert!(check_signature(ss58_address, h160_add, signature).is_ok());
}

async fn insert_signed(
    db_client: &DbClient,
    command: &ApplicationCommandInteraction,
    ksm: String,
    movr: String,
) -> Result<(), String> {
    let roles = &command
        .member
        .as_ref()
        .expect("Expected the bot to be in guild")
        .roles;
    let avatar = &command.user.avatar_url().unwrap_or_default();
    let success = db_client
        .insert_signed(
            command.user.id.to_string(),
            command.user.tag(),
            command.user.name.to_string(),
            ksm,
            format!("0x{}", movr),
            format!("{:?}", roles),
            avatar.to_string(),
        )
        .await
        .is_none();

    if success {
        return Ok(());
    }

    Err("Could not save wallet details".to_string())
}

async fn insert_non_signed(
    db_client: &DbClient,
    command: &ApplicationCommandInteraction,
    address_type: String,
    address: String,
) -> Option<Stderr> {
    let roles = &command
        .member
        .as_ref()
        .expect("Expected the bot to be in guild")
        .roles;
    let avatar = &command.user.avatar_url().unwrap_or_default();
    db_client
        .insert_non_signed(
            command.user.tag(),
            command.user.name.to_string(),
            address_type.to_string(),
            address.to_string(),
            format!("{:?}", roles),
            avatar.to_string(),
        )
        .await
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
            // return Err("Invalid H160 address provided".to_string());
            return Ok(());
        }
    }
}

fn check_ss58(address: &String) -> Result<(), String> {
    if let Ok(_) = AccountId32::from_ss58check(&address) {
        return Ok(());
    }

    return Err("Invalid ss58 address provided".to_string());
}
