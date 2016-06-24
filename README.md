# smexybot

<table>
    <tr>
        <td><strong>Linux / OS X</strong></td>
        <td><a href="https://travis-ci.org/indiv0/smexybot" title="Travis Build Status"><img src="https://travis-ci.org/indiv0/smexybot.svg?branch=master" alt="travis-badge"></img></a></td>
    </tr>
    <tr>
        <td colspan="2">
            <a href="https://crates.io/crates/smexybot" title="Crates.io"><img src="https://img.shields.io/crates/v/smexybot.svg" alt="crates-io"></img></a>
            <a href="#License" title="License: MIT/Apache-2.0"><img src="https://img.shields.io/crates/l/smexybot.svg" alt="license-badge"></img></a>
            <a href="https://coveralls.io/github/indiv0/smexybot?branch=master" title="Coverage Status"><img src="https://coveralls.io/repos/github/indiv0/smexybot/badge.svg?branch=master" alt="coveralls-badge"></img></a>
        </td>
    </tr>
</table>

Smexybot is a general-purpose [Discord][discord] bot written in
[Rust][rust-lang]. It is built upon the [tumult][tumult] bot plugin framework,
so Smexybot is easily extensible through the use of plugins.

# Table of Contents

* [Usage](#usage)
* [Commands](#commands)
* [Plugins](#plugins)
* [Contributing](#contributing)
* [Credits](#credits)
* [License](#license)

## Usage
### Invitation

The easiest way to use this bot is to invite it to your Discord server. The bot
is running 24/7 and gets updated as soon as new features are available.

To invite the bot to your server, [click here][bot-invite]. You must have the
**Manage Server** permission for that server. Upon clicking the link, you will
be asked to choose which permissions to give to smexybot, but be aware that
disabling the permissions may lead to unexpected behaviour.

If you have any questions or feedback, feel free to either email me or [open an
issue][new-issue] in this repo. You can also join my personal
[Discord server][discord-invite] and leave a message, or messaging iwearapot on
discord.

Instructions on running a local instance are coming soon.

the global instance (hosted by me)
to your server.

### Personal Setup

Alternatively, a personal instance of the bot can be configured.

Setup a bot account:

1. Go to Discord Developers [My Applications][my-applications] page
2. Click "New Application"
3. Enter a name for your application (this is not your bot username) in the
   "App Name" field
4. (Optional) Add a description and/or icon for your bot (the icon will be your
   bot's profile picture)
5. Click "Create Application"

Create a bot user:

1. On the page for your application, click "Create a Bot User"
2. Click "Yes, do it!" when asked to confirm
3. Now that your bot is created, press "click to reveal" on the application page
   to see your bot's token.
4. Copy this value, and paste it into the file [`.env.example`][env-example]
   (included in this repository), in the place of the existing
   `DISCORD_BOT_TOKEN` value.
5. Copy the `.env.example` file to `.env`, and modify any other configuration
   options you wish.

Run an instance of the bot, either by running a pre-built executable (coming
soon!), or by compiling the bot yourself (instructions coming soon!).

## Configuration

Currently, Smexybot reads its configuration from the environment. It is assumed
that you have then loaded the necessary configuration options into the
environment prior to running the bot.

A set of example configuration options can be found in the file
[`.env.example`][env-example], included in this repository.

Detailed information regarding plugin configurations can be found in the
plugins' respective `README.md` files.

## Commands

Interacting with Smexybot is done via commands. Commands may be performed by DM,
or by `@mention`ing the bot in a text channel on a server on which the bot is
present.

Currently, Smexybot only supports two commands internally. Support for any other
commands is provided by plugins.

### `!test`

Prints a reply text in the text channel. Intended for debugging purposes, mainly
for determing whether or not the bot is reacting to commands.

### `!quit`

Tells to bot to stop monitoring for new events and to logout from the Discord
REST API.

## Plugins

**Coming soon!**

## Contributing

Contributions are always welcome!
If you have an idea for something to add (code, documentation, tests, examples,
etc.) feel free to give it a shot.

Please read [CONTRIBUTING.md][contributing] before you start contributing.

## Credits

Smexybot and it's constituent components and plugins are built primarily using
the excellent [`discord-rs`][discord-rs] framework by
[SpaceManiac][spacemaniac].

The list of contributors to this project can be found at
[CONTRIBUTORS.md][contributors].

## License

smexybot is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE][license-apache], and [LICENSE-MIT][license-mit] for details.

[api-docs]: https://indiv0.github.io/smexybot/smexybot
[bot-invite]: https://discordapp.com/oauth2/authorize?client_id=183278017856929792&scope=bot&permissions=171039744
[contributing]: https://github.com/indiv0/smexybot/blob/master/CONTRIBUTING.md "Contribution Guide"
[contributors]: https://github.com/indiv0/smexybot/blob/master/CONTRIBUTORS.md "List of Contributors"
[discord]: https://discordapp.com/
[discord-invite]: https://discord.gg/qXwhun5
[discord-rs]: https://github.com/SpaceManiac/discord-rs
[env-example]: https://github.com/indiv0/smexybot/blob/master/.env.example
[license-apache]: https://github.com/indiv0/smexybot/blob/master/LICENSE-APACHE "Apache-2.0 License"
[license-mit]: https://github.com/indiv0/smexybot/blob/master/LICENSE-MIT "MIT License"
[my-applications]: https://discordapp.com/developers/applications/me
[new-issue]: https://github.com/indiv0/smexybot/issues/new
[rust-lang]: https://www.rust-lang.org/
[spacemaniac]: https://github.com/SpaceManiac
[tumult]: https://github.com/indiv0/tumult
