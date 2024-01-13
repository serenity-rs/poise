<!--
Template:
```
# 0.5.7
New features:
- Added some new feature / ability (3afd86c58d883970ddd236dc7c8a0d5c5a0d9e3)
  - If needed, additional explanation / context here

API updates:
- Changed something about the API, typically backwards-incompatible (d96d56b1eda29221e6f2d9c9af05c59a42feb9f)

Behavior changes:
- An existing feature or function changed its behavior (e40dd711d748b6398611db97f54e1622ac008ae)
  - For example, different output to the user

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.6...v0.5.7

Thanks to @ User1, @ User2, @ User3!
```

Also run the two find-and-replace regexes below for nice formatting:

To quickly annotate commit hashes, append the full hash in parantheses to each line and then run
this find-and-replace regex (VSCode flavor):
- Find: (?<!commit/)([0-9a-f]{7})([0-9a-f]{33})
- Replace: [$1](https://github.com/serenity-rs/poise/commit/$1$2)

To quickly make GitHub usernames into clickable links, prepend each username with @ and then run
this find-and-replace regex (VSCode flavor):
- Find: (?<=Thanks to.*)(?<!\[)@([a-z0-9]+)
- Replace: [@$1](https://github.com/$1)
-->

# 0.6.1

New features:
- Choice parameters can now be defined inline in a function signature ([6167311](https://github.com/serenity-rs/poise/commit/6167311216065b953c6dc64406e56a31e52cd9a1))
  
  For example:
  ```rust
  #[choices("Europe", "Asia", "Africa", "America", "Australia", "Antarctica")]
  continent: &'static str
  ```

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.6.0...v0.6.1

Thanks to [@kangalio](https://github.com/kangalio)!

# 0.6.0

Now with serenity 0.12!


New features:
- `ChoiceParameter` is now not only a derive macro, but also a trait, so you can implement it manually ([bc250b8](https://github.com/serenity-rs/poise/commit/bc250b852d5dc3ef57c5ab1d27d6de0bf5599b0b))
- `HelpConfiguration::include_description` can be used to toggle whether a single command's help output includes its `description`, or only its `help_text` ([0ad8ee6](https://github.com/serenity-rs/poise/commit/0ad8ee668ca2b131ec95cfd8f258f11d3f5b47fb))


API updates:
- A lot of structs and enum are now `#[non_exhaustive]` to allow backwards compatible new features in the future ([035e035](https://github.com/serenity-rs/poise/commit/035e03574956f68af582e3ac28478ac32273e172), [6c08cfb](https://github.com/serenity-rs/poise/commit/6c08cfba3af84b4a740611f46f380a3f92aaf810), [1cbfeef](https://github.com/serenity-rs/poise/commit/1cbfeefd7ac4fb26ab73cb61620717bb971a172c))
- `Command` no longer has `Default` bounds on its generics ([695ae1d](https://github.com/serenity-rs/poise/commit/695ae1dd8aaeeaf37129c3d53e90e63daaaa7be0))
- Changed some field types on `Command` to be more idiomatic ([6c08cfb](https://github.com/serenity-rs/poise/commit/6c08cfba3af84b4a740611f46f380a3f92aaf810))
- `CooldownTracker` methods now take a dedicated user-constructable `CooldownContext` instead of `Context<'_, U, E>`, to make `CooldownTracker` usable outside poise internals ([bd73861](https://github.com/serenity-rs/poise/commit/bd73861d2679c26040353eba460d933c7d3a7a15))

Behavior changes:
- Rework of the help command to make it nicer ([d38d226](https://github.com/serenity-rs/poise/commit/d38d226e82bd443f7615f4a505fc6803860d15ee), [d038ee6](https://github.com/serenity-rs/poise/commit/d038ee69ade7fc5bb90327093f5a90436eb30d45))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.7...v0.6.0

Thanks to [@kangalio](https://github.com/kangalio), [@docwilco](https://github.com/docwilco), [@drwilco](https://github.com/drwilco), [@GnomedDev](https://github.com/GnomedDev), [@scottbot95](https://github.com/scottbot95)!

# 0.5.7

New features:
- Added serenity trait impls for ApplicationContext and PrefixContext as well ([e40dd71](https://github.com/serenity-rs/poise/commit/e40dd711d748b6398611db97f54e1622ac008ae9))
  - Previously, `AsRef<Cache>`, `AsRef<Http>`, `AsRef<ShardMessenger>`, `AsRef<Context>`, and `CacheHttp` were only implemented for `poise::Context`
  - With these impls, poise's context types can be used as-is for the context parameter in serenity functions
- Support generics in `#[poise::command]`-annotated functions ([dfed53e](https://github.com/serenity-rs/poise/commit/dfed53ef14b4a492b2e0903fa154d4187554c84f))
- Added `Context.guild_channel()` ([83a73a3](https://github.com/serenity-rs/poise/commit/83a73a34fd40fbb8bc28feabd11989630b3c1b7f))

Behavior changes:
- Improved formatting of `builtins::servers` command ([3afd86c](https://github.com/serenity-rs/poise/commit/3afd86c58d883970ddd236dc7c8a0d5c5a0d9e38))
  - More compact and respects the message character limit
- Titles from thread creations are not interpreted as command invocations anymore ([bf3294d](https://github.com/serenity-rs/poise/commit/bf3294d44dfb39b5ca4e429cbe563804e1bfd998))
  - To return to previous behavior, set `PrefixFrameworkOptions.ignore_thread_creation` to `false`

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.6...v0.5.7

Thanks to [@NotNorom](https://github.com/NotNorom), [@kangalio](https://github.com/kangalio), [@m4tx](https://github.com/m4tx), [@seraxis](https://github.com/seraxis)!

# 0.5.6

New features:
- Added `cache()`, `http()`, `reply()`, `ping()` methods to `Context`, `ApplicationContext`, `PrefixContext` ([a72b3f3](https://github.com/serenity-rs/poise/commit/a72b3f36eeb0bb86b1fdf4f8beead842d96ecd1b), [2f9b95d](https://github.com/serenity-rs/poise/commit/2f9b95d6f83e8d7ef8cd522b971f706eda917915), [aaa57f6](https://github.com/serenity-rs/poise/commit/aaa57f628696f74a7d50ed6bf64e0fca1cce87ae))
- Added `subcommand_required` command attribute ([7866109](https://github.com/serenity-rs/poise/commit/78661097e9216b058c3f1a06323f528acf5492d5))
  - When a command with subcommand_required is invoked without a subcommand (only possible as a text/prefix command), an error is thrown
- Added `execute_modal_on_component_interaction` utility function ([5d02b87](https://github.com/serenity-rs/poise/commit/5d02b8757d30e4588c193c5ba06e38806bbc1021))
  - Previously, poise only features a utility function for opening modals from command invocations
- Added `remaining_cooldown_2` as the successor to `remaining_cooldown` which allows changing the cooldown config on a per-invocation basis instead of per-command ([c9c0373](https://github.com/serenity-rs/poise/commit/c9c037397afe5fabd56a3adff2ac3a59e52b68b4))
  - In the next breaking release, `remaining_cooldown` will be replaced with `remaining_cooldown_2`
- Added `Command.source_code_name` ([719bd50](https://github.com/serenity-rs/poise/commit/719bd50d20d823c34e8dd016a6200c45fa4a5ec6))

Behavior changes:
- Reply messages (i.e. reference_message set to Some) now ping by default ([a6b0b41](https://github.com/serenity-rs/poise/commit/a6b0b41301c0caef49496368678e6eb17c88cb18))
  - This matches the default from serenity and the default from the Discord client
- Raw identifiers can now be used for command names and command parameter names ([cfc1d42](https://github.com/serenity-rs/poise/commit/cfc1d42caec30c22b9c31b0ff4b666c0c64906ac))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.5...v0.5.6

Thanks to [@kangalio](https://github.com/kangalio), [@sadorowo](https://github.com/sadorowo), [@xtfdfr](https://github.com/xtfdfr), [@G0ldenSp00n](https://github.com/G0ldenSp00n), [@B-2U](https://github.com/B-2U), [@OverzealousLotus](https://github.com/OverzealousLotus), [@arqunis](https://github.com/arqunis), [@Friendly-Banana](https://github.com/Friendly-Banana), [@seqre](https://github.com/seqre)!

# 0.5.5

New features:
- Added `#[min_length]` and `#[max_length]` support for slash command string parameters ([116b8bb](https://github.com/serenity-rs/poise/commit/116b8bbe8638e1e9c69280fde43780d5657abbb1))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.4...v0.5.5

Thanks to [@kangalio](https://github.com/kangalio)!

# 0.5.4

API updates:
- The `payload` field of `FrameworkError::CommandPanic` has been changed from `Box<dyn Any + Send>` to `Option<String>` ([7a29dfe](https://github.com/serenity-rs/poise/commit/7a29dfe38eea638392ada7c4268e1c23a6ac7af4))
  - This is technically a breaking change
  - However, the newly introduced `payload` field in 0.5.3 made `FrameworkError` accidentally not Sync anymore
  - And `FrameworkError::CommandPanic` has only been introduced a few days ago in 0.5.3
  - Therefore, I think it's ok to release this as a patch release to reverse the accidental breaking change from 0.5.3

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.3...v0.5.4

Thanks to [@kangalio](https://github.com/kangalio)!

# 0.5.3

New features:
- Added `builtins::paginate()` as an example implementation of pagination ([2ab3662](https://github.com/serenity-rs/poise/commit/2ab3662e9fd3b4214a1877ba4336b80bd93c948f))
- Added missing events in `EventWrapper` ([1448eed](https://github.com/serenity-rs/poise/commit/1448eedb880376e4442429a3670db27568d412c2))
- Added `FrameworkError::CommandPanic` to allow custom handling of panics ([1c7a5a7](https://github.com/serenity-rs/poise/commit/1c7a5a7662c09744163b354f405fa45250fb5a0d))
  - `builtins::on_error` responds with an "Internal error" embed when encountering `CommandPanic`

Behavior changes:
- `builtins::on_error` now prints `FrameworkError::Command` not just in Discord chat, but in console as well ([0a03fb9](https://github.com/serenity-rs/poise/commit/0a03fb905ca0bc3b2ee0701fe35d3c89ecf5a654))
  - Because responding in Discord sometimes doesn't work, see commit description
- Fixed a compile error when `name_localized` or `description_localized` are used multiple times ([25fb3dc](https://github.com/serenity-rs/poise/commit/25fb3dc4b9aef36e96110c32306d2cdd872d553f))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.2...v0.5.3

Thanks to [@kangalio](https://github.com/kangalio), [@GnomedDev](https://github.com/GnomedDev), [@max-m](https://github.com/max-m), [@whitbur](https://github.com/whitbur), [@HigherOrderLogic](https://github.com/HigherOrderLogic)!

# 0.5.2

New features:
- Added `track_deletion` feature to commands ([35a8209](https://github.com/serenity-rs/poise/commit/35a8209e9e490f795367c34a74cfb18b15b16369))
- Added all of `Context`'s methods to `PrefixContext` and `ApplicationContext` ([c8b1497](https://github.com/serenity-rs/poise/commit/c8b1497123bc4b184a9d581b8ffeb033cb200940))

Behavior changes:
- Editing commands not marked track_edits no longer re-runs the command ([7e7224b](https://github.com/serenity-rs/poise/commit/7e7224bbc063fc1d9408614d6939fe679858a09d))
- `builtins::servers` now shows hidden statistics for the entire bot team, not just owner ([9cb5a77](https://github.com/serenity-rs/poise/commit/9cb5a77589a5255208d2baf3710292eab15802e6))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.1...v0.5.2

Thanks to [@kangalio](https://github.com/kangalio), [@NotNorom](https://github.com/NotNorom)!

# 0.5.1

New features:
- Added `FrameworkOptions::skip_checks_for_owner` ([09d8421](https://github.com/serenity-rs/poise/commit/09d84218861eab17bf62b47ac1e7da1563e36be4))

Behavior changes:
- `execute_modal` doesn't panic anymore when the timeout is reached ([7015c2b](https://github.com/serenity-rs/poise/commit/7015c2bc18b92b791e4da858925bcd4d258a8fa0))
- Checking user permissions properly falls back to HTTP when cache is enabled but empty ([b7a9f1f](https://github.com/serenity-rs/poise/commit/b7a9f1fdb4352b7c10c5f486e6c6055044f0360d))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.5.0...v0.5.1

Thanks to [@peanutbother](https://github.com/peanutbother), [@kangalio](https://github.com/kangalio)!

# 0.5.0

New features:
- Added `Context::parent_commands()` ([cf61765](https://github.com/serenity-rs/poise/commit/cf61765e86e51d9e42074cdb5c544f3820a75c90))
- Added `Context::invocation_string()` ([21db037](https://github.com/serenity-rs/poise/commit/21db037ed17db68b96f5603992be15590e844ae5))
- Added `builtins::register_in_guild()` and `builtins::register_globally()` convenience functions ([e044c9c](https://github.com/serenity-rs/poise/commit/e044c9c93cb5ac7773f48351e5a2cdfffafb40a2))
- The return value of autocomplete callbacks can be any serializable type now ([90ac24a](https://github.com/serenity-rs/poise/commit/90ac24a8ef621ec6dc3fc452762dc9cfa144f693))
- `Context` can now be passed directly into most serenity API functions ([713271b](https://github.com/serenity-rs/poise/commit/713271b76641116a99dd4543a59907d792ce1b5c))
  - Because it now implements `AsRef<Cache>`, `AsRef<Http>`, `AsRef<ShardMessenger>`, and `CacheHttp` traits
- Added `execute_modal()` function with support for modal timeouts ([e7121b6](https://github.com/serenity-rs/poise/commit/e7121b6628b3fe45440a1ed8520ffdf6955463b8))

API updates:
- `Modal::create()` gained a `custom_id: String` parameter ([cee480a](https://github.com/serenity-rs/poise/commit/cee480af9c4a706ea5851b1b26243a55cd0445c5))
  - To make it possible to tell apart two modal interactions
- Removed `CreateReply::reference_message(MessageReference)` in favor of `CreateReply::reply(bool)` ([30ee77b](https://github.com/serenity-rs/poise/commit/30ee77b70dc00b25fb2fe468db03265d3b5d6775))
  - For the unusual case of setting a different reference message than the invocation (why would you? I'm genuinely interested), you can still convert the `CreateReply` into `serenity::CreateMessage` manually via `.to_prefix()` and call `serenity::CreateMessage`'s `reference_message()` method
- Renamed `FrameworkBuilder::user_data_setup()` method to `setup()` ([af099d4](https://github.com/serenity-rs/poise/commit/af099d4052f02e583e705fdb95b2ea3ed0e4bfc9))
- Renamed `FrameworkOptions::listener` field to `event_handler` ([471a2c2](https://github.com/serenity-rs/poise/commit/471a2c2ed8ed3792593dd97c313968266de3183e))
- Renamed `Context::discord()` method to `serenity_context()` ([713271b](https://github.com/serenity-rs/poise/commit/713271b76641116a99dd4543a59907d792ce1b5c))

Behavior changes:
- `register_application_commands_buttons()` now has emojis, reworked wording, and prints the time taken to register ([31318ea](https://github.com/serenity-rs/poise/commit/31318ea1f7484e2c451b049f868a8bd5b15378bd))
- `Modal::execute()` always responds to the correct modal now ([cee480a](https://github.com/serenity-rs/poise/commit/cee480af9c4a706ea5851b1b26243a55cd0445c5))
- When a subcommand is invoked, all parent commands' checks are run too, now ([cceac77](https://github.com/serenity-rs/poise/commit/cceac770b616da33444da022e97cc0631580b773))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.4.1...v0.5.0

Thanks to [@Nilstrieb](https://github.com/Nilstrieb), [@SticksDev](https://github.com/SticksDev), [@p5nvTgip0r](https://github.com/p5nvTgip0r), [@kangalio](https://github.com/kangalio), [@keiveulbugs](https://github.com/keiveulbugs), [@chancedrigor](https://github.com/chancedrigor)!

# 0.4.1

Behavior changes:
- Slash commands marked guild_only don't show up in DMs anymore
  - Using Discord's dm_permission field on commands
- `poise::builtins::servers` now doesn't omit unavailable guilds from guild count and list anymore

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.4.0...v0.4.1

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

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.3.0...v0.4.0

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

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.2.2...v0.3.0

# 0.2.2

Behavior changes:
- `default_member_permissions` fixed to not constrain commands to administrators by default

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.2.1...v0.2.2

# 0.2.1

Behavior changes:
- Bot ID is retrieved from first Ready event again instead of extracted from the token. Extracting the bot ID from the bot token often didn't work and caused breakage for many users. This should now be fixed

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.2.0...v0.2.1

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

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.1.0...v0.2.0

# 0.1.0

Initial crates.io release
