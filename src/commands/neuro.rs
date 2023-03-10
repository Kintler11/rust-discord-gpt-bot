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

    if let CommandDataOptionValue::String(brain)= option {
        brain.to_string()
    } else {
        "".to_string()
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("change").name_localized("fi", "vaihda").description("Change the brains of the model").description_localized("fi", "Vaihda botin aivot").create_option(|option| {
        option
            .name("brain")
            .name_localized("fi", "aivot")
            .description("Ada, Babbage, Currie, Davinci1, Davinci2, Davinci3")
            .kind(CommandOptionType::String)
            .required(true)
    })
}