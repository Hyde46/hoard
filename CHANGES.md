# Release notes

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).


## 0.2.0 WIP
- Import other trove files `hoard import --file /path/to/trove.yml`
- Import trove files from url `hoard import --url https://this.trove.com/trove.yml`

## 0.1.4

- Can now delete all commands in a specific namespace `hoard remove --namespace <name>`
- Add 🐟 `fish` shell autocomplete support
- Add 🐟 `fish` installer script

## 0.1.3

- Strip autocompleted command of its leading spaces

## 0.1.2

- 🐛 Fix 'sending on a disconnected channel' bug when autocompleting

## 0.1.1

- Enable installation with cargo
- Generally improve installation script flow

## 0.1.0

- 🚀 Initial release. All basic features compoleted
- Add zsh support for autocomplete (zsh widget)
- Add Linux Ubuntu installer for bash and zsh support

## 0.2.0-beta

- Add bash support for autocomplete
- Rework list UI
- Add namespace tab
- Enable filtering commands in UI
- Replace crossterm UI backend with termion

## 0.1.2-beta

- When starting `hoard` the first time, it asks the user for a default namespace
- `hoard list` won't show UI anymore and break if no command has been saved before
- Show help if no command is supplied

## 0.1.1-beta

- Add command removal command

## 0.1.0-beta

Initial beta release
