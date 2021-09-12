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
        echo "Please visit https://rustup.rs/ install rust, then run the installer again"
        exit
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
    echo "Not yet supported, please install hoard from source"
}

# Set default installation location
HOARD_TARGET_INSTALLATION_DIR="${HOARD_TARGET_INSTALLATION_DIR:-"/usr/local/bin"}/hoard"

## Ask user if they want to install from source
echo "Do you wish to install hoard from source with cargo?"
select yn in "Yes" "No"; do
    case $yn in
        Yes ) __hoard_install_with_cargo ; break;;
        No ) exit;;
    esac
done

case "$OSTYPE" in
    #linux*) __hoard_install_ubuntu ;;
    #linux*) __hoard_install_linux ;;
    #darwin*) __hoard_install_mac ;;
    #*)        __hoard_install_unsupported ;;
esac



#SHELL=$(ps -o comm= -p $$)
#if [ "$SHELL" == "zsh" ]; then
#    echo 'Detected zsh shell'
#    curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.zsh >> ~/.zshrc
#elif [ "$SHELL" == "bash" ]; then
#    echo 'Detected bash shell'
#    curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.bash >> ~/.bashrc
#else
#    echo "$SHELL installation not implemented yet"
#fi
#echo 'source your .bashrc and press <Ctrl-H> to get started with the interactive hoard UI'
