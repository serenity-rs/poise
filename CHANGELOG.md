# 0.2.1

Behavior changes:
- Bot ID is retrieved from first Ready event again instead of extracted from the token. Extracting the bot ID from the bot token often didn't work and caused breakage for many users. This should now be fixed

# 0.2.0

New features:
- `PrefixFrameworkOptions::ignore_bots` to disable executing commands posted by bots
- `builtins::register_application_commands_buttons()` allows managing slash command registration with buttons
- `#[poise(subcommands(...))]` attribute argument as a convenient alternative to manually setting the `Command::subcommands` field
- `dispatch_event()` function to allow running your own event loop without involving Framework at all
- `Modal::create()` and `Modal::execute_with_defaults()` can be used to spawn a modal with pre-filled values
- `default_member_permissions` command attribute argument, part of Discord Permissions V2
- `Context::partial_guild` to easily get guild information without cache enabled

API updates:
- Poise no longer depends on serenity's cache feature
- Some structs and enums were made non-exhaustive to make future non-breaking changes easier
- The `Event` enum has been updated to include all `serenity::EventHandler` events again
- `FrameworkBuilder::initialize_owners()` takes self instead of &mut self, like the other builder methods
- The macro generated code emits `::std::result::Result` instead of `Result` now, which prevents collisions with user-defined Result type aliases
- `CreateReply` now implements `Clone`
- Some callbacks were changed to receive `FrameworkContext<'_, U, E>` instead of `&Framework<U, E>` (had to be done for `dispatch_event()`)

Behavior changes:
- Autocomplete callbacks and argument parse errors don't cause a cooldown trigger anymore
- Features using the bot's ID now work again (execute_self_messages, required_bot_permissions, mention_as_prefix)
- Guild owners can no longer register slash commands in their guild (only bot owners now)
- Compile times should be faster through less monomorphization bloat
- Attachments in initial responses are supported now

# 0.1.0

Initial crates.io release
