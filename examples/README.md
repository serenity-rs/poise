Here is a curated list of examples that thrive to demonstrate the capabilities of `poise` and how its features are meant to be used.

You must set the following environment variables:
- `DISCORD_TOKEN`: your application's token

Application ID and owner ID don't have to be set, since they are requested from Discord on startup
by poise.

You can start any of the examples by running `cargo run` with the `--example` flag, e.g.:

Unix:

```bash
set DISCORD_TOKEN="your_token"
cargo run --example=framework_usage
```

Windows:

```powershell
$env:DISCORD_TOKEN="your_token"
cargo run --example=framework_usage
```
