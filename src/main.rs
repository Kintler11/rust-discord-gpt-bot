use std::env;
mod commands;
mod ai_manager;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
const STATUS: &str = "I am reborn of ashes left by the windows disk formatter.";
struct Handler;
fn check_substring(string: &str, vec: &Vec<&str>) -> bool {
    for substr in vec {
        if string.contains(substr) {
            return true;
        }
    }
    false
}
#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "clean" => ai_manager::clear_log(),
                "change" => commands::neuro::run(&command.data.options),
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
    async fn message(&self, context: Context, msg: Message) {
        let mut is_reply: bool = false;
        let triggers = vec!["kait", "among", "sus", "vent", "winton"];
        let is_self: bool = &msg.author.id == &context.cache.current_user_id();
        for user in msg.mentions.iter(){
            if(user.id == context.cache.current_user_id()){
                is_reply = true;
            }
        }
        if check_substring(&msg.content.to_lowercase(), &triggers) && !is_self || is_reply {
            println!("Message Recieved {}", &msg.content);
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);
                    return;
                },
            };
            ai_manager::add_message(ai_manager::ChatMessage { content: msg.content.clone(), author: msg.author.to_string(), channel: msg.channel_id.into()});
            let ai_response = ai_manager::get_response(msg.channel_id.into());
            let response = MessageBuilder::new()
                .push(ai_response.await)
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let clean_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::clean::register(command)
        })
        .await;
        let brain_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::neuro::register(command)
        })
        .await;
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}