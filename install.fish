#! /usr/bin/fish

echo "

|_  _  _  _ _|
| |(_)(_|| (_|
              
command organizer                         

hoard setup - fish edition
https://github.com/hyde46/hoard

Please create an issue if you encounter any problems!

===============================================================================
"

function __hoard_install_with_cargo

    if ! command -v cargo &> /dev/null
        echo "cargo not found"
        if command -v rustup &> /dev/null
            echo "rustup was found, but cargo wasn't. Something is up with your installation."
            exit 1
        end


		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -q

        echo "rustup installed!"
    end

    echo "Attempting to install hoard with cargo"

    cargo install hoard-rs
end

function __hoard_install_ubuntu

	set ARTIFACT_URL "https://github.com/hyde46/hoard/releases/download/v$LATEST_VERSION/hoard_$LATEST_VERSION.deb"

    set TEMP_DEB (mktemp)
    wget -O "$TEMP_DEB" "$ARTIFACT_URL"
	sudo dpkg -i "$TEMP_DEB"
	rm -f "$TEMP_DEB"
end

function __hoard_install_mac
    echo "MacOS detected."
    echo "Not yet supported, please install hoard from source with cargo"
end

function __hoard_detect_os

    switch (uname)
        case Linux
            __hoard_install_ubuntu
        case Darwin
            __hoard_install_mac
        case FreeBSD NetBSD DragonFly
            echo "OS not supported yet. Please install from source with cargo"
        case '*'
            echo "OS not supported yet. Please install from source with cargo"
    end

end

function extend_config
    # Setup widget integration
    curl https://raw.githubusercontent.com/Hyde46/hoard/main/src/shell/hoard.fish >> ~/.config/fish/config.fish
    echo 'source your fish.config and press <Ctrl-H> to get started with the interactive hoard UI'
end

function read_input
    while true
        echo "How do you wish to install hoard?"
        echo "(1) From Source with cargo"
        echo "(2) OS-Specifc"
        echo "(3) Not at all, bye"
        read -l -P '> ' confirm
  
        switch $confirm
            case 1
                __hoard_install_with_cargo
                return 0
            case 2
                __hoard_detect_os
                return 0
            case '' 3
                echo "Bye!"
                return 1
        end
    end
end

function extend_fish_config
    while true
        read -l -P "Install fish plugin for autocomplete? This will extend your fish.config by a few lines [y/N] " confirm

        switch $confirm
            case Y y
                extend_config
                return 0
            case '' N n
                return 1
        end
    end
end
# Install hoard
read_input
extend_fish_config
