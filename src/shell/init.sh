ws() {
    if [ "$1" = "switch" ]; then
        local dir
        dir="$(command ws "$@")"
        if [ $? -eq 0 ] && [ -n "$dir" ] && [ -d "$dir" ]; then
            cd "$dir" || return 1
        else
            printf '%s\n' "$dir" >&2
            return 1
        fi
    else
        command ws "$@"
    fi
}

if [ -n "$ZSH_VERSION" ]; then
    _ws() {
        local -a subcmds
        subcmds=(
            'new:Create a new workspace'
            'switch:Switch to an existing workspace'
            'list:List all workspaces'
            'ls:List all workspaces'
            'remove:Remove a workspace and its worktrees'
            'rm:Remove a workspace and its worktrees'
            'template:Manage templates'
            'config:Show or update configuration'
            'shell-init:Print shell integration'
        )

        local -a template_subcmds
        template_subcmds=(
            'list:List all templates'
            'add:Add a new template or add repos to one'
            'remove:Remove a template or repos from one'
            'show:Show details of a template'
        )

        _arguments -C \
            '1:command:->command' \
            '*::arg:->args'

        case "$state" in
            command)
                _describe 'ws command' subcmds
                ;;
            args)
                case "${words[1]}" in
                    switch)
                        if (( CURRENT == 2 )); then
                            local -a workspaces
                            workspaces=(${(f)"$(command ws complete workspaces 2>/dev/null)"})
                            _describe 'workspace' workspaces
                        fi
                        ;;
                    new)
                        _arguments \
                            '-t[Template to use]:template:->template' \
                            '--template[Template to use]:template:->template' \
                            '1:name:'
                        if [ "$state" = "template" ]; then
                            local -a templates
                            templates=(${(f)"$(command ws complete templates 2>/dev/null)"})
                            _describe 'template' templates
                        fi
                        ;;
                    remove|rm)
                        _arguments \
                            '-t[Template to use]:template:->template' \
                            '--template[Template to use]:template:->template' \
                            '1:name:->workspace'
                        case "$state" in
                            workspace)
                                local -a workspaces
                                workspaces=(${(f)"$(command ws complete workspaces 2>/dev/null)"})
                                _describe 'workspace' workspaces
                                ;;
                            template)
                                local -a templates
                                templates=(${(f)"$(command ws complete templates 2>/dev/null)"})
                                _describe 'template' templates
                                ;;
                        esac
                        ;;
                    template)
                        if (( CURRENT == 2 )); then
                            _describe 'template command' template_subcmds
                        else
                            case "${words[2]}" in
                                add)
                                    _arguments \
                                        '1:template name:' \
                                        '*'{-r,--repo}'[Repo path]:repo:_directories'
                                    ;;
                                remove)
                                    _arguments \
                                        '1:template name:->tmpl' \
                                        '*'{-r,--repo}'[Repo path]:repo:_directories'
                                    if [ "$state" = "tmpl" ]; then
                                        local -a templates
                                        templates=(${(f)"$(command ws complete templates 2>/dev/null)"})
                                        _describe 'template' templates
                                    fi
                                    ;;
                                show)
                                    if (( CURRENT == 3 )); then
                                        local -a templates
                                        templates=(${(f)"$(command ws complete templates 2>/dev/null)"})
                                        _describe 'template' templates
                                    fi
                                    ;;
                            esac
                        fi
                        ;;
                    config)
                        _arguments \
                            '--workspace-dir[Set workspace directory]:dir:_directories'
                        ;;
                esac
                ;;
        esac
    }
    compdef _ws ws
elif [ -n "$BASH_VERSION" ]; then
    _ws_bash() {
        local cur prev subcmds
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        subcmds="new switch list ls remove rm template config shell-init"

        if [ "$COMP_CWORD" -eq 1 ]; then
            COMPREPLY=($(compgen -W "$subcmds" -- "$cur"))
            return
        fi

        case "${COMP_WORDS[1]}" in
            switch)
                if [ "$COMP_CWORD" -eq 2 ]; then
                    local workspaces
                    workspaces="$(command ws complete workspaces 2>/dev/null)"
                    COMPREPLY=($(compgen -W "$workspaces" -- "$cur"))
                fi
                ;;
            remove|rm)
                if [ "$COMP_CWORD" -eq 2 ]; then
                    local workspaces
                    workspaces="$(command ws complete workspaces 2>/dev/null)"
                    COMPREPLY=($(compgen -W "$workspaces" -- "$cur"))
                elif [ "$prev" = "-t" ] || [ "$prev" = "--template" ]; then
                    local templates
                    templates="$(command ws complete templates 2>/dev/null)"
                    COMPREPLY=($(compgen -W "$templates" -- "$cur"))
                fi
                ;;
            new)
                if [ "$prev" = "-t" ] || [ "$prev" = "--template" ]; then
                    local templates
                    templates="$(command ws complete templates 2>/dev/null)"
                    COMPREPLY=($(compgen -W "$templates" -- "$cur"))
                fi
                ;;
            template)
                if [ "$COMP_CWORD" -eq 2 ]; then
                    COMPREPLY=($(compgen -W "list add remove show" -- "$cur"))
                elif [ "$prev" = "-r" ] || [ "$prev" = "--repo" ]; then
                    COMPREPLY=($(compgen -d -- "$cur"))
                elif [ "$COMP_CWORD" -eq 3 ]; then
                    case "${COMP_WORDS[2]}" in
                        show|remove)
                            local templates
                            templates="$(command ws complete templates 2>/dev/null)"
                            COMPREPLY=($(compgen -W "$templates" -- "$cur"))
                            ;;
                    esac
                fi
                ;;
        esac
    }
    complete -F _ws_bash ws
fi
