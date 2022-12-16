# 0.5.2

New features:
- Added `track_deletion` feature to commands
- Added all of `Context`'s methods to `PrefixContext` and `ApplicationContext`

Behavior changes:
- Editing commands not marked track_edits no longer re-runs the command
- `builtins::servers` now shows hidden statistics for the entire bot team, not just owner

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.5.1...v0.5.2

# 0.5.1

New features:
- Added `FrameworkOptions::skip_checks_for_owner`

Behavior changes:
- `execute_modal` doesn't panic anymore when the timeout is reached
- Checking user permissions properly falls back to HTTP when cache is enabled but empty

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.5.0...v0.5.1

# 0.5.0

New features:
- Added `Context::parent_commands()`
- Added `Context::invocation_string()`
- Added `builtins::register_in_guild()` and `builtins::register_globally()` convenience functions
- The return value of autocomplete callbacks can be any serializable type now
- `Context` can now be passed directly into most serenity API functions
  - Because it now implements `AsRef<Cache>`, `AsRef<Http>`, `AsRef<ShardMessenger>`, and `CacheHttp` traits
- Added `execute_modal()` function with support for modal timeouts

API updates:
- `Modal::create()` gained a `custom_id: String` parameter
  - To make it possible to tell apart two modal interactions
- Removed `CreateReply::reference_message(MessageReference)` in favor of `CreateReply::reply(bool)`
  - For the unusual case of setting a different reference message than the invocation (why would you? I'm genuinely interested), you can still convert the `CreateReply` into `serenity::CreateMessage` manually via `.to_prefix()` and call `serenity::CreateMessage`'s `reference_message()` method
- Renamed `FrameworkBuilder::user_data_setup()` method to `setup()`
- Renamed `FrameworkOptions::listener` field to `event_handler`
- Renamed `Context::discord()` method to `serenity_context()`

Behavior changes:
- `register_application_commands_buttons()` now has emojis, reworked wording, and prints the time taken to register
- `Modal::execute()` always responds to the correct modal now
- When a subcommand is invoked, all parent commands' checks are run too, now

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.4.1...v0.5.0

# 0.4.1

Behavior changes:
- Slash commands marked guild_only don't show up in DMs anymore
  - Using Discord's dm_permission field on commands
- `poise::builtins::servers` now doesn't omit unavailable guilds from guild count and list anymore

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.4.0...v0.4.1

# 0.4.0

New features:
- Added std::error::Error impl for FrameworkError
- Added `FrameworkError::discord() -> &serenity::Context` method
- Added `FrameworkError::ctx() -> Option<poise::Context>` method
- Added `FrameworkError::handle()` method which calls the appropriate on_error function on itself
- Added Copy and Clone impl for PartialContext
- Added `ReplyHandle::delete()`
- Added `FrameworkError::UnknownCommand` and `FrameworkError::UnknownInteraction`
  - These error cases would previously just `log::warn!()`
- Exposed internals of `dispatch_message()` as new functions `parse_invocation()` and `run_invocation()`
- Added trigger and action callback to PrefixContext
- Made EditTracker methods public: `process_message_update()`, `set_bot_response()`, `track_command()`

API updates:
- Added or changed fields of some FrameworkError enum variants
- Removed `cmd: &Command` parameter from check_permissions_and_cooldown (Context already includes it)
- triggered_by_edit and previously_tracked bool parameters replaced by new MessageDispatchTrigger enum
- Simplified return type of `dispatch_message()`, `dispatch_interaction()`, and `dispatch_autocomplete()` to `Result<(), FrameworkError>`
- Simplified return type of `extract_command_and_run_checks()` to `Result<ApplicationContext, FrameworkError>`
- Removed `futures_core` re-export

Behavior changes:
- Internal warnings now use `log::warn!()`/`log::error!()` instead of `eprintln!()`
  - That way, you can mute them or handle them specially
- Default `FrameworkError::DynamicPrefix` handler now prints message content
- `ReplyHandle::edit()` now replaces existing attachments and embeds instead of adding on top
- Cooldowns are now triggered before command execution instead of after
- Added `log::warn!()` in some weird code paths that shouldn't be hit
- When a focused autocomplete option has an unrecognized name and when the autocomplete value is not a string, `FrameworkError::CommandStructureMismatch` is now thrown
  - Instead of discarding the error
- `register_application_commands_buttons()` switched order of rows
  - Guild-specific actions are at the top because they are more common and less destructive

Detailed changelog: https://github.com/kangalioo/poise/compare/v0.3.0...v0.4.0

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
