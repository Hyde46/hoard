#! /usr/bin/env bash

set -euo pipefail

cat << EOF

|_  _  _  _ _|
| |(_)(_|| (_|
              
command organizer                         

hoard setup
https://github.com/hyde46/hoard

Please create an issue if you encounter any issues!

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

    cargo install hoard-rs
}

__hoard_install_ubuntu(){
	echo "Ubuntu detected"
	ARTIFACT_URL="https://github.com/hyde46/hoard/releases/download/v$LATEST_VERSION/hoard_${LATEST_VERSION}.deb"

	TEMP_DEB="$(mktemp)" &&
	wget -O "$TEMP_DEB" "$ARTIFACT_URL"
	sudo dpkg -i "$TEMP_DEB"
	rm -f "$TEMP_DEB"
}

__hoard_install_mac(){
    echo "MacOS detected."
    echo "Not yet supported, please install hoard from source with cargo"
}

__hoard_detect_os(){

    case "$OSTYPE" in
        linux*) __hoard_install_ubuntu ;;
        #linux*) __hoard_install_linux ;;
        darwin*) __hoard_install_mac ;;
        #*)        __hoard_install_unsupported ;;
    esac
}

# Set default installation location
HOARD_TARGET_INSTALLATION_DIR="${HOARD_TARGET_INSTALLATION_DIR:-"/usr/local/bin"}/hoard"


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

SHELL=$(ps -o comm= -p $$)
if [ "$SHELL" == "zsh" ]; then
    echo 'Detected zsh shell'
    curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.zsh >> ~/.zshrc
elif [ "$SHELL" == "bash" ]; then
    echo 'Detected bash shell'
    curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.bash >> ~/.bashrc
else
    echo "$SHELL installation not implemented yet"
    echo "Plese create an issue on github to get this sorted out"
    echo "https://github.com/Hyde46/hoard/issues/new?title=Unsupported%20shell%20detected%20$SHELL&body=$SHELL%20is%20not%20supported.%20Please%20take%20a%20look"
fi
echo 'source your .bashrc and press <Ctrl-H> to get started with the interactive hoard UI'
