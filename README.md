[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://kangalioo.github.io/poise/poise/)
[![Docs](https://img.shields.io/badge/docs-online-informational)](https://kangalioo.github.io/poise/poise/)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust: 1.51+](https://img.shields.io/badge/rust-1.51+-93450a)](https://blog.rust-lang.org/2020/11/19/Rust-1.51.html)

# Poise
Poise is an opinionated Discord bot framework with a few distinctive features:
- edit tracking: when user edits their message, automatically update bot response 
- slash commands: completely define both normal and slash commands with a single function
- flexible argument parsing: command parameters are defined with normal Rust types and parsed automatically

I created this framework mainly for personal use ([rustbot](<https://github.com/kangalioo/rustbot>)
and [etternabot](<https://github.com/kangalioo/etternabot>)). Features are added on demand, since
it's easy to draft a good design when you know exactly what the feature will be used for.

**Warning: API details are subject to change**

# Quickstart
```rust,no_run
use poise::serenity_prelude as serenity;

type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Display your or another user's account creation date
#[poise::command(prefix_command, slash_command, track_edits)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or(ctx.author());
    poise::say_reply(
        ctx,
        format!("{}'s account was created at {}", user.name, user.created_at()),
    ).await?;
    
    Ok(())
}

#[tokio::main]
async fn main() {
    poise::Framework::build()
        .prefix("~")
        .token(std::env::var("DISCORD_BOT_TOKEN").unwrap())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(()) }),
        .options(poise::FrameworkOptions {
            // configure framework here
            ..Default::default()
        })
        .command(age(), |f| f)
        .run().await.unwrap();
}
```

A full functioning bot would contain a help command as well as a register command to register slash
commands. See [`examples/framework_usage`] for examples on that as well as other features of poise.

You can run the framework_usage example with
`cargo run --example=framework_usage --features collector`

# How to use

## Create commands
Every command is represented by a function:

```rust,ignore
/// Description of the command here
/// 
/// Here you can explain how the command \
/// is used and how it works.
#[poise::command(prefix_command, /* add more optional command settings here, like slash_command */)]
async fn command_name(
    ctx: Context<'_>,
    #[description = "Description of arg1 here"] arg1: serenity::Member,
    #[description = "Description of arg2 here"] arg2: Option<u32>,
) -> Result<(), Error> {
    // Command code here

    Ok(())
}
```

There are several things to note here:
- Documentation comments are used as help text. The first line is a single-line description,
    displayed in listings of your bot's commands (i.e. `~help`). Following paragraphs are detailed explanations,
    for example for command-specific help (i.e. `~help command_name`).

    Escape newlines with `\`
- `#[poise::command]` accepts a number of arguments to configure the command:
    - `prefix_command`: Generate a prefix command
    - `slash_command`: Generate a slash command
    - `context_menu_command`: Generate a context menu command
    - `aliases`: Command name aliases (only applies to prefix commands)
    - `track_edits`: Enable edit tracking (only applies to prefix commands)
    - `broadcast_typing`: Trigger a typing indicator while command runs (only applies to prefix commands I think)
    - `defer_response`: Immediately acknowledge incoming slash command invocation which shows a loading state to the user and gives the bot several minutes to respond
    - `explanation_fn`: Path to a string-returning function which is used for the detailed explanations instead of documentation comments
        - Useful if you have many commands with very similar help messages: you can abstract the common parts into a function
    - `check`: Path to a function which is invoked for every invocation. If the function returns false, the command is not executed
    - `on_error`: Error handling function
    - `rename`: Choose an alternative command name instead of the function name
        - Useful if your command name is a Rust keyword, like `move`
    - `discard_spare_arguments`: Don't throw an error if the user supplies too many arguments
    - `hide_in_help`: Hide this command in help menus
    - `ephemeral`: Make bot responses ephemeral if possible
        - Only poise's function, like `poise::send_reply`, respect this preference
    - `required_permissions`: Permissions which the command caller needs to have
    - `owners_only`: Restricts command callers to the list of owners specified in framework options
- `Context` is the first parameter of all command functions. It's an enum over either PrefixContext or SlashContext, which contain a variety of context data each. Context provides some utility methods to access data present in both PrefixContext and SlashContext, like `author()` or `created_at()`
- All following parameters are inputs to the command. You can use all types that implement `poise::PopArgumentAsync`, `poise::PopArgument`, `serenity::ArgumentConvert` or `std::str::FromStr`. You can also wrap types in `Option` or `Vec` to make them optional or variadic. In addition, there are multiple attributes you can use on parameters:
    - `#[description]`: Required for slash commands; a description of the parameter
    - `#[rest]`: Relevant for prefix commands; means that the entire rest of the message is parsed as the parameter even without quotes
    - `#[lazy]`: Relevant for prefix commands; can be used on Option and Vec parameters and is equivalent to regular expressions' laziness
    - `#[flag]`: Relevant for prefix commands; can be used on a bool parameter to set the bool to true if the user typed the parameter name literally
        - For example with `async fn my_command(ctx: Context<'_>, #[flag] my_flag: bool)`, `~my_command` would set my_flag to false, and `~my_command my_flag` would set my_flag to true

### Big example to showcase many command features

```rust,ignore
/// A test command for poise
#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    hide_in_help,
    required_permissions = "serenity::Permissions::SEND_MESSAGES",
    aliases("bigounce", "abomination"),
    explanation_fn = "my_huge_ass_command_help",
    check = "check",
    on_error = "error_handler",
)]
async fn my_huge_ass_command(
    ctx: Context<'_>,
    #[description = "Consectetur"] ip_addr: std::net::IpAddr, // implements FromStr
    #[description = "Amet"] user: serenity::Member, // implements ArgumentConvert
    #[description = "Sit"] code_block: poise::CodeBlock, // implements PopArgument
    #[description = "Dolor"] #[flag] my_flag: bool,
    #[description = "Ipsum"] #[lazy] always_none: Option<String>,
    #[description = "Lorem"] #[rest] rest: String,
) -> Result<(), Error> {
    Ok(())
}

fn my_huge_ass_command_help() -> String {
    String::from("\
Example usage:
~my_huge_ass_command 127.0.0.1 @kangalioo `i = i + 1` my_flag rest of the message")
}

async fn check(ctx: Context<'_>) -> Result<bool, Error> {
    // We discriminate against users starting with an X
    Ok(!ctx.author().name.starts_with('X'))
}

async fn error_handler(error: Error, _ctx: poise::ErrorContext) {
    println!("Oh noes, we got an error: {}", error);
}
```

## Create and configure framework

Use `Framework::build()` to create a framework builder and supply basic data to the framework:

```rust,ignore
poise::Framework::build()
    .prefix("~")
    .token("...")
    .user_data_setup(|_, _, _| Box::pin(async move {
        // construct user data here (invoked when bot connects to Discord)
        Ok(())
    }))
```

A lot of configuration is done via the `FrameworkOptions` struct, which you can define with a struct
literal (hint: use `..Default::default()` to fill uninitialized
settings with their default value):

```rust,ignore
.options(poise::FrameworkOptions {
    on_error: Some(|err, ctx| Box::pin(my_error_function(err, ctx))),
    prefix_options: poise::PrefixFrameworkOptions {
        edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600)))
        case_insensitive_commands: true,
        ..Default::default()
    },
    ..Default::default()
})
```

Finally, add commands and start the framework. You can add subcommands or assign a category to
a command. Commands with the same category are grouped together in the help menu.

```rust,ignore
.command(command1(), |f| f)
.command(command2(), |f| f)
.command(command3(), |f| f.category("My cool category"))
.command(command4(), |f| f.category("My cool category"))
.command(command5(), |f| f
    .category("This category has one command with subcommands")
    .subcommand(command5_1(), |f| f)
    .subcommand(command5_2(), |f| f)
)
.run().await?
```

# Tips and tricks

## Type aliases
As seen in the examples, it's useful to define type aliases for `Context` with
your bot's error type and user data type filled in:
```rust,ignore
type Context<'a> = poise::Context<'a, UserData, ErrorType>;
```

## Serenity prelude
When you're too lazy to import serenity items from their full path which can be quite lengthy at
times, you can use `poise::serenity_prelude`: a module which reexports almost all items from
serenity.

```rust,ignore
use poise::serenity_prelude as serenity;

// Short paths!
serenity::Member, serenity::UserId, serenity::ReactionType, serenity::GatewayIntents
```

# Limitations

- volatile state with breaking API changes to come
- only partial command group support
- many miscellaneous features missing, for example command cooldowns

# About the weird name
I'm bad at names. Google lists "poise" as a synonym to "serenity" which is the Discord library
underlying this framework, so that's what I chose.
