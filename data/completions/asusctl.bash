# Author: AlenPaulVarghese <alenpaul2001@gmail.com>

_asusctl_comp ()
{
    local cur prev

    cur=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}

    case ${COMP_CWORD} in
        1)
            COMPREPLY=($(compgen -W "led-mode profile graphics anime bios --help --version --show-supported \
                            --kbd-supported --fan-mode --chg-limit -h -v -s -k -f -c" -- ${cur}))
            ;;
        2)
            case ${prev} in
                led-mode)
                    COMPREPLY=($(compgen -W "--help --next-mode --prev-mode --awake-enable --sleep-enable -h -n -p -a -s \
                                    static breathe rainbow star rain highlight laser ripple pulse comet flash multi-static multi-breathe" -- ${cur}))
                    ;;
                profile)
                    COMPREPLY=($(compgen -W "--help --next --create --remove --list --active-name --active-data --profile-data \
                                --turbo --min-precentage --max-percentage --fan-preset -h -n -c -r -l -a -A -p -t -m -M -f" -- ${cur}))
                    ;;
                graphics)
                    COMPREPLY=($(compgen -W "--help --mode --get --pow --force -h -m -g -p -f" -- ${cur}))
                    ;;
                anime)
                    COMPREPLY=($(compgen -W "--help --turn --boot leds image -h -t -b" -- ${cur}))
                    ;;
                bios)
                    COMPREPLY=($(compgen -W "--help -h -p -P -d -D" -- ${cur}))
                    ;;
            esac
            ;;
        *)
            COMPREPLY=()
            ;;
    esac
}

complete -F _asusctl_comp asusctl
