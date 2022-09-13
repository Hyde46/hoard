# Release notes

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## 1.1.1

## 1.1.0
- ✨ Named Parameters. You can now add any string after your token `#`. Tokens with the same name will be filled out automatically after being prompted once for them when selecting a command from `hoard list`
## 1.0.1
- ✨ Read out local `trove.yml` file in current directory if available ( Edit ~/.config/hoard/config.yml `read_from_current_directory` to disable )
- ✨ `hoard set_parameter_token` to customize which parameter token is used
- ✨ `hoard info` shows where `config.yml` and `trove.yml` files are located
## 🚀 1.0.0
- ✨ Advanced export allowing subset of namespaces or commands to be exported
- 🐛 Fix bug where selecting a command when running `hoard` as a `zsh` plugin produces gibberish rendered text in the terminal 
- ✨ Customizable GUI colors through ~/config/.hoard/config.yml
- ✨ Support parameterized commands. Put '#' in place where a parameter is expected. When running `hoard pick <command_name>` or `hoard list` as a shell plugin and selecting a parameterized command, `hoard` will ask for all missing parameters to input before sending the complete command to your shell input. 
- ✨ Press `<F1>` when running `hoard list` to see all shortcuts
- ✨ Make tags optional

Breaking changes:
- 🔨 Find your hoard config files at `~/.config/hoard` (Used to be `~/.config/.hoard`. Copy them over or import your old trove file by running `hoard import ~/.config/.hoard/trove.yml`)

## 0.1.8
- Wrap command text in UI
- Left align command text in UI
- combine import url/path into one argument. Use `hoard import <path/url>`
- When picking a command not as a shell plugin, the command will be printed to the console
- `hoard pick` does not require `--name` argument anymore. To print a command run `hoard pick <name>`
- `hoard remove` does not require `--name` argument anymore. To remove a command run `hoard remove <name>`
- `hoard edit` does not require `--name` argument anymore. To remove a command run `hoard edit <name>`
- Add simple export functionality. Run `hoard export /path/to/exported/trove.yml`
- Enable short aliases for basic commands `new[n]`, `list[l]`, `copy[c]`, `pick[p]`, `remove[r]`, `import[i]`,  `export[x]`, `edit[e]`

## 0.1.7
- 🔧 Fix empty defaults for command field inputs

## 0.1.6
- Edit commands with `hoard edit --name <command_name>`
- Ask user for new command name if a collision is detected on creating a new command and importing troves. Name + namespace have to be unique.

## 0.1.5 
- Import other trove files `hoard import --file /path/to/trove.yml`
- Import trove files from url `hoard import --url https://this.trove.com/trove.yml`
- Move config files to `.config/.hoard` instead of `.hoard`

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
