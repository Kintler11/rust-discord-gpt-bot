use serenity::model::{channel::Message, prelude::automod::Rule};
use async_openai as openai;
use openai::{Client, Completion, types::{CreateCompletionRequest}};
use lazy_static::lazy_static; 
use std::sync::Mutex;
use serenity::prelude::*;
pub struct ChatMessage{
    pub content: String,
    pub author: String,
    pub channel: u64
}
pub enum ChatEngine{
    Ada,
    Babbage,
    Currie,
    Davinci1,
    Davinci2,
    Davinci3
}

pub struct MessageLog{
    pub messages: Vec<ChatMessage>,
    pub length: u32
}
pub struct RuleLog{
    pub rules: Vec<String>
}
impl Default for MessageLog {
    fn default () -> MessageLog {
        MessageLog{messages: Vec::new(), length:10}
    }
    
}
impl Default for RuleLog {
    fn default () -> RuleLog {
        RuleLog{rules: Vec::new()}
    }
    
}

lazy_static! {
    static ref LOG: Mutex<MessageLog> = Mutex::new(MessageLog::default());
    static ref RULES: Mutex<RuleLog> = Mutex::new(RuleLog::default());
    static ref MODEL: Mutex<ChatEngine> = Mutex::new(ChatEngine::Davinci3);
}

// Clear bots memory
pub fn clear_log() -> String{
    LOG.lock().unwrap().messages = Vec::new();
    RULES.lock().unwrap().rules = Vec::new();

    "Memory Cleared".to_string()
}

pub fn add_message(msg: ChatMessage){

    let log_ref = LOG.lock().unwrap();

    if log_ref.messages.len() >= log_ref.length as usize {LOG.lock().unwrap().messages.remove(0);}

    LOG.lock().unwrap().messages.push(msg);
}
pub fn set_rule(rule: String) -> String{
    let message = format!("'{}' - Rule set. Total rules: {} ", &rule, RULES.lock().unwrap().rules.len() + 1);
    clear_log();
    RULES.lock().unwrap().rules.push(rule);
    message
}

pub fn get_chat_log() -> String{
    // Format messagelog to a chatlog format
    // User: blablabla
    // Bot: blablabla
    let mut created_prompt = "".to_owned();
    for rule in RULES.lock().unwrap().rules.iter() {
        created_prompt.push_str(&format!("{}\n",rule));
    }
    for val in LOG.lock().unwrap().messages.iter() {
        created_prompt.push_str(&format!("{}: {}\n",val.author, val.content));
    }

    // Add wildcard to chatlog for the bot to latch on to
    created_prompt.push_str("You: ");
    created_prompt
}

pub fn get_model() -> String{
    match *MODEL.lock().unwrap(){
        ChatEngine::Ada => "text-ada-001".to_string(),
        ChatEngine::Babbage => "text-babbage-001".to_string(),
        ChatEngine::Currie => "text-currie-001".to_string(),
        ChatEngine::Davinci1 => "text-davinci-001".to_string(),
        ChatEngine::Davinci2 => "text-davinci-002".to_string(),
        ChatEngine::Davinci3 => "text-davinci-003".to_string(),
    }
}

pub fn change_model(name: String) -> String{
    let change_message: &str;
    // Change the model based on the request
    (*MODEL.lock().unwrap(), change_message) = match &name.to_lowercase()[..] {
        "ada" => (ChatEngine::Ada, "Changed brain to Ada"),
        "babbage" => (ChatEngine::Babbage, "Changed brain to Babbage"),
        "currie" => (ChatEngine::Currie, "Changed brain to Currie"),
        "davinci1" => (ChatEngine::Davinci1, "Changed brain to Davinci-001"),
        "davinci2" => (ChatEngine::Davinci2, "Changed brain to Davinci-002"),
        "davinci3" => (ChatEngine::Davinci3, "Changed brain to Davinci-003"),
        _ => (ChatEngine::Davinci1, "Changed brain to Davinci-001")
    };
    println!("model: {}", get_model());

    change_message.to_owned()
}

pub async fn get_response(channel_id: u64) -> String {
    // Create OpenAi Client
    let client = Client::new();
    // Set OpenAI Model settings
    let request = CreateCompletionRequest {
        model: get_model(),
        prompt: Some(get_chat_log()),
        ..Default::default()
    };
    // Get response from OpenAI Model
    let response = Completion::create(&client, request).await.unwrap();
    // Add bot response to message log
    add_message(ChatMessage{content: response.choices.first().unwrap().text.to_string(), author:"You:".to_string(), channel:channel_id });
    return response.choices.first().unwrap().text.to_string();
}