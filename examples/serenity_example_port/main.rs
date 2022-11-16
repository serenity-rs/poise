//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```
use poise::serenity_prelude as serenity;
use std::fmt::Write as _;

/// A shared instance of this struct is available across all events and framework commands
struct Data {
    command_counter: std::sync::Mutex<std::collections::HashMap<String, u64>>,
}
/// This Error type is used throughout all commands and callbacks
type Error = Box<dyn std::error::Error + Send + Sync>;

/// This type alias will save us some typing, because the Context type is needed often
type Context<'a> = poise::Context<'a, Data, Error>;

async fn event_event_handler(
    _ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            println!("{} is connected!", data_about_bot.user.name)
        }
        _ => {}
    }

    Ok(())
}

// INFO: poise doesn't yet support sophisticated groups like this
/*
// Sets multiple prefixes for a group.
// This requires us to call commands in this group
// via `~emoji` (or `~em`) instead of just `~`.
#[prefixes("emoji", "em")]
// Set a description to appear if a user wants to display a single group
// e.g. via help using the group-name or one of its prefixes.
#[description = "A group with commands providing an emoji as response."]
// Summary only appears when listing multiple groups.
#[summary = "Do emoji fun!"]
// Sets a command that will be executed if only a group-prefix was passed.
#[default_command(bird)]
#[commands(cat, dog)]
struct Emoji;

#[group]
// Sets a single prefix for this group.
// So one has to call commands in this group
// via `~math` instead of just `~`.
#[prefix = "math"]
#[commands(multiply)]
struct Math;

#[group]
#[owners_only]
// Limit all commands to be guild-restricted.
#[only_in(guilds)]
// Summary only appears when listing multiple groups.
#[summary = "Commands for server owners"]
#[commands(slow_mode)]
struct Owner;
*/

// INFO: this level of customization is currently not supported by poise's built-in help feature
/*
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_in_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
*/
// The framework provides built-in help functionality for you to use.
// You just have to set the metadata of the command like descriptions, to fit with the rest of your
// bot. The actual help text generation is delegated to poise
/// Show a help menu
#[poise::command(prefix_command, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Command to display specific information about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~

If you want more information about a specific command, just pass the command as argument.",
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;

    Ok(())
}

/// Registers slash commands in this guild or globally
#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

async fn pre_command(ctx: Context<'_>) {
    println!(
        "Got command '{}' by user '{}'",
        ctx.command().name,
        ctx.author().name
    );

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut command_counter = ctx.data().command_counter.lock().unwrap();
    let entry = command_counter
        .entry(ctx.command().name.to_string())
        .or_insert(0);
    *entry += 1;
}

async fn post_command(ctx: Context<'_>) {
    println!("Processed command '{}'", ctx.command().name);
}

// TODO: unify the command checks in poise::FrameworkOptions and then implement a command check here
// with this in it:
// ```
// true // if `check` returns false, command processing doesn't happen.
// ```

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx } => {
            println!(
                "Command '{}' returned error {:?}",
                ctx.command().name,
                error
            );
        }
        poise::FrameworkError::EventHandler { error, event, .. } => {
            println!(
                "EventHandler returned error during {:?} event: {:?}",
                event.name(),
                error
            );
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

// INFO: Poise currently does not support callbacks for these events
/*#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: Context<'_>) {
    println!("Message is not a command '{}'", msg.content);
}*/

// INFO: Currently not applicable because poise doesn't have cooldowns
/*#[hook]
async fn delay_action(ctx: Context<'_>) {
    // You may want to handle a Discord rate limit if this fails.
    let _ = msg.react(ctx, '⏱').await;
}

#[hook]
async fn dispatch_error(ctx: Context<'_>, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        // We notify them only once.
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                .await;
        }
    }
}

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
use serenity::{futures::future::BoxFuture, FutureExt};
fn _dispatch_error_no_macro<'fut>(
    ctx: &'fut mut Context,
    msg: &'fut Message,
    error: DispatchError,
) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(info) = error {
            if info.is_first_try {
                let _ = msg
                    .channel_id
                    .say(&ctx.http, &format!("Try this again in {} seconds.", info.as_secs()))
                    .await;
            }
        };
    }
    .boxed()
}*/

#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        commands: vec![
            // The `#[poise::command(prefix_command, slash_command)]` macro transforms the function into
            // `fn() -> poise::Command`.
            // Therefore, you need to call the command function without any arguments to get the
            // command definition instance to pass to the framework
            help(),
            // This function registers slash commands on Discord. When you change something about a
            // command signature, for example by changing its name, adding or removing parameters, or
            // changing a parameter type, you should call this function.
            register(),
            about(),
            am_i_admin(),
            say(),
            commands(),
            ping(),
            latency(),
            some_long_command(),
            upper_command(),
            bird(),
            cat(),
            dog(),
            multiply(),
            slow_mode(),
        ],
        event_handler: |ctx, event, framework, user_data| {
            Box::pin(event_event_handler(ctx, event, framework, user_data))
        },
        on_error: |error| Box::pin(on_error(error)),
        // Set a function to be called prior to each command execution. This
        // provides all context of the command that would also be passed to the actual command code
        pre_command: |ctx| Box::pin(pre_command(ctx)),
        // Similar to `pre_command`, except will be called directly _after_
        // command execution.
        post_command: |ctx| Box::pin(post_command(ctx)),

        // Options specific to prefix commands, i.e. commands invoked via chat messages
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some(String::from("~")),

            mention_as_prefix: false,
            // An edit tracker needs to be supplied here to make edit tracking in commands work
            edit_tracker: Some(poise::EditTracker::for_timespan(
                std::time::Duration::from_secs(3600 * 3),
            )),
            ..Default::default()
        },

        ..Default::default()
    };

    // The Framework builder will automatically retrieve the bot owner and application ID via the
    // passed token, so that information need not be passed here
    poise::Framework::builder()
        // Configure the client with your Discord bot token in the environment.
        .token(std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in the environment"))
        .options(options)
        .setup(|_ctx, _data_about_bot, _framework| {
            Box::pin(async move {
                Ok(Data {
                    command_counter: std::sync::Mutex::new(std::collections::HashMap::new()),
                })
            })
        })
        .run()
        .await
        .expect("Client error");

    // INFO: currently not supported by poise
    /*
    // Set a function that's called whenever an attempted command-call's
    // command could not be found.
    .unrecognised_command(unknown_command)
    // Set a function that's called whenever a message is not a command.
    .normal_message(normal_message)
    // Set a function that's called whenever a command's execution didn't complete for one
    // reason or another. For example, when a user has exceeded a rate-limit or a command
    // can only be performed by the bot owner.
    .on_dispatch_error(dispatch_error)
    // Can't be used more than once per 5 seconds:
    .bucket("emoji", |b| b.delay(5)).await
    // Can't be used more than 2 times per 30 seconds, with a 5 second delay applying per channel.
    // Optionally `await_ratelimits` will delay until the command can be executed instead of
    // cancelling the command invocation.
    .bucket("complicated", |b| b.limit(2).time_span(30).delay(5)
        // The target each bucket will apply to.
        .limit_for(LimitedFor::Channel)
        // The maximum amount of command invocations that can be delayed per target.
        // Setting this to 0 (default) will never await/delay commands and cancel the invocation.
        .await_ratelimits(1)
        // A function to call when a rate limit leads to a delay.
        .delay_action(delay_action)
    ).await
    */
}

// Commands can be created via the attribute `#[poise::command()]` macro.
// Options are passed as arguments to the macro.
#[poise::command(prefix_command, slash_command, category = "General")]
// INFO: not supported
/*
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
*/
/// Shows how often each command was used
async fn commands(ctx: Context<'_>) -> Result<(), Error> {
    let mut contents = "Commands used:\n".to_string();

    for (k, v) in &*ctx.data().command_counter.lock().unwrap() {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    ctx.say(contents).await?;

    Ok(())
}

/// Repeats what the user passed as argument safely
///
/// Ensures that user and role mentions are replaced with a safe textual alternative.
// In this example channel mentions are excluded via the `ContentSafeOptions`.
// The track_edits argument ensures that when the user edits their command invocation,
// the bot updates the response message accordingly.
#[poise::command(prefix_command, slash_command, track_edits, category = "General")]
async fn say(
    ctx: Context<'_>,
    #[description = "Text to repeat"]
    #[rest]
    content: String,
) -> Result<(), Error> {
    let settings = if let Some(guild_id) = ctx.guild_id() {
        // By default roles, users, and channel mentions are cleaned.
        serenity::ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        serenity::ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = serenity::content_safe(ctx, &content, &settings, {
        // If we are in a prefix command, we pass the Users that were mentioned in the message
        // to avoid them needing to be fetched from cache
        if let poise::Context::Prefix(ctx) = ctx {
            &ctx.msg.mentions
        } else {
            &[]
        }
    });

    ctx.say(content).await?;

    Ok(())
}

// A function which acts as a "check", to determine whether to call a command.
//
// In this case, this command checks to ensure you are the owner of the message
// in order for the command to be executed. If the check fails, the command is
// not called.
//
// Note: to allow command execution only for the owner, the owners_only #[command] macro argument
// is a much better method than this. This check is used just as an example.
async fn owner_check(ctx: Context<'_>) -> Result<bool, Error> {
    // Replace 7 with your ID to make this check pass.
    if ctx.author().id != 7 {
        return Ok(false);
    }

    Ok(true)
}

/// This is a command with a deliberately long name
#[poise::command(prefix_command, slash_command, category = "General")]
async fn some_long_command(
    ctx: Context<'_>,
    #[description = "Arguments to this command"]
    #[rest]
    args: String,
) -> Result<(), Error> {
    ctx.say(format!("Arguments: {:?}", args)).await?;

    Ok(())
}

/// Retrieves the role ID of a role
#[poise::command(prefix_command, slash_command)]
// INFO: not implemented
/*
// Limits the usage of this command to roles named:
#[allowed_roles("mods", "ultimate neko")]
*/
async fn about_role(
    ctx: Context<'_>,
    #[description = "Name of the role"] potential_role_name: String,
) -> Result<(), Error> {
    if let Some(guild) = ctx.guild() {
        // `role_by_name()` allows us to attempt attaining a reference to a role
        // via its name.
        if let Some(role) = guild.role_by_name(&potential_role_name) {
            if let Err(why) = ctx.say(format!("Role-ID: {}", role.id)).await {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    poise::say_reply(
        ctx,
        format!("Could not find role named: {:?}", potential_role_name),
    )
    .await?;

    Ok(())
}

/// Multiplies two numbers
#[poise::command(
    prefix_command,
    slash_command,
    // Lets us also call `~math *` instead of just `~math multiply`.
    aliases("*"),
    category = "Math",
)]
async fn multiply(
    ctx: Context<'_>,
    #[description = "First number"] first: f64,
    #[description = "Second number"] second: f64,
) -> Result<(), Error> {
    let res = first * second;

    ctx.say(res.to_string()).await?;

    Ok(())
}

/// Shows information about this bot
#[poise::command(prefix_command, slash_command, category = "General")]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This is a small test-bot! : )").await?;

    Ok(())
}

/// Shows current latency of this bot
#[poise::command(prefix_command, slash_command, category = "General")]
async fn latency(ctx: Context<'_>) -> Result<(), Error> {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let shard_manager = ctx.framework().shard_manager();

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = runners
        .get(&serenity::ShardId(ctx.shard_id))
        .ok_or("No shard found")?;

    ctx.say(format!("The shard latency is {:?}", runner.latency))
        .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    check = "owner_check",
    category = "General"
)]
// INFO: not implemented
/*
// Limit command usage to guilds.
#[only_in(guilds)]
*/
/// Ping pong
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong! : )").await?;

    Ok(())
}

/// Sends an emoji with a cat.
#[poise::command(
    prefix_command,
    slash_command,
    // Adds multiple aliases
    aliases("kitty", "neko"),
    // Allow only administrators to call this:
    required_permissions = "ADMINISTRATOR",
    category = "Emoji"
)]
// INFO: not implemented
/*
// Make this command use the "emoji" bucket.
#[bucket = "emoji"]
*/
async fn cat(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(":cat:").await?;

    // INFO: buckets not implemented
    /*
    // We can return one ticket to the bucket undoing the ratelimit.
    Err(RevertBucket.into())
    */

    Ok(())
}

/// Sends an emoji with a dog.
#[poise::command(prefix_command, slash_command, category = "Emoji")]
// INFO: not implemented
/*
#[bucket = "emoji"]
*/
async fn dog(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(":dog:").await?;

    Ok(())
}

/// Sends an emoji with a bird.
#[poise::command(prefix_command, slash_command, category = "Emoji")]
async fn bird(
    ctx: Context<'_>,
    #[description = "Name of the bird you're searching for"] bird_name: Option<String>,
) -> Result<(), Error> {
    let say_content = match bird_name {
        None => ":bird: can find animals for you.".to_string(),
        Some(bird_name) => format!(":bird: could not find animal named: `{}`.", bird_name),
    };

    ctx.say(say_content).await?;

    Ok(())
}

// We could also use `required_permissions = "ADMINISTRATOR"`
// but that would not let us reply when it fails.
/// Tells you whether you are an admin on the server
#[poise::command(prefix_command, slash_command, category = "General")]
async fn am_i_admin(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        for role in guild_id.member(ctx, ctx.author().id).await?.roles {
            if role.to_role_cached(ctx).map_or(false, |r| {
                r.has_permission(serenity::Permissions::ADMINISTRATOR)
            }) {
                ctx.say("Yes, you are.").await?;

                return Ok(());
            }
        }
    }

    ctx.say("No, you are not.").await?;

    Ok(())
}

/// Enable slowmode for a channel. Pass no argument to disable slowmode
#[poise::command(
    prefix_command,
    slash_command,
    check = "owner_check",
    category = "Owner"
)]
async fn slow_mode(
    ctx: Context<'_>,
    #[description = "Minimum time between sending messages per user"] rate_limit: Option<u64>,
) -> Result<(), Error> {
    let say_content = if let Some(rate_limit) = rate_limit {
        if let Err(why) = ctx
            .channel_id()
            .edit(ctx, |c| c.rate_limit_per_user(rate_limit))
            .await
        {
            println!("Error setting channel's slow mode rate: {:?}", why);
            format!("Failed to set slow mode to `{}` seconds.", rate_limit)
        } else {
            format!(
                "Successfully set slow mode rate to `{}` seconds.",
                rate_limit
            )
        }
    } else if let Some(serenity::Channel::Guild(channel)) = ctx.channel_id().to_channel_cached(ctx)
    {
        format!(
            "Current slow mode rate is `{}` seconds.",
            channel.rate_limit_per_user.unwrap_or(0)
        )
    } else {
        "Failed to find channel in cache.".to_string()
    };

    ctx.say(say_content).await?;

    Ok(())
}

/// Dummy command to test subcommands
#[poise::command(
    prefix_command,
    slash_command,
    rename = "upper",
    category = "General",
    // A command can have sub-commands, just like in command lines tools.
    // Imagine `cargo help` and `cargo help run`.
    subcommands("sub")
)]
async fn upper_command(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This is the main function!").await?;

    Ok(())
}

/// "This is `upper`'s sub-command.
#[poise::command(prefix_command, slash_command, aliases("sub-command", "secret"))]
// This will only be called if preceded by the `upper`-command.
async fn sub(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This is a sub function!").await?;

    Ok(())
}
