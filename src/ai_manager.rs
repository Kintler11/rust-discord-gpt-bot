use serenity::model::channel::Message;
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

pub struct MessageLog{
    pub messages: Vec<ChatMessage>
}
impl Default for MessageLog {
    fn default () -> MessageLog {
        MessageLog{messages: Vec::new()}
    }
}

lazy_static! {
    static ref LOG: Mutex<MessageLog> = Mutex::new(MessageLog{messages:Vec::new()});
    static ref MODEL: Mutex<String> = Mutex::new(String::from("text-davinci-003"));
}

pub fn get_log_length() -> usize{
    let length = LOG.lock().unwrap().messages.len();
    return length;
}

pub fn clear_log() -> String{
    LOG.lock().unwrap().messages = Vec::new();
    return "Memory Cleared".to_string();
}

pub fn add_message(msg: ChatMessage) -> bool{
    if(get_log_length() >= 10){LOG.lock().unwrap().messages.remove(0);}
    LOG.lock().unwrap().messages.push(msg);
    return true;
}

pub async fn get_response(channel_id: u64) -> String {
    // Create OpenAi Client
    let client = Client::new();

    // Format messagelog to a chatlog format
    // Person1: blablabla
    // Person2: blablabla
    let mut created_prompt = "".to_owned();
    for val in LOG.lock().unwrap().messages.iter() {
        created_prompt.push_str(&val.author[..]);
        created_prompt.push_str(": ");
        created_prompt.push_str(&val.content[..]);
        created_prompt.push_str("\n");
    }
    // Add wildcard to chatlog for the bot to latch on
    created_prompt.push_str("You: ");
    println!("{}",created_prompt);
    // Set OpenAI Model settings
    let request = CreateCompletionRequest {
        model: MODEL.lock().unwrap().to_string(),
        prompt: Some(created_prompt.to_owned()),
        stop: Some("You: ".to_string()),
        ..Default::default()
    };
    // Get response from OpenAI Model
    let response = Completion::create(&client, request).await.unwrap();

    // Add bot response to messagelog
    add_message(ChatMessage{content: response.choices.first().unwrap().text.to_string(), author:"You:".to_string(), channel:channel_id });
    return response.choices.first().unwrap().text.to_string();
}