use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::{GuildId},
        interactions::{
            application_command::ApplicationCommandOptionType, Interaction, InteractionResponseType,
        }, 
    },
    prelude::*,
};

use wallet::data::DbClient;
pub mod data;
mod wallet;
pub struct Handler {
    db_client: DbClient,
    pre_role: String,
    post_role: String
}

// const ERROR_POSTFIX: &str = ". Follow the guide here <some link>";
const ERROR_POSTFIX: &str = "";
#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "sign" => match wallet::sign(&ctx, &command, &self).await {
                    Ok(_) => "Your details have been recorded.".to_string(),
                    Err(e) => format!("{} {}", e, ERROR_POSTFIX),
                },
                "wallet" => match wallet::register(&ctx, &command, &self.db_client, &self).await {
                    Ok(_) => "Your details have been recorded.".to_string(),
                    Err(e) => e,
                },
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            dotenv::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("sign")
                        .description("Register and verify wallet")
                        .create_option(|option| {
                            option
                                .name("kusama_address")
                                .description("Kusama wallet address")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("moonriver_address")
                                .description("Moonriver wallet address")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("signature")
                                .description(
                                    "MOVR address signed as a message with your KSM wallet",
                                )
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command
                        .name("wallet")
                        .description("Register user wallet")
                        .create_option(|option| {
                            option
                                .name("type")
                                .description("Type of wallet")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                                .add_string_choice("Moonriver", "Moonriver")
                                .add_string_choice("Kusama", "Kusama")
                        })
                        .create_option(|option| {
                            option
                                .name("address")
                                .description("The wallet address")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
        })
        .await;

        let roles = guild_id.roles(&ctx.http).await.unwrap();
        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );

        println!(
            "I found the following roles: {:#?}",
            roles
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_file = dotenv::var("DB_FILE").expect("Expected DB File in the environment");
    let db_client = data::init(db_file).await;
    // user needs this role before they can use /sign command
    let pre_role = dotenv::var("PRE_ROLE").expect("Expected pre role in the environment");
    // user is assigned this role after successfully using the /sign command
    let post_role = dotenv::var("POST_ROLE").expect("Expected post role in the environment");

    let handler = Handler { db_client, pre_role, post_role };

    let application_id: u64 = dotenv::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let mut client = Client::builder(token)
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

impl Handler {
    fn db_client(&self) -> &DbClient {
        &self.db_client
    }
    
    fn is_valid_role(&self, user_role: &str) -> bool {
        user_role.eq(&self.pre_role)
    }

    fn post_role(&self) -> &str {
        &self.post_role
    }
}