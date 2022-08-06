# 0.3.0

New features:
- Custom arbitrary data associated with commands via custom_data command attribute and Command struct field
- Make it easier to run a custom event loop by exposing important setup functions
  - `set_qualified_names()`
  - `insert_owners_from_http()`
  - This code is run under the hood in the framework setup code and is now accessible to custom event loops too
- The time crate can now be used in place of chrono (#83)
- You can now use the `check` command attribute multiple times to add multiple checks
- Support for Discord's localization API (#82)
- `Framework::client()` method to get the underlying `serenity::Client`

API updates:
- `Framework::shard_manager()` now returns `&Arc` instead of `Arc`
- `Context::author_member()` return type changed from `Member` to `Cow<'_, Member>` to avoid needless cloning (#81)
  - Use `.into_owned()` to convert `Cow<'_, Member>` back to `Member`
- `fn message(self) -> Message` method on ReplyHandle split into `fn message(&self) -> Cow<'_, Message>` and `fn into_message(self) -> Message` (#84)
  - Reduces cloning if you just need a reference to the message
  - Allows using the ReplyHandle after retrieving the message
- `Framework::build()` renamed to `Framework::builder()` (#96)
- `Framework::start_with()` now takes `impl FnOnce` instead of `fn`
  - To be usable with with `serenity::Client`'s `start_shard`, `start_shard_range`, and `start_shards` methods
- Removed `serenity` re-export at crate root (`serenity_prelude` still exists!)
- Command fields renamed: `inline_help` => `description`, `multiline_help` => `help_text`
- Command and parameter `name` and `description` fields changed from `&'static str` to `String`
  - To make them overrideable at runtime
- Autocomplete functions' partial input parameter is now always `&str`
  - Was previously String, [i/u][8/16/32/64], f[32/64], depending on parameter type

Behavior changes:
- Fix identifying_name command attribute not being applied
- Built-in register commands now report command count correctly
  - Prefix-only commands aren't included anymore
- `ReplyHandle::edit()` now keeps parts of the message as before if not specified
- Fix number autocomplete (was completely borked)
- Code block parsing is now more precise
  - With \`\`\`textwithspecialcharacters, textwithspecialcharacters is no longer recognized as the programming language, but as part of the code (mirroring Discord's behavior)
- `ReplyHandle::edit()` now works on ephemeral followup responses

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.2.2...v0.3.0

# 0.2.2

Behavior changes:
- `default_member_permissions` fixed to not constrain commands to administrators by default

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.2.1...v0.2.2

# 0.2.1

Behavior changes:
- Bot ID is retrieved from first Ready event again instead of extracted from the token. Extracting the bot ID from the bot token often didn't work and caused breakage for many users. This should now be fixed

https://github.com/kangalioo/poise/compare/v0.2.0...v0.2.1

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

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.1.0...v0.2.0

# 0.1.0

Initial crates.io release
