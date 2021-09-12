# shellcheck disable=SC2034,SC2153,SC2086,SC2155

# Above line is because shellcheck doesn't support zsh, per
# https://github.com/koalaman/shellcheck/wiki/SC1071, and the ignore: param in
# ludeeus/action-shellcheck only supports _directories_, not _files_. So
# instead, we manually add any error the shellcheck step finds in the file to
# the above line ...

# Source this in your ~/.zshrc
autoload -U add-zsh-hook

_hoard_list(){
	emulate -LR zsh
	zle -I

	echoti rmkx
    # Similar to bash plugin in hoard.bash
	# output=$(RUST_LOG=error hoard --autocomplete list 3>&1 1>&2 2>&3)
	output=$(RUST_LOG=error hoard --autocomplete list 3>&1 1>&2 2>&3)
	echoti smkx

	if [[ -n $output ]] ; then
		LBUFFER=$output
	fi

	zle reset-prompt
}

zle -N _hoard_list_widget _hoard_list

if [[ -z $HOARD_NOBIND ]]; then
	bindkey '^h' _hoard_list_widget

	# depends on terminal mode
	#bindkey '^[[A' _hoard_list_widget
	#bindkey '^[OA' _hoard_list_widget
fi
