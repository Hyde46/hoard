# autocomplete.sh
# should be sourced from ~/.bashrc or something

hoard_complete() {
    #EXPANSION=($(magical_autocomplete $READLINE_LINE))
    #we store the desired value for the line in ${EXPANSION[0]}
   # [[ ${#EXPANSION[@]} -gt 1 ]] && echo ${EXPANSION[@]:1}
    HOARD_COMMAND=$(hoard --autocomplete list)
    READLINE_LINE=${HOARD_COMMAND}
    READLINE_POINT=${#READLINE_LINE}
}
