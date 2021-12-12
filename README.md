[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://kangalioo.github.io/poise/)
[![Docs](https://img.shields.io/badge/docs-online-informational)](https://kangalioo.github.io/poise/)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust: 1.51+](https://img.shields.io/badge/rust-1.51+-93450a)](https://blog.rust-lang.org/2020/11/19/Rust-1.51.html)

# Poise
Poise is an opinionated Discord bot framework with a few distinctive features:
- edit tracking: when user edits their message, automatically update bot response 
- slash commands: completely define both normal and slash commands with a single function
- flexible argument parsing: command parameters are defined with normal Rust types and parsed automatically

I created this framework mainly for personal use ([rustbot](<https://github.com/kangalioo/rustbot>)
and [etternabot](<https://github.com/kangalioo/etternabot>)). Features are added on demand, since
it's easy to draft a good design when you know exactly what the feature will be used for.
