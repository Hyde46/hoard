#! /usr/bin/env bash

set -euo pipefail

cat << EOF

|_  _  _  _ _|
| |(_)(_|| (_|
              
command organizer                         

hoard setup
https://github.com/hyde46/hoard

Please create an issue if you encounter any problems!

===============================================================================

EOF

__hoard_install_with_cargo(){
    echo "Building from source..."
    
    ## Check if cargo is installed
    if ! command -v cargo &> /dev/null
    then
        echo "cargo not found"
		if command -v rustup &> /dev/null
		then
			echo "rustup was found, but cargo wasn't. Something is up with your installtion"
			exit 1
		fi

		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -q

		echo "rustup installed! Attempting cargo install"
    fi

}

__hoard_install_ubuntu(){
	echo "Assuming Ubuntu distro. Trying to install .deb package"
	ARTIFACT_URL="https://github.com/hyde46/hoard/releases/download/v0.1.8/hoard_0.1.8.deb"

	TEMP_DEB="$(mktemp)" &&
	wget -O "$TEMP_DEB" "$ARTIFACT_URL"
	sudo dpkg -i "$TEMP_DEB"
	rm -f "$TEMP_DEB"
}

__hoard_install_mac(){
    echo "MacOS detected."
    echo "Not yet supported by installer.sh"
    echo "please install hoard from source with cargo or with brew"
    echo "brew tap add Hyde46/hoard"
    echo "brew install hoard"
    echo "To install as zshell plug, run:"
    echo 'curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.zsh >> ~/.zshrc'
}

__hoard_detect_os(){

    case "$OSTYPE" in
        linux*) __hoard_install_ubuntu ;;
        #linux*) __hoard_install_linux ;;
        darwin*) __hoard_install_mac ;;
        #*)        __hoard_install_unsupported ;;
    esac
}

PS3='How do you wish to install hoard:  '
foods=("From Source with cargo" "OS-Specifc" "Not at all, bye")
select fav in "${foods[@]}"; do
    case $fav in
        "From Source with cargo")
            __hoard_install_with_cargo
            break
            ;;
        "OS-Specifc")
            __hoard_detect_os
            break
            ;;
	"Not at all, bye")
        echo "Bye!"
	    break
	    ;;
        *) echo "invalid option $REPLY";;
    esac
done

# TODO: Properly check which shell is installed
curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.zsh >> ~/.zshrc
curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.bash >> ~/.bashrc
echo 'source your .bashrc/.zshrc and press <Ctrl-H> to get started with the interactive hoard UI'
