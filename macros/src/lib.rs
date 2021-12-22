mod command;
mod slash_choice_parameter;

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
- `aliases`: Command name aliases (only applies to prefix commands)
- `track_edits`: Enable edit tracking (only applies to prefix commands)
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
- `owners_only`: Restricts command callers to the list of owners specified in framework options

# Function parameters

`Context` is the first parameter of all command functions. It's an enum over either PrefixContext or
SlashContext, which contain a variety of context data each. Context provides some utility methods to
access data present in both PrefixContext and SlashContext, like `author()` or `created_at()`.

All following parameters are inputs to the command. You can use all types that implement
`poise::PopArgumentAsync`, `poise::PopArgument`, `serenity::ArgumentConvert` or `std::str::FromStr`.
You can also wrap types in `Option` or `Vec` to make them optional or variadic. In addition, there
are multiple attributes you can use on parameters:
- `#[description]`: Required for slash commands; a description of the parameter
- `#[rest]`: Relevant for prefix commands; means that the entire rest of the message is parsed as the parameter even without quotes
- `#[lazy]`: Relevant for prefix commands; can be used on Option and Vec parameters and is equivalent to regular expressions' laziness
- `#[flag]`: Relevant for prefix commands; can be used on a bool parameter to set the bool to true if the user typed the parameter name literally
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
        prefix: Some(poise::PrefixCommand {
            name: "my_command",
            action: |ctx, args| Box::pin(async move {
                inner(ctx.into()).await
            }),
            // ...
        }),
        slash: Some(ooise::SlashCommand {
            name: "my_command",
            description: "This is a command",
            action: |ctx, args| Box::pin(async move {
                inner(ctx.into()).await
            })
            // ...
        }),
        context_menu: None,
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
Use this derive macro on an enum to easily generate a _choice parameter_ type. A _choice parameter_
is mainly useful in slash commands. It allows you to constrain input to a fixed set of choices.

# Example

```rust
#[derive(Debug, poise::SlashChoiceParameter)]
pub enum MyStringChoice {
    #[name = "The first choice"]
    ChoiceA,
    #[name = "The second choice"]
    ChoiceB,
}

/// Dummy command to test slash command choice parameters
#[poise::command(prefix_command, slash_command)]
pub async fn choice(
    ctx: Context<'_>,
    #[description = "The choice you want to choose"] choice: MyStringChoice,
) -> Result<(), Error> {
    ctx.say(format!("You entered {:?}", choice)).await?;
    Ok(())
}
```
*/
#[proc_macro_derive(SlashChoiceParameter, attributes(name))]
pub fn slash_choice_parameter(input: TokenStream) -> TokenStream {
    let enum_ = syn::parse_macro_input!(input as syn::DeriveInput);

    match slash_choice_parameter::slash_choice_parameter(enum_) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}
