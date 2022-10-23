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

## Useful serenity methods

Many serenity structs have an ID field. Some useful methods are defined only on the Id types.
For example:
- [`serenity::Guild`] and [`serenity::GuildId`]
- [`serenity::User`] and [`serenity::UserId`]
- [`serenity::Role`] and [`serenity::RoleId`]
- ...

Some examples for methods you should have in your repertoire:
- [`serenity::GuildId::to_guild_cached`] and [`serenity::GuildId::to_partial_guild`] to access full
    guild data
- [`serenity::ChannelId::to_channel`] to access full channel data
- [`serenity::Channel::guild`] to try convert a generic [`serenity::Channel`] into a
    [`serenity::GuildChannel`], to access guild specific channel data

# Introduction to slash commands

Discord slash commands can be a bit unintuitive at first. If you're unfamiliar, please read this

To activate a slash command, your bot
needs to _register_ in on Discord. You may want to do this manually, with a `register` command
(poise provides [`builtins::register_application_commands_buttons`] as a starting point for that), or you
may want to re-register commands automatically on every bot startup. Choose what you prefer. Also
see [Registering Slash Commands](#registering-slash-commands).

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

### Subcommands
Commands in poise have a tree structure. Every commands refers to a list of subcommands, which you
can easily set using the [`command`] macro like so:

```rust
# type Error = Box<dyn std::error::Error + Send + Sync>;
# type Context<'a> = poise::Context<'a, (), Error>;
#[poise::command(prefix_command, slash_command, subcommands("child1", "child2"))]
pub async fn parent(ctx: Context<'_>, arg: String) -> Result<(), Error> { Ok(()) }

#[poise::command(prefix_command, slash_command)]
pub async fn child1(ctx: Context<'_>, arg: String) -> Result<(), Error> { Ok(()) }
#[poise::command(prefix_command, slash_command)]
pub async fn child2(ctx: Context<'_>, arg: String) -> Result<(), Error> { Ok(()) }
```

With this setup, users can call `~parent [arg]` or `~parent child1 [arg]` or `~parent child2 [arg]`.
Slash command subcommands are also supported, but the base command (`/parent`) [cannot be used](https://discord.com/developers/docs/interactions/application-commands#subcommands-and-subcommand-groups)
as per Discord; only the leaf commands (`/parent child1 [arg]`, `/parent child2 [arg]`).

When adding the commands to the framework, add just the parent command (since it fully contains its
subcommands):

```rust
# type Error = Box<dyn std::error::Error + Send + Sync>;
# type Context<'a> = poise::Context<'a, (), Error>;
# #[poise::command(prefix_command)]
# pub async fn parent(ctx: Context<'_>, arg: String) -> Result<(), Error> { Ok(()) }
let options = poise::FrameworkOptions {
    commands: vec![
        parent(),
    ],
    ..Default::default()
};
```

Subcommands are stored in [`Command::subcommands`]. As with all [`Command`] fields, you can
programmatically modify those any way you'd like. The [`command`] macro is just a convenience thing
to set the fields for you.

For another example of subcommands, see `examples/framework_usage/commands/subcommands.rs`.

### Big example to showcase many command features

Also see the [`command`] macro docs

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

## Registering slash commands

As explained in [Introduction to slash commands](introduction-to-slash-commands), slash
commands need to be _registered_ to Discord. Poise provides several ways to do it, with varying
degree of abstraction. (Note: you can access a list of framework commands from anywhere with
[`ctx.framework().options.commands`](Context::framework)).

The easiest way is with [`builtins::register_application_commands_buttons`].
It spawns a message with buttons to register and unregister all commands, globally or in the current
guild (see its docs).

A more flexible approach is to serialize the commands to a `Vec<`[`serenity::CreateCommand`]`>`
using [`builtins::create_application_commands`]. That way, you can call serenity's registration
functions manually:
- [`serenity::Command::set_global_application_commands`]
- [`serenity::GuildId::set_application_commands`]

For example, you could call this function in [`FrameworkBuilder::user_data_setup`] to automatically
register commands on startup. Also see the docs of [`builtins::create_application_commands`].

The lowest level of abstraction for registering commands is [`Command::create_as_slash_command`]
and [`Command::create_as_context_menu_command`].

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

pub mod builtins;
pub mod cooldown;
pub mod dispatch;
pub mod event;
pub mod framework;
pub mod modal;
pub mod prefix_argument;
pub mod reply;
pub mod structs;
pub mod track_edits;
mod util;
pub use poise_macros as macros;

#[doc(no_inline)]
pub use {
    cooldown::*, dispatch::*, event::*, framework::*, macros::*, modal::*, prefix_argument::*,
    reply::*, structs::*, track_edits::*,
};

/// See [`builtins`]
#[deprecated = "`samples` module was renamed to `builtins`"]
pub mod samples {
    pub use crate::builtins::*;
}

#[doc(hidden)]
pub use {async_trait::async_trait, futures_util};

pub use poise_slash_argument::*;
pub use poise_utils::*;

use serenity_prelude as serenity; // private alias for crate root docs intradoc-links
