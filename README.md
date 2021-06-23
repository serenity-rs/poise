# Poise
Poise is an opinionated Discord bot framework with a few distinctive features:
- edit tracking: when user edits their message, automatically update bot response 
- slash commands: completely define both normal and slash commands with a single function
- flexible argument parsing: command parameters are defined with normal Rust types and parsed automatically

I created this framework mainly for personal use ([rustbot](https://github.com/kangalioo/rustbot) and [etternabot](https://github.com/kangalioo/etternabot)). Features are added on demand, since features can be designed better with a concrete use case at hand.

# Example
This is what a sample command definition looks like:
```rust
/// Display your or another user's account creation date
#[poise::command(slash_command, track_edits)]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or(ctx.author());
    poise::say_reply(
        ctx,
        format!("{}'s account was created at {}", user.name, user.created_at())
    );
}
```

# About the weird name
I'm bad at names. Google lists "poise" as a synonym to "serenity" which is the Discord library
underlying this framework, so that's what I chose.