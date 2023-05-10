Here is a curated list of examples that thrive to demonstrate the capabilities of `poise` and how its features are meant to be used.

You must set the following environment variables:
- `DISCORD_TOKEN`: your application's token

Application ID and owner ID don't have to be set, since they are requested from Discord on startup
by poise.

# basic_structure

Showcases the basics of poise: `FrameworkOptions`, creating and accessing the data struct, a help
command, defining commands and sending responses.

# feature_showcase

Kitchen sink demonstration of most of poise's features. Each file showcases one feature using
one or more example commands.

# fluent_localization

Example implementation of localization how it might be suitable for large-scale bots, using the
[Fluent localization framework](https://projectfluent.org/).

# invocation_data

Small example to test and demonstrate how `Context.invocation_data` flows through the various stages
of a command invocation.

# manual_dispatch

Demonstrates how to circumvent `poise::Framework` and invoke poise's event dispatch functions
manually. This is useful only in special cases.

# quickstart

Contains the quickstart code from the crate root docs. It is stored here so that it can be
automatically tested using `cargo check --example quickstart`.
