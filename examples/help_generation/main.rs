use ::serenity::small_fixed_array::FixedString;
use poise::{samples::HelpConfiguration, serenity_prelude as serenity};
use rand::Rng;

type Data = (); // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const FRUIT: &[&str] = &["🍎", "🍌", "🍊", "🍉", "🍇", "🍓"];
const VEGETABLES: &[&str] = &["🥕", "🥦", "🥬", "🥒", "🌽", "🥔"];
const MEAT: &[&str] = &["🥩", "🍗", "🍖", "🥓", "🍔", "🍕"];
const DAIRY: &[&str] = &["🥛", "🧀", "🍦", "🍨", "🍩", "🍪"];
const FOOD: &[&str] = &[
    "🍎", "🍌", "🍊", "🍉", "🍇", "🍓", "🥕", "🥦", "🥬", "🥒", "🌽", "🥔", "🥩", "🍗", "🍖", "🥓",
    "🍔", "🍕", "🥛", "🧀", "🍦", "🍨", "🍩", "🍪",
];

fn ninetynine_bottles() -> String {
    let mut bottles = String::new();
    for i in (95..100).rev() {
        bottles.push_str(&format!(
            "{0} bottles of beer on the wall, {0} bottles of beer!\n",
            i
        ));
        bottles.push_str(&format!(
            "Take one down, pass it around, {0} bottles of beer on the wall!\n",
            i - 1
        ));
    }
    bottles += "That's quite enough to demonstrate this function!";
    bottles
}

#[poise::command(
    slash_command,
    prefix_command,
    category = "Vegan",
    help_text_fn = "ninetynine_bottles"
)]
async fn beer(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍺").await?;
    Ok(())
}

/// Respond with a random fruit
///
/// Subcommands can be used to get a specific fruit
#[poise::command(
    slash_command,
    prefix_command,
    subcommands(
        "apple",
        "banana",
        "orange",
        "watermelon",
        "grape",
        "strawberry",
        "help"
    ),
    category = "Vegan"
)]
async fn fruit(ctx: Context<'_>) -> Result<(), Error> {
    let response = FRUIT[rand::thread_rng().gen_range(0..FRUIT.len())];
    ctx.say(response).await?;
    Ok(())
}

/// Respond with an apple
#[poise::command(slash_command, prefix_command, subcommands("red", "green"))]
async fn apple(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍎").await?;
    Ok(())
}

/// Respond with a red apple
#[poise::command(slash_command, prefix_command)]
async fn red(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍎").await?;
    Ok(())
}

/// Respond with a green apple
#[poise::command(slash_command, prefix_command)]
async fn green(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍏").await?;
    Ok(())
}

/// Respond with a banana
#[poise::command(slash_command, prefix_command)]
async fn banana(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍌").await?;
    Ok(())
}

/// Respond with an orange
#[poise::command(slash_command, prefix_command)]
async fn orange(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍊").await?;
    Ok(())
}

/// Respond with a watermelon
#[poise::command(slash_command, prefix_command)]
async fn watermelon(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍉").await?;
    Ok(())
}

/// Respond with a grape
#[poise::command(slash_command, prefix_command)]
async fn grape(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍇").await?;
    Ok(())
}

/// Respond with a strawberry
#[poise::command(slash_command, prefix_command)]
async fn strawberry(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍓").await?;
    Ok(())
}

/// Respond with a random vegetable
#[poise::command(slash_command, prefix_command, category = "Vegan")]
async fn vegetable(ctx: Context<'_>) -> Result<(), Error> {
    let response = VEGETABLES[rand::thread_rng().gen_range(0..VEGETABLES.len())];
    ctx.say(response).await?;
    Ok(())
}

/// Respond with a random meat
#[poise::command(slash_command, prefix_command, category = "Other")]
async fn meat(ctx: Context<'_>) -> Result<(), Error> {
    let response = MEAT[rand::thread_rng().gen_range(0..MEAT.len())];
    ctx.say(response).await?;
    Ok(())
}

/// Respond with a random dairy product
#[poise::command(slash_command, prefix_command, category = "Other")]
async fn dairy(ctx: Context<'_>) -> Result<(), Error> {
    let response = DAIRY[rand::thread_rng().gen_range(0..DAIRY.len())];
    ctx.say(response).await?;
    Ok(())
}

/// Give a user some random food
#[poise::command(context_menu_command = "Give food")]
async fn context_food(
    ctx: Context<'_>,
    #[description = "User to give food to"] user: serenity::User,
) -> Result<(), Error> {
    let response = format!(
        "<@{}>: {}",
        user.id,
        FOOD[rand::thread_rng().gen_range(0..FOOD.len())]
    );

    ctx.say(response).await?;
    Ok(())
}

/// Give a user some random fruit
#[poise::command(
    slash_command,
    context_menu_command = "Give fruit",
    category = "Context menu but also slash/prefix"
)]
async fn context_fruit(
    ctx: Context<'_>,
    #[description = "User to give fruit to"] user: serenity::User,
) -> Result<(), Error> {
    let response = format!(
        "<@{}>: {}",
        user.id,
        FRUIT[rand::thread_rng().gen_range(0..FRUIT.len())]
    );

    ctx.say(response).await?;
    Ok(())
}

/// Give a user some random vegetable
#[poise::command(
    prefix_command,
    context_menu_command = "Give vegetable",
    category = "Context menu but also slash/prefix"
)]
async fn context_vegetable(
    ctx: Context<'_>,
    #[description = "User to give vegetable to"] user: serenity::User,
) -> Result<(), Error> {
    let response = format!(
        "<@{}>: {}",
        user.id,
        VEGETABLES[rand::thread_rng().gen_range(0..VEGETABLES.len())]
    );

    ctx.say(response).await?;
    Ok(())
}

/// Give a user some random meat
#[poise::command(
    prefix_command,
    slash_command,
    context_menu_command = "Give meat",
    category = "Context menu but also slash/prefix"
)]
async fn context_meat(
    ctx: Context<'_>,
    #[description = "User to give meat to"] user: serenity::User,
) -> Result<(), Error> {
    let response = format!(
        "<@{}>: {}",
        user.id,
        MEAT[rand::thread_rng().gen_range(0..MEAT.len())]
    );

    ctx.say(response).await?;
    Ok(())
}

/// React to a message with random food
// This command intentionally doesn't have a slash/prefix command, and its own
// category, so that we can test whether the category shows up in the help
// message. It shouldn't.
#[poise::command(
    context_menu_command = "React with food",
    ephemeral,
    category = "No slash/prefix",
    subcommands("fruit_react", "vegetable_react")
)]
async fn food_react(
    ctx: Context<'_>,
    #[description = "Message to react to (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    let reaction = FOOD[rand::thread_rng().gen_range(0..FOOD.len())];
    let reaction = serenity::ReactionType::Unicode(FixedString::from_str_trunc(reaction));

    msg.react(ctx.http(), reaction).await?;
    ctx.say("Reacted!").await?;
    Ok(())
}

// These next two commands are subcommands of `food_react`, so they're not
// visible in the overview help command. But they should still show up in
// `?help react with food`

/// React to a message with a random fruit
#[poise::command(
    slash_command,
    context_menu_command = "React with fruit",
    ephemeral,
    category = "No slash/prefix"
)]
async fn fruit_react(
    ctx: Context<'_>,
    #[description = "Message to react to (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    let reaction = FRUIT[rand::thread_rng().gen_range(0..FRUIT.len())];
    let reaction = serenity::ReactionType::Unicode(FixedString::from_str_trunc(reaction));

    msg.react(ctx.http(), reaction).await?;
    ctx.say("Reacted!").await?;
    Ok(())
}

/// React to a message with a random vegetable
#[poise::command(
    slash_command,
    context_menu_command = "React with vegetable",
    ephemeral,
    category = "No slash/prefix"
)]
async fn vegetable_react(
    ctx: Context<'_>,
    #[description = "Message to react to (enter a link or ID)"] msg: serenity::Message,
) -> Result<(), Error> {
    let reaction = VEGETABLES[rand::thread_rng().gen_range(0..VEGETABLES.len())];
    let reaction = serenity::ReactionType::Unicode(FixedString::from_str_trunc(reaction));

    msg.react(ctx.http(), reaction).await?;
    ctx.say("Reacted!").await?;
    Ok(())
}

/// Show help message
#[poise::command(prefix_command, track_edits, category = "Utility")]
async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"]
    #[rest]
    mut command: Option<String>,
) -> Result<(), Error> {
    // This makes it possible to just make `help` a subcommand of any command
    // `/fruit help` turns into `/help fruit`
    // `/fruit help apple` turns into `/help fruit apple`
    if ctx.invoked_command_name() != "help" {
        command = match command {
            Some(c) => Some(format!("{} {}", ctx.invoked_command_name(), c)),
            None => Some(ctx.invoked_command_name().to_string()),
        };
    }
    let extra_text_at_bottom = "\
Type `?help command` for more info on a command.
You can edit your `?help` message to the bot and the bot will edit its response.";

    let config = HelpConfiguration {
        show_subcommands: true,
        show_context_menu_commands: true,
        ephemeral: true,
        extra_text_at_bottom,

        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
async fn register_commands(ctx: Context<'_>) -> Result<(), Error> {
    let commands = &ctx.framework().options().commands;
    poise::builtins::register_globally(ctx.http(), commands).await?;

    ctx.say("Successfully registered slash commands!").await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let options = poise::FrameworkOptions {
        commands: vec![
            register_commands(),
            fruit(),
            vegetable(),
            beer(),
            meat(),
            dairy(),
            help(),
            context_food(),
            context_fruit(),
            context_vegetable(),
            context_meat(),
            food_react(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("?".into()),
            ..Default::default()
        },
        ..Default::default()
    };

    let client = serenity::ClientBuilder::new(&token, intents)
        .framework(poise::Framework::new(options, true))
        .await;

    client.unwrap().start().await.unwrap();
}
