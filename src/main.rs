use std::env;
use std::fs;
mod commands;
mod ai_manager;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::guild::automod::Trigger;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use lazy_static::lazy_static; 
use std::sync::Mutex;
use serde::{Deserialize,Serialize};
struct Handler;
lazy_static! {
    static ref OPENAI_KEY: Mutex<String> = Mutex::new(String::new());
    static ref DISCORD_KEY: Mutex<String> = Mutex::new(String::new());
    static ref TRIGGERS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}
#[derive(Serialize, Deserialize)]
struct Settings{
    triggers: Vec<String>,
    discord_token: String,
    openai_token: String
}

fn load_settings(){
    let settings = fs::read_to_string("./settings.json")
    .expect("Settings file not found. Change settings_example.json to settings.json and populate it with relevant data.");
    let json: Settings = serde_json::from_str(&settings)
    .expect("JSON does not have correct format.");

    *OPENAI_KEY.lock().unwrap() = json.openai_token;
    *DISCORD_KEY.lock().unwrap() = json.discord_token;
    *TRIGGERS.lock().unwrap() = json.triggers;
}

fn check_for_trigger(keyword: &str, triggers: &Vec<String>) -> bool {
    for trigger in triggers {
        if keyword.contains(trigger) {
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
                "change" => ai_manager::change_model(commands::neuro::parse(&command.data.options)),
                "set_rule" => ai_manager::set_rule(commands::rule::parse(&command.data.options)),
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
        let is_self: bool = &msg.author.id == &context.cache.current_user_id();
        for user in msg.mentions.iter(){
            if(user.id == context.cache.current_user_id()){
                is_reply = true;
            }
        }
        if check_for_trigger(&msg.content.to_lowercase(), &TRIGGERS.lock().unwrap()) && !is_self || is_reply {
            println!("Message recieved: {}", &msg.content);
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);
                    return;
                },
            };
            ai_manager::add_message(ai_manager::ChatMessage { content: msg.content.clone(), author: msg.author.to_string(), channel: msg.channel_id.into()});
            let response = MessageBuilder::new()
                .push(ai_manager::get_response(msg.channel_id.into()).await)
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
        let rule_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::rule::register(command)
        })
        .await;
        println!("I created the following global slash command: {:#?}", rule_command);
    }
}

#[tokio::main]
async fn main() {
    load_settings();

    // Load tokens, use settings.json as a backup if environment variables don't exist.
    let token = env::var("DISCORD_TOKEN").unwrap_or(DISCORD_KEY.lock().unwrap().to_string());
    let ai_token = env::var("OPENAI_API_KEY").unwrap_or(OPENAI_KEY.lock().unwrap().to_string());

    // Throw error if unable to get token from either the environment or settings.json
    assert_eq!(token.is_empty(), false, "TOKEN UNAVAILABLE CHECK THE 'settings.json' FILE OR 'DISCORD_TOKEN' ENVIRONMENT VARIABLE.");
    assert_eq!(ai_token.is_empty(), false, "TOKEN UNAVAILABLE CHECK THE 'settings.json' FILE OR 'OPENAI_API_KEY' ENVIRONMENT VARIABLE.");
    
    // Reset the environment token of openai (Incase it doesn't exist, the token from settings.json is passed on to the environment )
    env::set_var("OPENAI_API_KEY", ai_token);

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}