[![crates.io badge]][crates.io] [![build badge]][build] [![docs badge]][docs] [![current docs badge]][current docs] [![next docs badge]][next docs] [![msrv badge]][msrv] [![license badge]][license] [![guild badge]][guild]

# Poise
Poise is an opinionated Discord bot framework with a few distinctive features:
- slash commands: completely define slash commands with a single function signature
- flexible argument parsing: command parameters are defined with normal Rust types and parsed automatically
- text commands: commands are agnostic over old text-based commands and slash commands
- edit tracking: when user edits their message, automatically update bot response

# How to use

Most information is in the [API documentation][docs]. Also take a
look at the [examples], especially [`feature_showcase`], to learn what poise can do.

If you're using a development version from git directly, you probably want to look at the documentation for the
[`current`][current docs] or [`next`][next docs] branch instead.

For further questions, don't hesitate to join the [support server][guild].

# Bots using poise

For each bot, there's a list of notable features for you to take inspiration from.

- [Dexscreener Pricebot] by [@keiveulbugs]: embeds, API calls, ephemeral messages
- [TTS Bot] by [@GnomedDev]: localization, database, voice
- [Scripty] by [@tazz4843]: localization, database, voice
- [Etternabot] by [@kangalio]: response transformation, variadic and lazy arguments
- [Ferrisbot] by [@rust-community-discord]: database, custom prefixes
- [StatPixel] by [@matteopolak]: modals, buttons, dropdowns, database, localization
- [CrackTunes] by [@cycle-five]: database, voice, custom prefixes, modals
- [Claude Bot] by [@wyatt-avilla]: database, reactions, slash commands, attachments

You're welcome to add your own bot [via a PR]!

For more projects, see GitHub's [Used by] page.

[examples]: examples
[`feature_showcase`]: examples/feature_showcase
[via a PR]: https://github.com/serenity-rs/poise/compare
[Used by]: https://github.com/serenity-rs/poise/network/dependents

<!-- Bots using poise -->
[Dexscreener Pricebot]: https://github.com/keiveulbugs/Dexscreener_pricebot
[@keiveulbugs]: https://github.com/keiveulbugs
[TTS Bot]: https://github.com/Discord-TTS/Bot/
[@GnomedDev]: https://github.com/GnomedDev
[Scripty]: https://codeberg.org/scripty-bot/scripty
[@tazz4843]: https://github.com/tazz4843
[Etternabot]: https://github.com/kangalio/Etternabot
[@kangalio]: https://github.com/kangalio
[Ferrisbot]: https://github.com/rust-community-discord/ferrisbot-for-discord
[@rust-community-discord]: https://github.com/rust-community-discord
[StatPixel]: https://github.com/statpixel-rs/statpixel
[@matteopolak]: https://github.com/matteopolak
[CrackTunes]: https://github.com/cycle-five/cracktunes
[@cycle-five]: https://github.com/cycle-five
[Claude Bot]: https://github.com/wyatt-avilla/claude-discord-bot
[@wyatt-avilla]: https://github.com/wyatt-avilla

<!-- Badges -->
[crates.io]: https://crates.io/crates/poise
[crates.io badge]: https://img.shields.io/crates/v/poise.svg?style=flat-square
[build]: https://github.com/serenity-rs/poise/actions
[build badge]: https://img.shields.io/github/actions/workflow/status/serenity-rs/poise/ci.yml?branch=current&style=flat-square
[docs]: https://docs.rs/poise/
[docs badge]: https://img.shields.io/badge/docs-online-informational?style=flat-square
[current docs]: https://serenity-rs.github.io/poise/current/poise/index.html
[current docs badge]: https://img.shields.io/badge/docs-current-4d76ae.svg?style=flat-square
[next docs]: https://serenity-rs.github.io/poise/next/poise/index.html
[next docs badge]: https://img.shields.io/badge/docs-next-4d76ae.svg?style=flat-square
[msrv]: https://blog.rust-lang.org/2023/11/16/Rust-1.74.0.html
[msrv badge]: https://img.shields.io/badge/rust-1.74+-93450a.svg?style=flat-square
[license]: LICENSE
[license badge]: https://img.shields.io/crates/l/poise.svg?style=flat-square&color=yellow
[guild]: https://discord.gg/serenity-rs
[guild badge]: https://img.shields.io/discord/381880193251409931.svg?style=flat-square&colorB=7289DA
