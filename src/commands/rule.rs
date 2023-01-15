use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};

pub fn parse(options: &[CommandDataOption]) -> String {
    let option = options
        .get(0)
        .expect("Expected user option")
        .resolved
        .as_ref()
        .expect("Expected user object");

    if let CommandDataOptionValue::String(rule)= option {
        rule.to_string()
    } else {
        "".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("set_rule").description("Set a rule for the bot. Setting a rule will reset the memory of the bot.").create_option(|option| {
        option
            .name("rule")
            .description("Example: Act like a dog, only answer in riddles")
            .kind(CommandOptionType::String)
            .required(true)
    })
}