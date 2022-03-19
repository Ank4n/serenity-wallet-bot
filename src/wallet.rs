use schnorrkel::keys::*;
use schnorrkel::sign::Signature;
use schnorrkel::signing_context;
use std::{io::Stderr, str::FromStr};

use serenity::{
    client::Context,
    model::{
        interactions::application_command::{
            ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
        },
    },
};
use sp_core::crypto::{AccountId32, Ss58Codec};

pub use crate::data;
use crate::Handler;

use ethereum_types;

use self::data::DbClient;

pub async fn sign(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    handler: &Handler,
) -> Result<(), String> {
    let ksm = extract_option_str(command, 0).unwrap();
    let movr = extract_option_str(command, 1).unwrap();
    let signature = extract_option_str(command, 2).unwrap();

    let member = &command
    .member
    .as_ref()
    .expect("Expected user to be member of guild");
    let roles = command
        .guild_id
        .expect("Expected command to come from the guild")
        .roles(&ctx.http)
        .await
        .unwrap();
    let user_roles = &member.roles;
    let user_roles = user_roles.iter().map(|role_id| {
        &roles
            .get(role_id)
            .expect("expected role id in the guild")
            .name
    });

    let filtered_roles = user_roles
        .to_owned()
      //  .filter(|&role_name| handler.is_valid_role(&role_name))
        .collect::<Vec<&std::string::String>>();

    let user_roles = user_roles.collect::<Vec<&std::string::String>>();

    if filtered_roles.len() != 1 {
        return Err("You do not have proper role to use this command.".to_string());
    }

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

    match insert_signed(&handler.db_client(), command, ksm, movr, user_roles).await {
        Ok(_) => (),
        Err(_) => {
            return Err("Something went wrong while trying to record your details".to_string())
        }
    }

    Ok(())
}

pub async fn register(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db_client: &DbClient,
    handler: &Handler,
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
        let member = &command
        .member
        .as_ref()
        .expect("Expected user to be member of guild");
        let roles = command
            .guild_id
            .expect("Expected command to come from the guild")
            .roles(&ctx.http)
            .await
            .unwrap();
        let user_roles = &member.roles;
        let user_roles = user_roles.iter().map(|role_id| {
            &roles
                .get(role_id)
                .expect("expected role id in the guild")
                .name
        });
        println!("User roles: {:?}", user_roles);
        let filtered_roles = user_roles
            .to_owned()
            .filter(|&role_name| handler.is_valid_role(&role_name))
            .collect::<Vec<&std::string::String>>();
        println!("Filtered roles: {:?}", filtered_roles);
        let user_roles = user_roles.collect::<Vec<&std::string::String>>();
    
        if filtered_roles.len() != 1 {
            let msg = format!("You do not have proper role to use this command." );
            return Err(msg);
        }
    if let ApplicationCommandInteractionDataOptionValue::String(address_type) = address_type {
        if let ApplicationCommandInteractionDataOptionValue::String(address) = address {
            match verify(address_type, address) {
                Ok(_) => match insert_non_signed(
                    db_client,
                    command,
                    address_type.to_string(),
                    address.to_string(),
                )
                .await
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

const MSG_WRAP_PREFIX: &str = "<Bytes>";
const MSG_WRAP_POSTFIX: &str = "</Bytes>";

fn check_signature(ss58_add: &String, h160_add: &String, signature: &String) -> Result<(), String> {

    let h160_add = if h160_add.starts_with("0x") {
        h160_add[2..].to_string()
    } else {
        h160_add.to_string()
    };

    let signature = if signature.starts_with("0x") {
        signature[2..].to_string()
    } else {
        signature.to_string()
    };

    let mut unwrapped_msg: Vec<u8> = match hex::FromHex::from_hex(&h160_add) {
        Ok(m) => m,
        Err(_) => return Err("Not a valid hex message".to_string()),
    };

    let mut msg = MSG_WRAP_PREFIX.as_bytes().to_vec();
    let mut msg_postfix = MSG_WRAP_POSTFIX.as_bytes().to_vec();

    msg.append(&mut unwrapped_msg);
    msg.append(&mut msg_postfix);

    let context = signing_context(b"substrate");

    match check_h160(&format!("0x{}", &h160_add)) {
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

    match pk.verify(context.bytes(&msg), &sig) {
        Ok(_) => return Ok(()),
        Err(_) => return Err("Signature could not be verified.".to_string()),
    }
}

async fn insert_signed(
    db_client: &DbClient,
    command: &ApplicationCommandInteraction,
    ksm: String,
    movr: String,
    roles: Vec<&String>,
) -> Result<(), String> {
    let avatar = &command.user.avatar_url().unwrap_or_default();
    let success = db_client
        .insert_signed(
            command.user.id.to_string(),
            command.user.tag(),
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
            command.user.id.to_string(),
            command.user.tag(),
            address_type.to_string(),
            address.to_string(),
            format!("{:?}", roles),
            avatar.to_string(),
        )
        .await
}

fn verify(address_type: &String, address: &String) -> Result<(), String> {
    if address_type.eq("Moonbeam") {
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
            print!("Error while parsing Moonbeam address: {}", e.to_string());
            return Err("Invalid H160 address provided".to_string());
        }
    }
}

fn check_ss58(address: &String) -> Result<(), String> {
    if let Ok(_) = AccountId32::from_ss58check(&address) {
        return Ok(());
    }

    return Err("Invalid ss58 address provided".to_string());
}

#[test]
fn test_signature_unstripped_hex() {
    let ss58_address = &"14AkzFjCFtdwzCJnnfPxgwL87W1h7AHFdzjKh9q9YaojWFxx".to_string();
    let h160_add = &"0xb794f5ea0ba39494ce839613fffba74279579268".to_string();
    let signature = &"0xc67b20ee54a52ba6636e8f41f7aa984a47916ef17a119d441d29a97ac6ebfa6921f649cd3a02084df393a6614f3ac699aca98bdb5ccf5504dd74fd6e3f6dd48a".to_string();
    let check = check_signature(ss58_address, h160_add, signature);
    assert!(check.is_ok(), "err: {}", check.unwrap_err());
}

#[test]
fn test_signature_stripped_hex() {
    let ss58_address = &"14AkzFjCFtdwzCJnnfPxgwL87W1h7AHFdzjKh9q9YaojWFxx".to_string();
    let h160_add = &"b794f5ea0ba39494ce839613fffba74279579268".to_string();
    let signature = &"c67b20ee54a52ba6636e8f41f7aa984a47916ef17a119d441d29a97ac6ebfa6921f649cd3a02084df393a6614f3ac699aca98bdb5ccf5504dd74fd6e3f6dd48a".to_string();
    let check = check_signature(ss58_address, h160_add, signature);
    assert!(check.is_ok(), "err: {}", check.unwrap_err());
}
#[test]
fn test_signature_wrong_signature() {
    let ss58_address = &"14AkzFjCFtdwzCJnnfPxgwL87W1h7AHFdzjKh9q9YaojWFxx".to_string();
    let h160_add = &"b794f5ea0ba39494ce839613fffba74279579268".to_string();
    let signature = &"367b20ee54a52ba6636e8f41f7aa984a47916ef17a119d441d29a97ac6ebfa6921f649cd3a02084df393a6614f3ac699aca98bdb5ccf5504dd74fd6e3f6dd48a".to_string();
    let check = check_signature(ss58_address, h160_add, signature);
    assert!(check.is_err(), "Signature was expected to fail but passed");
}
