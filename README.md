<p align="center">
<img src=img/hoard_icon.png width="400">
</p>

<p align="center">
  <a href="https://github.com/Hyde46/hoard/actions/workflows/test.yml">
    <img src="https://img.shields.io/github/workflow/status/hyde46/hoard/Rust?style=flat-square" />
  </a>
  <a href="https://crates.io/crates/hoard-rs"
    ><img
      src="https://img.shields.io/crates/v/hoard-rs?style=flat-square"
      alt="Crates.io version"
  /></a>
  <a href="https://github.com/Hyde46/hoard/issues"
    ><img
      src="https://img.shields.io/github/issues/Hyde46/hoard?style=flat-square"
      alt="Crates.io version"
  /></a>
  <a href="https://crates.io/crates/hoard-rs"
    ><img
      src="https://img.shields.io/github/license/Hyde46/hoard?style=flat-square"
      alt="Crates.io version"
  /></a>
</p>

<p align="center">
<em> command organizer tool to hoard all your precious commands 💎🐉</em>
</p>

![Example usage](img/hoard_usage_example.gif)

#### What is a command organizer?

A command organizer lets you save commands that you often use, but are too complicated or long to remember.
For every **hoarded** command, `hoard` saves

- the command ( parameterized with a customizable token, default `#` )
- name
- description
- namespace where it lives in
- tags ( Optional )

If you get lost in your massive command history, and can't find for example a specific `docker` command out of thousand `docker` commands you've already ran,
just **hoard** it. With a **name** and **description** it will be much easier to find again. When you look for that command again a month later, take a look at your **hoarded** commands.

`hoard` is **not** supposed to replace shell history finder like `fzf` `atuin` or similar utilities. It rather should be used in conjunction with them.

## :love_letter: Table of contents

- [📦 Install](#install)
- [🤸 Usage](#usage)
- [:zap: Shortcuts](#shortcuts)

<a name="install"/>

## 📦 Install

### From source

It's best to use [rustup](https://rustup.rs/) to get setup with a Rust
toolchain, then you can run:

```
cargo install hoard-rs
```

Or build it yourself:

```
cargo build --release
```

Find the binaries in `./target/release/hoard`
Move it to wherever you need it ( Like `/usr/local/bin/hoard` )
Or run 
```
cargo install --path .
```

### Linux

Tested on:
- Ubuntu
- EndeavourOS

Install `hoard` by running

```
./install.sh
```

If you are running `fish` shell

```
LATEST_RELEASE=1.3.0 ./install.fish
```

### AUR (Arch Linux)
```
paru -S hoard
```

### Homebrew (macOS)
```
brew tap Hyde46/hoard
brew install hoard
```

### MacPorts (macOS)
```
sudo port install hoard-cli
```

More info [here](https://ports.macports.org/port/hoard-cli/)

### Windows
Best to install from source, good luck

### Install Shell plugin

Install `hoard` as a plugin to enable autocomplete.
Depending on your shell, run one of the following commands.
To keep it installed for your next shell session, add the `source` command with an absolute path to your `.bashrc` or copy-paste the plugins content to your `.bashrc`.

#### bash

```
source src/shell/hoard.bash
```

#### zsh

```
source src/shell/hoard.zsh
```

#### fish

```
source src/shell/hoard.fish
```

### Nix

The hoard package is in the [nixpkgs](https://search.nixos.org/packages?channel=unstable&from=0&size=50&sort=relevance&type=packages&query=hoard) package repository.

Either install it with `nix-env -iA hoard`, get it temporarily with `nix-shell -p hoard` or add it to your configuration.


<a name="usage"/>

## 🤸 Usage

#### Save a new command

```
hoard new
```

If a parameter is not known when saving the command, put a `#` ( Or your customized token from your `~/.config/hoard/config.yml` )
You can also name your parameters like this:
```
echo "My name is #first and I live at #city. Did I tell you my name, #first?" 
```
When putting `#first` you only have to do it once for each occurence in the command.
A parameter name is defined as everyting followed by the token until the first space character is found.
Alternatively you can determine where the named parameter ends by putting a `!` ( Or your customized token from your `~/.config/hoard.config.yml`)
```
echo "My name is #first named parameter! and I live at #city. Did I tell you my name, #first?" 
```
#### Search through command trove

```
<Ctrl-h>
```

Or alternatively, if not installed as a plugin, the interactive search can still be performed, though without autocomplete. This assumes the user to copy the command by mouse from the UI

```
hoard list
```

When running `hoard list` as a shell plugin and selecting a parameterized command, `hoard` will ask for all missing parameters to input before sending the complete command to your shell input. 

If there is a `trove.yml` file present in the local directory, `hoard` will only load this trove file and not display your "global" trove!
( Edit ~/.config/hoard/config.yml `read_from_current_directory` to disable )

#### Synchronize commands with another terminal
You can keep your commands in sync in multiple terminals by using `hoard sync`

First register a new account
```bash
hoard sync register
```

Then login with your new account
```bash
hoard sync login
```

Save your local commands online
```bash
hoard sync save
```

Login with the same account on a different computer/terminal and then run
```bash
hoard sync get
```
This will merge your local trove file with the cloud-based one. You will get prompted how to handle collisions if there are any.
However, if you are unhappy with how the merge happened, you've got the option to revert the latest `hoard sync get` command
```bash
hoard sync revert
```

If you want to host your own sync server, checkout it's [repository](https://github.com/Hyde46/trove_server).
Then, update your config file to point to the new server location `~/.config/hoard/config.yml`

#### Delete a command

```
hoard remove <name>
```

#### Delete all commands in a namespace

```
hoard remove_namespace <namespace_name>
```

#### Edit a command

```
hoard edit <name>
```

#### Info

Shows location of config file and trove file

```
hoard info
```

#### Set parameter token

Change parameter token of config file

```
hoard set_parameter_token <parameter_token>
```

#### Import other trove files from `trove.yml` or urls pointing to a trove.yml file

```
hoard import /path/to/trove.yml
```
or
```
hoard import https://troves.com/new_trove.yml
```

#### Export trove file
```
hoard export /path/to/exported/trove.yml
```

<a name="shortcuts"/>

## :zap: Hoard list shortcuts 

Show list of commands in the GUI
```
<F1>
```

Next item in command list

```
<Ctrl-N> / <Down-Arrow>
```

Previous item in command list

```
<Ctrl-P> / <Ctrl-Y> / <Up-Arrow>
```

Next namespace tab

```
<Ctrl-L> / <Right-Arrow>
```

Previous namespace tab

```
<Ctrl-H> / <Left-Arrow>
```

Select command

```
<Enter>
```

Quit

```
<Esc> / <Ctrl-D> / <Ctrl-C> / <Ctrl-G>
```
