use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::ApplicationCommandOptionType, Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};

use wallet::data::DbClient;
pub mod data;
mod wallet;
struct Handler {
    db_client: DbClient,
}

const GUIDE: &str = "<some link>";
#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "register_ksm_movr" => match wallet::register(&command, &self.db_client).await {
                    Ok(_) => "Your details have been recorded.".to_string(),
                    Err(e) => format!("{}. Follow the guide here {}", e, GUIDE)
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
                        .name("register_ksm_movr")
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
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let db_file = dotenv::var("DB_FILE").expect("Expected DB File in the environment");
    let db_client = data::init(db_file).await;
    let handler = Handler { db_client };

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
