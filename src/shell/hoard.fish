# Hoard bindings
function __hoard_list
    set hoard_command (hoard --autocomplete list 3>&1 1>&2 2>&3)
    commandline -j $hoard_command
end

if ! set -q HOARD_NOBIND
    bind \ch __hoard_list
end
