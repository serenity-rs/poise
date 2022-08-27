#![cfg_attr(doc_nightly, feature(doc_cfg, doc_auto_cfg))]
#![doc(test(attr(deny(deprecated))))]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::type_complexity)]
// native #[non_exhaustive] is awful because you can't do struct update syntax with it (??)
#![allow(clippy::manual_non_exhaustive)]
#![allow(deprecated)]

/*!
Poise is an opinionated Discord bot framework with a few distinctive features:
- edit tracking: when user edits their message, automatically update bot response
- slash commands: completely define both normal and slash commands with a single function
- flexible argument parsing: command parameters are defined with normal Rust types and parsed automatically

# Quickstart
```rust,no_run
*/
// Nested cfg_attr is needed for some reason
#![cfg_attr(doc_nightly, cfg_attr(doc_nightly, doc = include_str!("../examples/quickstart/main.rs")))]
#![cfg_attr(not(doc_nightly), doc = "// See ../examples/quickstart/main.rs")]
/*!
```

To run commands, ping your bot and write the command name and arguments after. Run the register
command to register slash commands, after which you can use those, too.

See examples/framework_usage/ in the git repository for a full-featured example bot, showcasing most
features of poise: `cargo run --example=framework_usage`

# Introduction to serenity

Serenity is the Discord API wrapper library poise is built on top of. Using poise automatically
means using serenity, so here's a couple tips:

## `impl Trait` parameters

Many serenity functions take an argument of type [`impl CacheHttp`](serenity::CacheHttp) or
[`impl AsRef<Http>`](serenity::Http). Here, you commonly pass in
[`&serenity::Context`](serenity::Context), which you can get from
[`poise::Context`](crate::Context) via [`ctx.discord()`](crate::Context::discord)

## Gateway intents

To run a Discord bot, you need to set _gateway intents_: a list of event types you want to receive
from Discord. A sensible default is [`serenity::GatewayIntents::non_privileged()`] which contains
all event types except privileged ones. Privileged intents require manual enabling in your bot
dashboard to use (and large bots require whitelisting by Discord). A notable privileged intent
is [MESSAGE_CONTENT](serenity::GatewayIntents::MESSAGE_CONTENT) which is required for poise prefix
commands.

To set multiple gateway events, use the OR operator:
`serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT`

## Discord actions outside a command

You can run Discord actions outside of commands by cloning and storing [`serenity::CacheAndHttp`]/
[`Arc<serenity::Http>`](serenity::Http)/[`Arc<serenity::Cache>`](serenity::Cache). You can get
those either from [`serenity::Context`] (passed to
[`user_data_setup`](crate::FrameworkBuilder::user_data_setup) and all commands via
[`ctx.discord()`](crate::Context::discord)) or before starting the framework via
[`framework.client()`](crate::Framework::client)[`.cache_and_http`](serenity::Client::cache_and_http).

Pass your `CacheAndHttp` or `Arc<Http>` to serenity functions in place of the usual
`serenity::Context`

## Finding serenity methods

Many serenity structs have an ID field. Some useful methods are defined only on the Id types.
For example:
- [`serenity::Guild`] and [`serenity::GuildId`]
- [`serenity::User`] and [`serenity::UserId`]
- [`serenity::Role`] and [`serenity::RoleId`]
- ...

# Introduction to slash commands

Discord slash commands can be a bit unintuitive at first. If you're unfamiliar, please read this

To activate a slash command, your bot
needs to _register_ in on Discord. You may want to do this manually, with a `register` command
(poise provides [`builtins::register_application_commands_buttons`] as a starting point for that), or you
may want to re-register commands automatically on every bot startup. Choose what you prefer

Commands can be registered _globally_ or _per guild_. Global commands are available on every guild
your bot is invited on, but it takes up to an hour for global registration to roll out. Per guild
registration only updates a single guild, but it happens instantly, which is useful for testing.

Your bot also needs to be invited with the `applications.commands` scope. For example, in Discord's
invite link generator (discord.com/developers/applications/XXX/oauth2/url-generator),
tick the `applications.commands` box.

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
    help_text_fn = "my_huge_ass_command_help",
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

# async {
// Use `Framework::builder()` to create a framework builder and supply basic data to the framework:
poise::Framework::builder()
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
            // You can also modify a command by changing the fields of its Command instance
            poise::Command {
                // [override fields here]
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

Also, poise is a stat in Dark Souls
*/

mod prefix_argument;
pub use prefix_argument::*;

mod slash_argument;
pub use slash_argument::*;

mod event;
pub use event::{Event, EventWrapper};

mod structs;
pub use structs::*;

mod dispatch;
pub use dispatch::*;

mod framework;
pub use framework::*;

mod reply;
pub use reply::*;

mod cooldown;
pub use cooldown::*;

mod modal;
pub use modal::*;

mod track_edits;
pub use track_edits::*;

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

/// This module re-exports a bunch of items from all over serenity. Useful if you can't
/// remember the full paths of serenity items.
///
/// One way to use this prelude module in your project is
/// ```rust
/// use poise::serenity_prelude as serenity;
/// ```
pub mod serenity_prelude {
    #[doc(no_inline)]
    pub use serenity::{
        async_trait,
        builder::*,
        client::{
            bridge::gateway::{event::*, *},
            *,
        },
        collector::*,
        http::*,
        // Explicit imports to resolve ambiguity between model::prelude::* and
        // model::application::interaction::* due to deprecated same-named type aliases
        model::{
            application::interaction::{
                Interaction, InteractionResponseType, InteractionType,
                MessageFlags as InteractionResponseFlags, MessageInteraction,
            },
            // There's two MessageFlags in serenity. The interaction response specific one was
            // renamed to InteractionResponseFlags above so we can keep this one's name the same
            channel::MessageFlags,
        },
        model::{
            application::{
                command::*,
                component::*,
                interaction::{
                    application_command::*, autocomplete::*, message_component::*, modal::*, *,
                },
            },
            event::*,
            prelude::*,
        },
        prelude::*,
        utils::*,
        *,
    };
}
use serenity_prelude as serenity; // private alias for crate docs intradoc-links

use std::future::Future;
use std::pin::Pin;

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
