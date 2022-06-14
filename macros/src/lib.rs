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
- `prefix_command`: Generate a prefix command
- `slash_command`: Generate a slash command
- `context_menu_command`: Generate a context menu command
- `description_localized`: Adds localized description of the parameter `description_localized("locale", "Description")` (slash-only)
- `name_localized`: Adds localized name of the parameter `name_localized("locale", "new_name")` (slash-only)
- `subcommands`: List of subcommands `subcommands("foo", "bar", "baz")`
- `aliases`: Command name aliases (only applies to prefix commands)
- `invoke_on_edit`: Reruns the command if an existing invocation message is edited (prefix only)
- `reuse_response`: After the first response, post subsequent responses as edits to the initial message (prefix only)
- `track_edits`: Shorthand for `invoke_on_edit` and `reuse_response` (prefix only)
- `broadcast_typing`: Trigger a typing indicator while command runs (only applies to prefix commands I think)
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
- `required_bot_permissions`: Permissions which the bot is known to need
- `owners_only`: Restricts command callers to a configurable list of owners (see FrameworkOptions)
- `guild_only`: Restricts command callers to only run on a guild
- `dm_only`: Restricts command callers to only run on a DM
- `nsfw_only`: Restricts command callers to only run on a NSFW channel
- `identifying_name`: Optionally, a unique identifier for this command for your personal usage
- `category`: Category of this command which affects placement in the help command
- `custom_data`: Arbitrary expression that will be boxed and stored in `Command::custom_data`
- `global_cooldown`: Minimum duration between invocations, globally
- `user_cooldown`: Minimum duration between invocations, per user
- `guild_cooldown`: Minimum duration between invocations, per guild
- `channel_cooldown`: Minimum duration between invocations, per channel
- `member_cooldown`: Minimum duration between invocations, per guild member

# Function parameters

`Context` is the first parameter of all command functions. It's an enum over either PrefixContext or
SlashContext, which contain a variety of context data each. Context provides some utility methods to
access data present in both PrefixContext and SlashContext, like `author()` or `created_at()`.

All following parameters are inputs to the command. You can use all types that implement
`poise::PopArgumentAsync`, `poise::PopArgument`, `serenity::ArgumentConvert` or `std::str::FromStr`.
You can also wrap types in `Option` or `Vec` to make them optional or variadic. In addition, there
are multiple attributes you can use on parameters:
- `#[description = ""]`: Sets description of the parameter (slash-only)
- `#[description_localized("locale", "Description")]`: Adds localized description of the parameter (slash-only)
- `#[name_localized("locale", "new_name")]`: Adds localized name of the parameter (slash-only)
- `#[autocomplete = "callback()"]`: Sets the autocomplete callback (slash-only)
- `#[channel_types("", "")]`: For channel parameters, restricts allowed channel types (slash-only)
- `#[rename = "new_name"]`: Changes the user-facing name of the parameter (slash-only)
- `#[min = 0]`: Minimum value for this number parameter (slash-only)
- `#[max = 0]`: Maximum value for this number parameter (slash-only)
- `#[rest]`: Use the entire rest of the message for this parameter (prefix-only)
- `#[lazy]`: Can be used on Option and Vec parameters and is equivalent to regular expressions' laziness (prefix-only)
- `#[flag]`: Can be used on a bool parameter to set the bool to true if the user typed the parameter name literally (prefix-only)
    - For example with `async fn my_command(ctx: Context<'_>, #[flag] my_flag: bool)`, `~my_command` would set my_flag to false, and `~my_command my_flag` would set my_flag to true

# Internals

Internally, this attribute macro generates a function with a single [`poise::Command`]
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
    let args = syn::parse_macro_input!(args as Vec<syn::NestedMeta>);
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
