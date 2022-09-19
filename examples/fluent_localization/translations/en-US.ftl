# Command metadata
welcome = welcome
    .description = Welcomes a user
    .user = user
    .user-description = The user to welcome
    .message = message
    .message-description = The message to send
info = info
    .description = Shows information about the server

# Parameter choices
Ask = Welcome to our cool server! Ask me if you need help
GoodPerson = Welcome to the club, you're now a good person. Well, I hope.
Controller = I hope that you brought a controller to play together!
Coffee = Hey, do you want a coffee?

# The rest
guild-info =
    Name of guild: {$name}
    Has {$emojiCount} guild {$emojiCount ->
        [one] emoji
       *[other] emojis
    }: {$emojis}
    Has {$roleCount} roles
    Has {$stickerCount} custom stickers
not-in-guild-error = Command must be invoked in a guild
