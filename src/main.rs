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
mod wallet;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "wallet" => match wallet::register(&command) {
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
            commands.create_application_command(|command| {
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

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = dotenv::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = dotenv::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
