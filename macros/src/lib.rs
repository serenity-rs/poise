/*!
Procedural macros used in poise, like [`macro@command`]
*/

mod choice_parameter;
mod command;
mod modal;
mod util;

use proc_macro::TokenStream;

/**
This macro transforms plain functions into poise bot commands.

Documentation comments are used as help text. The first line is a single-line description,
displayed in listings of your bot's commands (i.e. `~help`). Following paragraphs are detailed explanations,
for example for command-specific help (i.e. `~help command_name`). Escape newlines with `\`

# Macro arguments

`#[poise::command]` accepts a number of arguments to configure the command:

## Command types

- `prefix_command`: Generate a prefix command
- `slash_command`: Generate a slash command
- `context_menu_command`: Generate a context menu command

## Meta properties

- `subcommands`: List of subcommands `subcommands("foo", "bar", "baz")`
- `name_localized`: Adds localized name of the parameter `name_localized("locale", "new_name")` (slash-only)
- `description_localized`: Adds localized description of the parameter `description_localized("locale", "Description")` (slash-only)
- `rename`: Choose an alternative command name instead of the function name
    - Useful if your command name is a Rust keyword, like `move`
- `aliases`: Command name aliases (only applies to prefix commands)
- `category`: Category of this command which affects placement in the help command
- `custom_data`: Arbitrary expression that will be boxed and stored in `Command::custom_data`
- `identifying_name`: Optionally, a unique identifier for this command for your personal usage
- `install_context`: Installation contexts where this command is available (slash-only) (`unstable` feature)
- `interaction_context`: Interaction contexts where this command is available (slash-only) (`unstable` feature)

## Checks

- `required_permissions`: Permissions which the command caller needs to have
- `required_bot_permissions`: Permissions which the bot is known to need
- `default_member_permissions`: Like `required_permissions`, but checked server-side (slash only)
    - Due to being checked server-side, users without the required permissions are prevented from executing the command in the first place, which is a better experience
    - However, `default_member_permissions` has no effect on subcommands, which always inherit their permissions from the top-level command
    - Also, guild owners can freely change the required permissions for any bot command for their guild
- `owners_only`: Restricts command callers to a configurable list of owners (see FrameworkOptions)
- `guild_only`: Restricts command callers to only run on a guild
- `dm_only`: Restricts command callers to only run on a DM
- `nsfw_only`: Restricts command callers to only run on a NSFW channel
- `subcommand_required`: Requires a subcommand to be specified (prefix only)
- `check`: Path to a function which is invoked for every invocation. If the function returns false, the command is not executed (can be used multiple times)

## Help-related arguments

- `hide_in_help`: Hide this command in help menus
- `help_text_fn`: Path to a string-returning function which is used for command help text instead of documentation comments
    - Useful if you have many commands with very similar help messages: you can abstract the common parts into a function

## Edit tracking (prefix only)

- `track_edits`: Shorthand for `invoke_on_edit`, `track_deletion`, and `reuse_response` (prefix only)
- `invoke_on_edit`: Reruns the command if an existing invocation message is edited (prefix only)
- `track_deletion`: Deletes the bot response to a command if the command message is deleted (prefix only)
- `reuse_response`: After the first response, post subsequent responses as edits to the initial message (prefix only)

## Cooldown

- `global_cooldown`: Minimum duration in seconds between invocations, globally
- `user_cooldown`: Minimum duration in seconds between invocations, per user
- `guild_cooldown`: Minimum duration in seconds between invocations, per guild
- `channel_cooldown`: Minimum duration in seconds between invocations, per channel
- `member_cooldown`: Minimum duration in seconds between invocations, per guild member

## Other

- `on_error`: Error handling function
- `broadcast_typing`: Trigger a typing indicator while command runs (prefix only)
- `discard_spare_arguments`: Don't throw an error if the user supplies too many arguments (prefix only)
- `ephemeral`: Make bot responses ephemeral if possible (slash only)
    - Only poise's functions, like `poise::send_reply`, respect this preference

# Function parameters

`Context` is the first parameter of all command functions. It's an enum over either PrefixContext or
SlashContext, which contain a variety of context data each. Context provides some utility methods to
access data present in both PrefixContext and SlashContext, like `author()` or `created_at()`.

All following parameters are inputs to the command. You can use all types that implement `poise::PopArgument`, `serenity::ArgumentConvert` or `std::str::FromStr`.
You can also wrap types in `Option` or `Vec` to make them optional or variadic. In addition, there
are multiple attributes you can use on parameters:

## Meta properties

- `#[description = ""]`: Sets description of the parameter (slash-only)
- `#[description_localized("locale", "Description")]`: Adds localized description of the parameter (slash-only)
- `#[name_localized("locale", "new_name")]`: Adds localized name of the parameter (slash-only)
- `#[autocomplete = "callback()"]`: Sets the autocomplete callback (slash-only)
- `#[rename = "new_name"]`: Changes the user-facing name of the parameter (slash-only)

## Input filter (slash only)

- `#[channel_types("", "")]`: For channel parameters, restricts allowed channel types (slash-only)
- `#[min = 0]`: Minimum value for this number parameter (slash-only)
- `#[max = 0]`: Maximum value for this number parameter (slash-only)
- `#[min_length = 0]`: Minimum length for this string parameter (slash-only)
- `#[max_length = 1]`: Maximum length for this string parameter (slash-only)

## Parser settings (prefix only)
- `#[rest]`: Use the entire rest of the message for this parameter (prefix-only)
- `#[lazy]`: Can be used on Option and Vec parameters and is equivalent to regular expressions' laziness (prefix-only)
- `#[flag]`: Can be used on a bool parameter to set the bool to true if the user typed the parameter name literally (prefix-only)
    - For example with `async fn my_command(ctx: Context<'_>, #[flag] my_flag: bool)`, `~my_command` would set my_flag to false, and `~my_command my_flag` would set my_flag to true

# Help text

Documentation comments are used as command help text. The first paragraph is the command
description (`Command::description`) and all following paragraphs are the multiline help text
(`Command::help_text`).

In the multiline help text, put `\` at the end of a line to escape the newline.

Example:

```rust
/// This is the description of my cool command, it can span multiple
/// lines if you need to
///
/// Here in the following paragraphs, you can give information on how \
/// to use the command that will be shown in your command's help.
///
/// You could also put example invocations here:
/// `~coolcommand test`
#[poise::command(slash_command)]
pub async fn coolcommand(ctx: Context<'_>, s: String) -> Result<(), Error> { ... }
```
results in
```rust
poise::Command {
    description: Some("This is the description of my cool command, it can span multiple lines if you need to".into()),
    help_text: Some("Here in the following paragraphs, you can give information on how to use the command that will be shown in your command's help.\n\nYou could also put example invocations here:\n`~coolcommand test`".into()),
    ...
}
```

# Internals

Internally, this attribute macro generates a function with a single `poise::Command`
return type, which contains all data about this command. For example, it transforms a function of
this form:
```rust
/// This is a command
#[poise::command(slash_command, prefix_command)]
async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    // code
}
```
into something like
```rust
fn my_command() -> poise::Command<Data, Error> {
    async fn inner(ctx: Context<'_>) -> Result<(), Error> {
        // code
    }

    poise::Command {
        name: "my_command",
        description: "This is a command",
        prefix_action: Some(|ctx| Box::pin(async move {
            inner(ctx.into()).await
        })),
        slash_action: Some(|ctx| Box::pin(async move {
            inner(ctx.into()).await
        })),
        context_menu_action: None,
        // ...
    }
}
```

If you're curious, you can use [`cargo expand`](https://github.com/dtolnay/cargo-expand) to see the
exact desugaring
*/
#[proc_macro_attribute]
pub fn command(args: TokenStream, function: TokenStream) -> TokenStream {
    let args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(x) => x,
        Err(e) => return e.into_compile_error().into(),
    };

    let args = match <command::CommandArgs as darling::FromMeta>::from_list(&args) {
        Ok(x) => x,
        Err(e) => return e.write_errors().into(),
    };

    let function = syn::parse_macro_input!(function as syn::ItemFn);

    match command::command(args, function) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}
/**
Use this derive macro on an enum to easily generate a choice parameter type. A choice parameter
is mainly useful in slash commands. It allows you to constrain input to a fixed set of choices.

```rust
#[derive(poise::ChoiceParameter)]
pub enum MyChoice {
    #[name = "The first choice"]
    ChoiceA,
    // A choice can have multiple names
    #[name = "The second choice"]
    #[name = "ChoiceB"]
    ChoiceB,
    // Or no name, in which case it falls back to the variant name "ChoiceC"
    ChoiceC,
}
```

Example invocations:
- `~yourcommand "The first choice"` - without the quotes, each word would count as a separate argument
- `~yourcommand ChoiceB`
- `~yourcommand cHoIcEb` - names are case-insensitive

# Localization

In slash commands, you can take advantage of Discord's localization.

```rust
#[derive(poise::ChoiceParameter)]
pub enum Food {
    #[name_localized("de", "Eier")]
    #[name_localized("es-ES", "Huevos")]
    Eggs,
    #[name_localized("de", "Pizza")]
    #[name_localized("es-ES", "Pizza")]
    Pizza,
    #[name_localized("de", "Müsli")]
    #[name_localized("es-ES", "Muesli")]
    Cereals,
}
```

When invoking your slash command, users will be shown the name matching their locale.

You can also set localized choice names programmatically; see `CommandParameter::choices`
*/
#[proc_macro_derive(ChoiceParameter, attributes(name, name_localized))]
pub fn choice_parameter(input: TokenStream) -> TokenStream {
    let enum_ = syn::parse_macro_input!(input as syn::DeriveInput);

    match choice_parameter::choice_parameter(enum_) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}

/// See [`ChoiceParameter`]
#[deprecated = "renamed to ChoiceParameter"]
#[proc_macro_derive(SlashChoiceParameter, attributes(name))]
pub fn slash_choice_parameter(input: TokenStream) -> TokenStream {
    choice_parameter(input)
}

/// See `Modal` trait documentation
#[proc_macro_derive(
    Modal,
    attributes(name, placeholder, min_length, max_length, paragraph)
)]
pub fn modal(input: TokenStream) -> TokenStream {
    let struct_ = syn::parse_macro_input!(input as syn::DeriveInput);

    match modal::modal(struct_) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}
