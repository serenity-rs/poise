#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(test(attr(deny(deprecated))))]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::type_complexity)]

/*!
Poise is an opinionated Discord bot framework with a few distinctive features:
- edit tracking: when user edits their message, automatically update bot response
- slash commands: completely define both normal and slash commands with a single function
- flexible argument parsing: command parameters are defined with normal Rust types and parsed automatically

I initially this framework mainly for personal use ([rustbot](<https://github.com/kangalioo/rustbot>)
and [etternabot](<https://github.com/kangalioo/etternabot>)). Features are added on demand, since
it's easier to draft a good design when you know exactly which practical needs it should cover.

**Warning: API details are subject to change**

# Quickstart
```rust,no_run
use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Display your or another user's account creation date
#[poise::command(prefix_command, slash_command, track_edits)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or(ctx.author());
    ctx.say(format!("{}'s account was created at {}", user.name, user.created_at())).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    poise::Framework::build()
        .token(std::env::var("DISCORD_BOT_TOKEN").unwrap())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move {
            Ok(Data {})
        }))
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            commands: vec![age()],
            ..Default::default()
        })
        .run().await.unwrap();
}
```

A full functioning bot would contain a help command as well as a register command to register slash
commands. See [`examples/framework_usage`] for examples on that as well as other features of poise.

You can run the framework_usage example with
`cargo run --example=framework_usage --features collector`

# Introduction to slash commands

Discord slash commands can be a bit unintuitive at first. If you're unfamiliar, please read this

To activate a slash command, your bot
needs to _register_ in on Discord. You may want to do this manually, with a `register` command
(poise provides [`builtins::register_application_commands`] as a starting point for that), or you
may want to re-register commands automatically on every bot startup. Choose what you prefer

Commands can be registered _globally_ or _per guild_. Global commands are available on every guild
your bot is invited on, but it takes up to an hour for global registration to roll out. Per guild
registration only updates a single guild, but it happens instantly, which is useful for testing.

Your bot also needs to be invited with the `applications.commands` scope. For example, in Discord's
invite link generator (discord.com/developers/applications/XXX/oauth2/url-generator),
tick the `applications.commands` box.

## Crate features

- collector: enables serenity's `collector` feature

# How to use

## Create commands
Every command is represented by a function annotated with [`#[poise::command]`](command):

```rust
# type Error = Box<dyn std::error::Error + Send + Sync>;
# type Context<'a> = poise::Context<'a, (), Error>;
# use poise::serenity_prelude as serenity;
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

See [`#[poise::command]`](command) for detailed information.

### Big example to showcase many command features

```rust
use poise::serenity_prelude as serenity;
type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// A test command for poise
#[poise::command(
    prefix_command,
    track_edits,
    hide_in_help,
    required_permissions = "SEND_MESSAGES",
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

async fn error_handler(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Oh noes, we got an error: {:?}", error);
}
```

## Create and configure framework

```rust
# type Error = Box<dyn std::error::Error + Send + Sync>;
# type Context<'a> = poise::Context<'a, (), Error>;
# async fn my_error_function(_: poise::FrameworkError<'_, (), Error>) {}
# #[poise::command(prefix_command)] async fn command1(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }
# #[poise::command(prefix_command)] async fn command2(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }
# #[poise::command(prefix_command)] async fn command3(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }
# #[poise::command(prefix_command)] async fn command3_1(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }
# #[poise::command(prefix_command)] async fn command3_2(ctx: Context<'_>) -> Result<(), Error> { Ok(()) }

# async {
// Use `Framework::build()` to create a framework builder and supply basic data to the framework:
poise::Framework::build()
    .token("...")
    .user_data_setup(|_, _, _| Box::pin(async move {
        // construct user data here (invoked when bot connects to Discord)
        Ok(())
    }))

    // Most configuration is done via the `FrameworkOptions` struct, which you can define with
    // a struct literal (hint: use `..Default::default()` to fill uninitialized
    // settings with their default value):
    .options(poise::FrameworkOptions {
        on_error: |err| Box::pin(my_error_function(err)),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600))),
            case_insensitive_commands: true,
            ..Default::default()
        },
        // This is also where commands go
        commands: vec![
            command1(),
            command2(),
            // To add subcommands, modify the `subcommands` field of the `Command` struct returned
            // by the command functions
            poise::Command {
                subcommands: vec![command3_1(), command3_2()],
                ..command3()
            }
        ],
        ..Default::default()
    })

    .run().await?;
# Ok::<(), Error>(()) };
```

# Tips and tricks

## Type aliases
As seen in the examples, it's useful to define type aliases for `Context` with
your bot's error type and user data type filled in:
```rust
# struct UserData;
# struct ErrorType;
type Context<'a> = poise::Context<'a, UserData, ErrorType>;
```

## Serenity prelude
When you're too lazy to import serenity items from their full path which can be quite lengthy at
times, you can use `poise::serenity_prelude`: a module which reexports almost all items from
serenity.

```rust
use poise::serenity_prelude as serenity;

// Short paths!
# struct _Foo(
serenity::Member, serenity::UserId, serenity::ReactionType, serenity::GatewayIntents
# );
```

# About the weird name
I'm bad at names. Google lists "poise" as a synonym to "serenity" which is the Discord library
underlying this framework, so that's what I chose.
*/

mod prefix;
pub use prefix::*;

mod slash;
pub use slash::*;

mod event;
pub use event::{Event, EventWrapper};

mod structs;
pub use structs::*;

mod framework;
pub use framework::*;

mod reply;
pub use reply::*;

mod cooldown;
pub use cooldown::*;

mod modal;
pub use modal::*;

pub(crate) mod util;

pub mod builtins;
/// See [`builtins`]
#[deprecated = "`samples` module was renamed to `builtins`"]
pub mod samples {
    pub use crate::builtins::*;
}

#[doc(no_inline)]
pub use async_trait::async_trait;
pub use futures_core;
pub use futures_util;
pub use poise_macros::*;
pub use serenity;

/// This module re-exports a bunch of items from all over serenity. Useful if you can't
/// remember the full paths of serenity items.
///
/// One way to use this prelude module in your project is
/// ```rust
/// use poise::serenity_prelude as serenity;
/// ```
pub mod serenity_prelude {
    #[cfg(feature = "collector")]
    #[doc(no_inline)]
    pub use serenity::collector::*;
    #[doc(no_inline)]
    pub use serenity::{
        async_trait,
        builder::*,
        client::{
            bridge::gateway::{event::*, *},
            *,
        },
        http::*,
        model::{
            event::*,
            interactions::{
                application_command::*, autocomplete::*, message_component::*, modal::*, *,
            },
            prelude::*,
        },
        prelude::*,
        utils::*,
        *,
    };
}

use std::future::Future;
use std::pin::Pin;

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
