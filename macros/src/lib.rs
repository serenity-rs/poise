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
- `install_context`: Installation contexts where this command is available (slash-only)
- `interaction_context`: Interaction contexts where this command is available (slash-only)

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

## Edit tracking (prefix only)

- `track_edits`: Shorthand for `invoke_on_edit`, `track_deletion`, and `reuse_response` (prefix only)
- `invoke_on_edit`: Reruns the command if an existing invocation message is edited (prefix only)
- `track_deletion`: Deletes the bot response to a command if the command message is deleted (prefix only)
- `reuse_response`: After the first response, post subsequent responses as edits to the initial message (prefix only)

## Cooldown
- `manual_cooldowns`: Allows overriding the framework's built-in cooldowns tracking without affecting other commands.
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

All following parameters are inputs to the command. You can use all types that implement
`PopArgument` (for prefix commands) or `SlashArgument` (for slash commands). You can also wrap
types in `Option` or `Vec` to make them optional or variadic. In addition, there are multiple
attributes you can use on parameters:

## Meta properties

- `#[description = ""]`: Sets description of the parameter (slash-only)
- `#[description_localized("locale", "Description")]`: Adds localized description of the parameter (slash-only)
- `#[name_localized("locale", "new_name")]`: Adds localized name of the parameter (slash-only)
- `#[autocomplete = "callback()"]`: Sets the autocomplete callback (slash-only)
- `#[rename = "new_name"]`: Changes the user-facing name of the parameter (slash-only)

## Input filter (slash only)

- `#[channel_types("", "")]`: For channel parameters, restricts allowed channel types (slash-only)
- `#[file_types("", "")`: For attachment parameters, restricts allowed file types (slash-only)
- `#[min = 0]`: Minimum value for this number parameter (slash-only)
- `#[max = 0]`: Maximum value for this number parameter (slash-only)
- `#[min_length = 0]`: Minimum length for this string parameter (slash-only)
- `#[max_length = 1]`: Maximum length for this string parameter (slash-only)

## Parser settings
- `#[string]`: Indicates that a type implements `FromStr` and should be parsed from a string argument.
- `#[rest]`: Use the entire rest of the message for this parameter (prefix-only)
- `#[lazy]`: Can be used on Option and Vec parameters and is equivalent to regular expressions' laziness (prefix-only)
- `#[flag]`: Can be used on a bool parameter to make it optional and default to `false`; additionally,
  in prefix commands only, the user can pass in the parameter name literally to set it to `true`.
    - For example with `async fn my_command(ctx: Context<'_>, #[flag] my_flag: bool)`, `~my_command` or `/my_command` would set `my_flag` to false, while `~my_command my_flag` would set `my_flag` to true

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

/**
Use this derive macro on a struct to easily generate a modal interaction, Discord's version
of interactive forms.

Modals are made up of components, which define their layout, content, and type of input accepted.
A single modal can include up to five components, and all [available modal components][components]
are supported by the macro.

# Example

```rust
# use poise::serenity_prelude as serenity;
# type Data = ();
# type Error = serenity::Error;
use poise::Modal;
type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;

#[derive(Debug, Modal)]
#[name = "Modal Title"] // Struct name by default
#[text = "My *fancy* `modal`, created using [Poise](https://serenity-rs.github.io/) :crab:"]
struct MyModal {
    #[name = "First text input"] // Field name by default
    #[description = "Displayed under name"] // No description by default
    #[placeholder = "Your first input goes here"] // No placeholder by default
    #[min_length = 5] // No length restriction by default (up to 4000 chars)
    #[max_length = 500]
    first_input: String,
    #[name = "Second text input"]
    #[paragraph] // Switches from single-line to multi-line text box
    second_input: Option<String>, // Option means optional input
    #[name = "File upload"]
    #[file_upload] // Allows user to upload up to 10 files
    #[min_values = 2] // Min number of files (0-10 for files)
    #[max_values = 5]
    third_input: Vec<serenity::Attachment>,
    #[name = "String select menu"]
    #[string_select("Option 1", "Option 2")] // Selectable strings
    #[min_values = 2] // Min number of selections required (0-25 for select menus)
    fourth_input: Vec<String>,
}

#[poise::command(slash_command)]
pub async fn modal(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let data = MyModal::execute(ctx).await?;
    println!("Got data: {:?}", data);

    Ok(())
}
```

# Struct attributes

- `#[name = ""]`: Sets the modal title. Defaults to struct name if omitted.
- `#[text = ""]`: Optional [text display][td] component, shown below the modal title. Can
include markdown-formatted text, mentions (users, roles, etc.), and emojis. Note that this
counts toward the maximum total of five components per modal.

# Field attributes

The text display component is the only content-style component available in modals.
However, despite not being interactive, they still count toward the five-component
maximum per modal. Text display components can be added above any input field using
the following attribute:

- `#[text_display]`: Equivalent to the struct-level `#[text]` attribute above.

Because attributes must be placed above fields, text display components will necessarily
be paired with interactive (input) components. Position of the attribute relative to the
interactive component attribute does not matter; the text display will be rendered on top.

```rust
#[text_display = "**Huge** markdown-friendly text. Shown *above* the `text input` component."]
#[name = "Input Label #1"]
text_input_one: Option<String>,
#[name = "Input Label #2"]
#[text_display = "Despite the attribute position, still shows __above__ 'Input Label 2'."]
text_input_two: Option<String>,
```

The following field attributes are shared by all interactive components:

- `#[name = ""]`: Sets the input label. Defaults to field name. Max 45 chars.
- `#[description = ""]`: Adds an optional description under the label. Max 100 chars.
- `#[placeholder = ""]`: Adds optional placeholder text. Max 100 chars.

The default component is the [text input][ti] component, which returns a `String`. The following
field attributes are valid for text input components only:

- `#[min_length = 0]`: Minimum number of characters (0-4000).
- `#[max_length = 1]`: Maximum number of characters (1-4000).
- `#[paragraph]`: Switches to a multi-line input box. Default is single-line.

Other interactive components supported by the macro include the [file upload][fu],
[string select][ss], [user select][us], [role select][rs], [mentionable select][ms], and
[channel select][cs] components. Component type is indicated by using one of the following
field attributes (**one per field**):

- `#[file_upload]`: Allows the user to upload files (0-10). Returns [`Vec<Attachment>`][att].
- `#[string_select("", "")]`: Supports 1-25 **unique** options (up to 100 chars each), defined
in the attribute. Returns `Vec<String>`.
- `#[user_select]`: Returns [`Vec<User>`][user].
- `#[role_select]`: Returns [`Vec<Role>`][role].
- `#[mentionable_select]`: Returns [`Mentionables`][mentionables].
- `#[channel_select]`: Returns [`Vec<GenericChannelId>`][gic].

Optionally, emojis and/or descriptions may be added to string select menu options:

- `#[string_select_emojis("", "")`
- `#[string_select_descriptions("", "")]`: Max 100 chars per description.

If used, the number of emojis and/or descriptions provided must not be less than the number
of options provided; any additional items will be ignored. Unicode emojis should be inserted
directly. Custom emojis should use the Discord angle bracket format: `<:NAME:EMOJI_ID>` for
static or `<a:NAME:EMOJI_ID>` for animated. Emojis given in an invalid format will be ignored.

```rust
#[name = "My cool select menu"]
#[string_select("Option 1", "Option 2", "Option 3")]
#[string_select_emojis(
    "🦀",
    "<:ferris_owo:1033109474782761110>",
    "<a:ferris_bongo:494140332812926981>"
)]
#[string_select_descriptions(
    "Uses a Unicode icon",
    "Uses a custom static icon",
    "Uses a custom animated icon"
)]
selections: Option<Vec<String>>
```

Minimum and maximum values for file upload and select menu components are defined
using the following field attributes:

- `#[min_values = 0]`: 0-10 for files; 0-25 for select menus. Defaults to 1.
- `#[max_values = 25]`: 1-10 for files; 1-25 for select menus. Defaults to 1.

```rust
#[name = "Role select menu"]
#[role_select]
#[max_values = 1]
roles: Option<Vec<serenity::Role>>
```

Note that file upload and select menu components can be optional ***and*** have a `min_values`
value defined at the same time. In such cases, the defined minimum only comes into effect when
input is attempted. In the following example, the user would be able to submit the modal with
either no mentionables ***or*** at least three mentionables selected.

```rust
#[name = "Mentionable select menu"]
#[mentionable_select]
#[min_values = 3]
mentionables: Option<poise::Mentionables>
```

For the channel select menu, channel types to include in the list may optionally be defined
using the following field attribute:

- `#[channel_types("", "")]`: See [`ChannelType`][ct] for valid channel types.

```rust
#[name = "Channel select menu"]
#[channel_select()]
#[channel_types("Text", "Forum")]
channels: Vec<serenity::GenericChannelId>
```

# Specifying defaults

Defaults may be provided for text input and select menu components using an initialized instance
of the modal struct with [`execute_with_defaults()`][ewd], or with [`execute_modal()`][exe] if you
wish to specify a timeout. For example, assuming the struct from the initial example:

```rust
let data = MyModal::execute_with_defaults(
    ctx,
    MyModal {
        first_input: "Default text input".to_string(),
        second_input: None,
        third_input: vec![],
        fourth_input: vec!["Option 2".to_string()],
    },
)
.await?;
```

Alternatively, if the struct also derives `Default`:

```rust
let data = MyModal::execute_with_defaults(
    ctx,
    MyModal {
        first_input: "Default text input".to_string(),
        fourth_input: vec!["Option 2".to_string()],
        ..Default::default()
    },
)
.await?;
```

And using [`execute_modal()`][exe] with a timeout:

```rust
let data = poise::execute_modal(
    ctx,
    Some(MyModal {
        first_input: "Default text input".to_string(),
        fourth_input: vec!["Option 2".to_string()],
        ..Default::default()
    }),
    Some(std::time::Duration::from_secs(300)),
)
.await?;
```

[td]:https://docs.discord.com/developers/components/reference#text-display
[ti]:https://docs.discord.com/developers/components/reference#text-input
[fu]:https://docs.discord.com/developers/components/reference#file-upload
[ss]:https://docs.discord.com/developers/components/reference#string-select
[us]:https://docs.discord.com/developers/components/reference#user-select
[rs]:https://docs.discord.com/developers/components/reference#role-select
[cs]:https://docs.discord.com/developers/components/reference#channel-select
[ms]:https://docs.discord.com/developers/components/reference#mentionable-select
[att]:https://docs.rs/serenity/latest/serenity/model/channel/struct.Attachment.html
[user]:https://docs.rs/serenity/latest/serenity/model/user/struct.User.html
[role]:https://docs.rs/serenity/latest/serenity/model/guild/struct.Role.html
[gic]:https://serenity-rs.github.io/serenity/next/serenity/model/id/struct.GenericChannelId.html
[mentionables]:https://serenity-rs.github.io/poise/next/poise/modal/struct.Mentionables.html
[ct]:https://docs.rs/serenity/latest/serenity/model/channel/enum.ChannelType.html
[exe]:https://serenity-rs.github.io/poise/next/poise/modal/fn.execute_modal.html
[ewd]:https://serenity-rs.github.io/poise/next/poise/modal/trait.Modal.html#method.execute_with_defaults
[components]:https://docs.discord.com/developers/components/reference#component-object-component-types
*/
#[proc_macro_derive(
    Modal,
    attributes(
        name,
        text,
        description,
        placeholder,
        min_length,
        max_length,
        paragraph,
        text_display,
        file_upload,
        string_select,
        string_select_emojis,
        string_select_descriptions,
        user_select,
        role_select,
        mentionable_select,
        channel_select,
        channel_types,
        min_values,
        max_values,
    )
)]
pub fn modal(input: TokenStream) -> TokenStream {
    let struct_ = syn::parse_macro_input!(input as syn::DeriveInput);

    match modal::modal(struct_) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}
