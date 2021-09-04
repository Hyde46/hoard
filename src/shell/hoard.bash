__hoard_list ()
{
    ## Thanks github.com/ellie/atuin for the inspiration how to get TUI with termion working as a bash Plugin
	tput rmkx
    HOARD_COMMAND="$(RUST_LOG=error hoard --autocomplete list 3>&1 1>&2 2>&3)"
	tput smkx 

    READLINE_LINE=${HOARD_COMMAND}
    READLINE_POINT=${#READLINE_LINE}
}

if [[ -z $HOARD_NOBIND ]]; then
	bind -x '"\C-h": __hoard_list'
fi
