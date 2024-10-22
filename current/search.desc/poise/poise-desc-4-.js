searchState.loadedDescShard("poise", 4, "Provide a callback to be invoked before every command. The …\nList of commands in the framework\nCallback to execute when this command is invoked in a …\nContext menu specific name for this command, displayed in …\nConfiguration for the <code>crate::CooldownTracker</code>\nCreate a <code>crate::CooldownContext</code> based off the underlying …\nCreate a <code>crate::CooldownContext</code> based off the underlying …\nCreate a <code>crate::CooldownContext</code> based off the underlying …\nHandles command cooldowns. Mainly for framework internal …\nGenerates a context menu command builder from this <code>Command</code> …\nGenerates a slash command builder from this <code>Command</code> …\nGenerates a slash command parameter builder from this …\nReturn the datetime of the invoking message or interaction\nReturn the datetime of the invoking message or interaction\nReturn the datetime of the invoking message or interaction\nReturns the <code>crate::Context</code> of this error, if it has one\nArbitrary data, useful for storing custom metadata about …\nReturn a reference to your custom user data\nReturn a reference to your custom user data\nReturn a reference to your custom user data\nYour custom user data\nYour custom user data\nYour custom user data\nPermissions which users must have to invoke this command. …\nDefer the response, giving the bot multiple minutes to …\nDefer the response, giving the bot multiple minutes to …\nDefer the response, giving the bot multiple minutes to …\nSee <code>Self::defer()</code>\nSee <code>Self::defer()</code>\nSee <code>Self::defer()</code>\nIf this is an application command, <code>Self::defer()</code> is called\nIf this is an application command, <code>Self::defer()</code> is called\nIf this is an application command, <code>Self::defer()</code> is called\nSee <code>crate::Context::defer()</code>\nShort description of the command. Displayed inline in help …\nDescription of the command. Required for slash commands\nLocalized descriptions with locale string as the key …\nLocalized descriptions with locale string as the key …\nSee <code>Self::serenity_context</code>.\nSee <code>Self::serenity_context</code>.\nSee <code>Self::serenity_context</code>.\nIf true, the command may only run in DMs\nCallback invoked on every message to return a prefix.\nIf Some, the framework will react to message edits by …\nWhether responses to this command should be ephemeral by …\nCalled on every Discord event. Can be used to react to …\nWhether commands in messages emitted by this bot itself …\nIf the user makes a typo in their message and a subsequent …\nReturns a view into data stored by the framework, like …\nReturns a view into data stored by the framework, like …\nReturns a view into data stored by the framework, like …\nUseful if you need the list of commands, for example for a …\nRead-only reference to the framework\nRead-only reference to the framework\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturn the guild of this context, if we are inside a guild.\nReturn the guild of this context, if we are inside a guild.\nReturn the guild of this context, if we are inside a guild.\nReturn the guild channel of this context, if we are inside …\nReturn the guild channel of this context, if we are inside …\nReturn the guild channel of this context, if we are inside …\nReturns the guild ID of this context, if we are inside a …\nReturns the guild ID of this context, if we are inside a …\nReturns the guild ID of this context, if we are inside a …\nID of the guild, if not invoked in DMs\nIf true, only people in guilds may use this command\nCalls the appropriate <code>on_error</code> function (command-specific …\nKeeps track of whether an initial response has been sent.\nMultiline description with detailed usage instructions. …\nWhether to hide this command in help menus.\nReturns serenity’s raw Discord API client to make raw …\nReturns serenity’s raw Discord API client to make raw …\nReturns serenity’s raw Discord API client to make raw …\nReturn a ID that uniquely identifies this command …\nReturn a ID that uniquely identifies this command …\nReturn a ID that uniquely identifies this command …\nA string to identify this particular command within a list …\nWhether to ignore messages from bots for command invoking. …\nWhether to ignore message edits on messages that have not …\nWhether to ignore commands contained within thread …\nIf true, <code>Self::owners</code> is automatically initialized with …\nIf set and <code>Self::initialize_owners</code> is <code>true</code>, the selected …\nThe interaction which triggered this command execution.\nThe type of the interaction which triggered this command …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nAttempts to get the invocation data with the requested type\nAttempts to get the invocation data with the requested type\nAttempts to get the invocation data with the requested type\nCustom user data carried across a single command invocation\nCustom user data carried across a single command invocation\nReturns the string with which this command was invoked.\nReturns the string with which this command was invoked.\nReturns the string with which this command was invoked.\nWhether to rerun the command if an existing invocation …\nReturns the command name that this command was invoked with\nReturns the command name that this command was invoked with\nReturns the command name that this command was invoked with\nCommand name used by the user to invoke this command\nRenamed to <code>Self::event_handler</code>!\nIf available, returns the locale (selected language) of …\nIf available, returns the locale (selected language) of …\nIf available, returns the locale (selected language) of …\nLocalized labels with locale string as the key (slash-only)\nIf <code>true</code>, disables automatic cooldown handling before every …\nTreat a bot mention (a ping) like a prefix\nThe invoking user message\nMain name of the command. Aliases (prefix-only) can be set …\nLabel of this choice\nName of this command parameter\nLocalized names with locale string as the key (slash-only)\nLocalized names with locale string as the key (slash-only)\nCallback for all non-command messages. Useful if you want …\nIf true, the command may only run in NSFW channels\nProvide a callback to be invoked when any user code yields …\nCommand-specific override for …\nUser IDs which are allowed to use owners_only commands\nIf true, only users from the owners list may use this …\nList of parameters for this command\nIf the invoked command was a subcommand, these are the …\nIf the invoked command was a subcommand, these are the …\nIf the invoked command was a subcommand, these are the …\nIf the invoked command was a subcommand, these are the …\nIf the invoked command was a subcommand, these are the …\nReturn the partial guild of this context, if we are inside …\nReturn the partial guild of this context, if we are inside …\nReturn the partial guild of this context, if we are inside …\nReturns the current gateway heartbeat latency (…\nReturns the current gateway heartbeat latency (…\nReturns the current gateway heartbeat latency (…\nCalled after every command if it was successful (returned …\nCalled before every command\nReturns the prefix this command was invoked with, or a …\nReturns the prefix this command was invoked with, or a …\nReturns the prefix this command was invoked with, or a …\nPrefix used by the user to invoke this command\nThe main bot prefix. Can be set to None if the bot …\nCallback to execute when this command is invoked in a …\nPrefix command specific options.\nFull name including parent command names.\nLike <code>Self::say</code>, but formats the message as a reply to the …\nLike <code>Self::say</code>, but formats the message as a reply to the …\nLike <code>Self::say</code>, but formats the message as a reply to the …\nBuilds a <code>crate::CreateReply</code> by combining the builder …\nBuilds a <code>crate::CreateReply</code> by combining the builder …\nBuilds a <code>crate::CreateReply</code> by combining the builder …\nInvoked before every message sent using <code>crate::Context::say</code>…\nIf <code>true</code>, changes behavior of guild_only command check to …\n<code>true</code> is this parameter is required, <code>false</code> if it’s …\nPermissions without which command execution will fail. You …\nPermissions which users must have to invoke this command. …\nRe-runs this entire command invocation\nRe-runs this entire command invocation\nRe-runs this entire command invocation\nAfter the first response, whether to post subsequent …\nShorthand of <code>crate::say_reply</code>\nShorthand of <code>crate::say_reply</code>\nShorthand of <code>crate::say_reply</code>\nShorthand of <code>crate::send_reply</code>\nShorthand of <code>crate::send_reply</code>\nShorthand of <code>crate::send_reply</code>\nReturn the stored <code>serenity::Context</code> within the underlying …\nReturn the stored <code>serenity::Context</code> within the underlying …\nReturn the stored <code>serenity::Context</code> within the underlying …\nReturns the <code>serenity::Context</code> of this error\nSerenity’s context, like HTTP or cache\nSerenity’s context, like HTTP or cache\nSerenity’s context, like HTTP or cache\nStores the given value as the data for this command …\nStores the given value as the data for this command …\nStores the given value as the data for this command …\nIf set to true, skips command checks if command was issued …\nCallback to execute when this command is invoked in a …\nThe name of the <code>#[poise::command]</code>-annotated function\nCallback invoked on every message to strip the prefix off …\nRequire a subcommand to be invoked\nSubcommands of this command, if any\nWhether to delete the bot response if an existing …\nHow this command invocation was triggered\nClosure that sets this parameter’s type and min/max …\nThe serenity Context passed to the event\nThe serenity context passed to the event handler\nGeneral context\nGeneral context\nCommand context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nGeneral context\nSerenity’s Context\nSerenity’s Context\nSerenity’s Context\nDiscord Ready event data present during setup\nDeveloper-readable description of the type mismatch\nError which was thrown in the setup code\nError which was thrown in the event handler code\nError which was thrown in the command code\nError which was thrown by the parameter type’s parsing …\nIf execution wasn’t aborted because of an error but …\nError which was thrown in the dynamic prefix code\nThe error thrown by user code\nWhich event was being processed when the error occurred\nThe Framework passed to the event\nThe Framework passed to the event\nFramework context\nFramework context\nFramework context\nIf applicable, the input on which parsing failed\nThe interaction in question\nSee <code>crate::Context::invocation_data</code>\nWhich permissions in particular the bot is lacking for …\nList of permissions that the user is lacking. May be None …\nMessage which the dynamic prefix callback was evaluated …\nThe message in question\nThe interaction in question\nThe rest of the message (after the prefix) which was not …\nPanic payload which was thrown in the command code\nThe prefix that was recognized\nTime until the command may be invoked for the next time in …\nWhich event triggered the message parsing routine\nStores messages and the associated bot responses in order …\nGiven a message by a user, find the corresponding bot …\nCreate an edit tracker which tracks messages for the …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nRemoves this command invocation from the cache and returns …\nReturns a copy of a newly up-to-date cached message, or a …\nForget all of the messages that are older than the …\nNotify the <code>EditTracker</code> that the given user message should …\nStore that this command is currently running; so that if …")