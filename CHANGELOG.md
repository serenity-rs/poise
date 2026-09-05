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
- Find: (?<=Thanks to.*)(?<!\[)@([a-z0-9-]+)
- Replace: [@$1](https://github.com/$1)
-->

# 0.7.0

New features:
- Added configurable limits on prefix command edit tracking to prevent unintended command invocation ([6ea8d6d](https://github.com/serenity-rs/poise/commit/6ea8d6dbf74d496a261d212794ab8dc60dc4d77d))
  - `PrefixFrameworkOptions::check_edits_against_cache` (default: `true`) determines whether edits are checked against the cached message, when available.
  - `PrefixFrameworkOptions::tracking_initiation_window` (default: 15 min.) defines a window starting from message creation during which edit tracking is active.

API updates:
- Removed redundant `serenity_context` and user `data` fields from `Context` ([6ea554d](https://github.com/serenity-rs/poise/commit/6ea554dc8fc9acb95240b68309d3075e3d13f2aa))
  - Serenity context can be accessed via `Context::serenity_context()`/`FrameworkContext::serenity_context`.
  - User data can be accessed via `Framework::user_data()`/`FrameworkContext::user_data()`/`FrameworkContext::user_data`.
- `reply::CreateReply` fields are now private ([ead8cb1](https://github.com/serenity-rs/poise/commit/ead8cb1eb186e7aeb1bdb6a753e2b62656414e30))
- `PrefixFrameworkOptions::prefix` now takes `Cow<'static, str>` instead of `String` ([621bf48](https://github.com/serenity-rs/poise/commit/621bf48c53e8929fb05850e874eea4142798d97c))
- `modal::execute_modal_on_component_interaction()` now takes `&serenity::Context` instead of `impl AsRef<serenity::Context>` ([8f45e5d](https://github.com/serenity-rs/poise/commit/8f45e5dfb023631ec002514cb0ea88918dc3b050))
- Removed magic from autocomplete ([cefd69d](https://github.com/serenity-rs/poise/commit/cefd69d0cf7b7f92d25c2a9a525b9b557efe12ca))
  - Autocomplete functions now must return `serenity::CreateAutocompleteResponse`.
- Removed `builtins::help()`, `builtins::pretty_help()`, and `help_text_fn` ([cc20f24](https://github.com/serenity-rs/poise/commit/cc20f24e8f982ebecba40f39313c5ca7ffe78d91))
- Inline choice parameters are now converted to `String` at compile time ([63c89c6](https://github.com/serenity-rs/poise/commit/63c89c61c3b43120415676e9d6e69f9bef11a84d))
  - This makes inline choices slightly more restrictive, as byte string literals, nul-terminated C-string literals, and byte literals are no longer supported.
- `builtins::create_application_commands()`, `builtins::register_globally()`, and `builtins::register_in_guild()` now take `impl IntoIterator<Item = &Command<U, E>>` instead of `&[Command<U, E>]` ([d7c5599](https://github.com/serenity-rs/poise/commit/d7c55997b7f558415d36c089f8b21b0bd1f5502f))
- Removed specialization hacks for `PopArgument` and `SlashArgument` ([2d3816c](https://github.com/serenity-rs/poise/commit/2d3816c64323789e97fa689fb459144c7d3af02a))
  - Types that do not implement `PopArgument` but implement `FromStr` must now be decorated with a new `#[string]` attribute, or users can implement the `PopArgument` trait manually.
- Added support for the `#[string]` attribute to slash commands ([fbf89f1](https://github.com/serenity-rs/poise/commit/fbf89f167f9044516d6259240958ce760b45522a))
  - Types that do not implement `SlashArgument` but implement `FromStr` must now be decorated with a new `#[string]` attribute, or users can implement the `SlashArgument` trait manually.
- Restored support for `Option<T: FromStr>` arguments to prefix commands using the `#[string]` attribute ([ff6c74b](https://github.com/serenity-rs/poise/commit/ff6c74b99db1c38c023e264b44786a22827ea102))
- Removed redundant fields from `Context` and `FrameworkError` to reduce their sizes ([83d2040](https://github.com/serenity-rs/poise/commit/83d20405312b480ac1003684eb5afd510ef3aa54))
  - Removed `Context::command` fields. Use `Context::command()` instead.
  - Removed `Context::parent_commands` fields. Use `Context::parent_commands()` instead.
  - New `Context::command_tree`/`Context::command_tree()` now contains the full invoked command tree (parents + invoked command).
  - Removed `PrefixContext::action` field. Use `Command::prefix_action` instead.
  - Removed `PrefixContext::prefix` field. Use `Context::prefix()` instead.
  - Removed `prefix` and `msg_content` from `FrameworkError::UnknownCommand`, which already includes the full `Message` object. A new `content_start` field gives the position in the message that the prefix ends, allowing the user to determine the prefix and message content, if necessary.

Behavior changes:
- Permission checks have been rewritten ([fd83067](https://github.com/serenity-rs/poise/commit/fd83067615d6a9d5557f2e3f48ac3361c63fd62d))
  - Previously, when the bot permissions were unknown, the command would run anyway. Now, `FrameworkError::PermissionFetchFailed` is returned.
- Prefix commands no longer panic when `serenity::Message::author_permissions()` cannot find a channel while calculating permissions ([50c2130](https://github.com/serenity-rs/poise/commit/50c2130ed4a70bd694f0f6253d9bd5c7a8967c59))
- `builtins::on_error()` now shows the full error source chain ([3c01afc](https://github.com/serenity-rs/poise/commit/3c01afc5d064c39c57aa93a88e1fca0dda6339f3))
- Permissions calculations now properly handle `SEND_MESSAGES_IN_THREADS` ([fa332af](https://github.com/serenity-rs/poise/commit/fa332af323e45d1f13f9fb6c91f9cd0f0f01da66))
- Interaction type is now checked in command matching ([6347d12](https://github.com/serenity-rs/poise/commit/6347d124e35db0911e0c44c27a5cb048444deb61))
  - This fixes a bug that prevented different command types (`USER`/`MESSAGE`/`CHAT_INPUT`) from having the same name.
- `Modal::execute_with_defaults()` now properly handles a defined default value of `None` ([26dbb0d](https://github.com/serenity-rs/poise/commit/26dbb0dd5f9d4cdfa398f45c36f655e55c2e6fa9))
  - This fixes a bug that would cause an `Invalid Form Body` error from Discord when a default of `None` was defined for an `Option<String>` parameter.

Miscellaneous:
- Switched from `String`/`Vec` to `Cow<'static, str>`/`Cow<'static, [T]>` in `structs::Command` and related structs ([a917c4c](https://github.com/serenity-rs/poise/commit/a917c4c23550d9ecd796749d61eef2793990f4b6))
  - This avoids allocations and simplifies command construction, but does not impact modifying the values at runtime.
- Removed the `parse_prefix_args!` and `parse_slash_args!` declarative macros ([40e99ec](https://github.com/serenity-rs/poise/commit/40e99ec1ce35cb480527548be60f4011d115e3a0))
  - Their functionality is now directly incorporated into the `poise::command` proc macro, which makes things more readable and maintainable, and opens the door for added features down the line.
- Fixed several parsing bugs introduced when specialization hacks were removed ([303023a](https://github.com/serenity-rs/poise/commit/303023a92609872add5c415fddbc1046289d77f4))
- Fixed exponential code generation for prefix commands with many `Option<T>` parameters ([8d7bb43](https://github.com/serenity-rs/poise/commit/8d7bb43f5bcbd34fcaf0acb331568952489881ae))
- Switched from the unmaintained `derivative` crate to `derive_where` ([e9ffb58](https://github.com/serenity-rs/poise/commit/e9ffb581a059bec6de40fcd910a45fecc603a87b))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.6.2...v0.7.0

Thanks to [@arqunis](https://github.com/arqunis), [@GnomedDev](https://github.com/GnomedDev), [@jamesbt365](https://github.com/jamesbt365), [@zkxs](https://github.com/zkxs), [@mkrasnitski](https://github.com/mkrasnitski), [@inklesspen](https://github.com/inklesspen), [@SabrinaJewson](https://github.com/SabrinaJewson), and [@meditationmind](https://github.com/meditationmind)!

# 0.6.2

New features:
- Added handler for non-command messages ([1f0abfb](https://github.com/serenity-rs/poise/commit/1f0abfbc4c6d79af62867ff94fe79f5ae4fe6f8f))
- Added `builtins::pretty_help`, a help command that uses embeds ([ec19915](https://github.com/serenity-rs/poise/commit/ec19915d817cc6ad8f02ec0cab260d29d2704cce))
- Added support for modifying the last invocation of a command ([c67dde5](https://github.com/serenity-rs/poise/commit/c67dde58e2a185193738b30f2b1e8600dcf391cd))
  - This makes it possible to shorten or lengthen a cooldown after invocation.
- Added ability to only initialize certain roles to owners ([b887224](https://github.com/serenity-rs/poise/commit/b887224bbb4c3a9ad3196d3d58aba782ca3de909))
  - This makes it possible to specify `TeamMemberRole`s other than the default `Admin` and `Developer` to be initialized to owners.
- Added support for editing initial interaction response attachments ([525bd69](https://github.com/serenity-rs/poise/commit/525bd69b7f91dacdc8eb1da33558102f2f1d8d19))
- Added support for manual cooldowns on single commands ([7ae055c](https://github.com/serenity-rs/poise/commit/7ae055c95c38432ac7f0622c24d4b36d9d2970aa))
- Added support for polls ([3eefed4](https://github.com/serenity-rs/poise/commit/3eefed431125398677a6d81fc27a7df85c196f06))
- Added support for user-installed apps, aka user apps ([773da6c](https://github.com/serenity-rs/poise/commit/773da6cbe78dfab81d631879466c47616df17548))

Behavior changes:
- The pound sign (#) is now supported in language identifiers (e.g., `c#`, `f#`) in code blocks ([59dba56](https://github.com/serenity-rs/poise/commit/59dba560f9de22c2e6123369b1b52d56cad82a84))
- Fixed event handler example to remove a race condition and prevent the bot from replying to itself ([9def484](https://github.com/serenity-rs/poise/commit/9def4848ee9958f4b4bcc6ddbf2937717a762062), [48b0318](https://github.com/serenity-rs/poise/commit/48b03181e6d0f604caac829852f64a986ee5f3a2))
- Disabled allowed mentions for `Command` and `ArgumentParse` errors ([6ead1e1](https://github.com/serenity-rs/poise/commit/6ead1e1962efdfa4c5dec764e7df79694ace35f3))
  - This prevents these errors from unintentionally pinging users.
- Fixed `additional_prefixes` usage in the basic structure example ([8ba38c0](https://github.com/serenity-rs/poise/commit/8ba38c04a471fac1f00c43fb2635e3253f43a816))
- Snowflake ID arguments to prefix commands are now parsed as mentions where appropriate ([bbc837a](https://github.com/serenity-rs/poise/commit/bbc837a1dd170d6ebb5c9208d7a9fd8b3dc27a27))
- Updated examples to use `tracing-subscriber` instead of `env_logger` ([db10b12](https://github.com/serenity-rs/poise/commit/db10b126c8b07f7e1924cba1672f04ff24ed4ec7))
- Context menu commands now properly check `default_member_permissions` ([e850875](https://github.com/serenity-rs/poise/commit/e850875cf3925786502d9f126c85f8c61f89ace2))
  - Previously, the builder failed to set `default_member_permissions` when creating a context menu command.
- Command description length is now determined based on `char`s instead of bytes ([e144ede](https://github.com/serenity-rs/poise/commit/e144ede7a6e1f29b018ed1982af797cf020371be))
  - This resolves an artifical constraint on languages that use multibyte (CJK) characters. Further discussion in [#379](https://github.com/serenity-rs/poise/pull/379).
- The resolved `User` returned by a user context menu command now includes `PartialMember` data when available ([4c7661d](https://github.com/serenity-rs/poise/commit/4c7661d120451d199e62fa3987fb9d0676c8cbb7))
- `Command` and `ArgumentParse` error replies are now ephemeral ([24fe146](https://github.com/serenity-rs/poise/commit/24fe1469d0b110385a85df7263c0bc547d823800))

Miscellaneous:
- Reduced generated code for field conversions in macros ([5b369bb](https://github.com/serenity-rs/poise/commit/5b369bbafde38f74670681dc9f4b55430712a4ff))
- Bumped Rust edition to 2021 and added edition to `.rustfmt.toml` ([1c34184](https://github.com/serenity-rs/poise/commit/1c3418473636a3e3648e36b2e852bb6f4d9e7993), [f1e79b5](https://github.com/serenity-rs/poise/commit/f1e79b5409a2234529053c683387f523ec5667d9))

Detailed changelog: https://github.com/serenity-rs/poise/compare/v0.6.1...v0.6.2

Thanks to [@kangalio](https://github.com/kangalio), [@emilyyyylime](https://github.com/emilyyyylime), [@GnomedDev](https://github.com/GnomedDev), [@fee1-dead](https://github.com/fee1-dead), [@asibahi](https://github.com/asibahi), [@c-git](https://github.com/c-git), [@Spacerulerwill](https://github.com/Spacerulerwill), [@matteopolak](https://github.com/matteopolak), [@jamesbt365](https://github.com/jamesbt365), [@NotNorom](https://github.com/NotNorom), [@keiveulbugs](https://github.com/keiveulbugs), [@arqunis](https://github.com/arqunis), [@ravener](https://github.com/ravener), [@DocJade](https://github.com/DocJade), [@fgardt](https://github.com/fgardt), [@cycle-five](https://github.com/cycle-five), [@Nydauron](https://github.com/Nydauron), [@zkxs](https://github.com/zkxs), [@yuimarudev](https://github.com/yuimarudev), [@meditationmind](https://github.com/meditationmind), [@nwerosama](https://github.com/nwerosama), [@black-sock](https://github.com/black-sock), [@TapGhoul](https://github.com/TapGhoul), and [@HactarCE](https://github.com/HactarCE)!

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
