# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_sbf_global_optspecs
	string join \n h/help V/version
end

function __fish_sbf_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_sbf_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_sbf_using_subcommand
	set -l cmd (__fish_sbf_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c sbf -n "__fish_sbf_needs_command" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_needs_command" -s V -l version -d 'Print version'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "init" -d 'Initialize systemd-boot-friend'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "update" -d 'Install all kernels and update boot entries'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "install-kernel" -d 'Install the kernels specified'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "remove-kernel" -d 'Remove the kernels specified'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "select" -d 'Select kernels to install or remove'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "list-available" -d 'List all available kernels'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "list-installed" -d 'List all installed kernels'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "config" -d 'Configure systemd-boot'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "set-default" -d 'Set the default kernel'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "set-timeout" -d 'Set the boot menu timeout'
complete -c sbf -n "__fish_sbf_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c sbf -n "__fish_sbf_using_subcommand init" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand update" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand install-kernel" -s f -l force -d 'Force overwrite the entry config or not'
complete -c sbf -n "__fish_sbf_using_subcommand install-kernel" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand remove-kernel" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand select" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand list-available" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand list-installed" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand config" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand set-default" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand set-timeout" -s h -l help -d 'Print help'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "init" -d 'Initialize systemd-boot-friend'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "update" -d 'Install all kernels and update boot entries'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "install-kernel" -d 'Install the kernels specified'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "remove-kernel" -d 'Remove the kernels specified'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "select" -d 'Select kernels to install or remove'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "list-available" -d 'List all available kernels'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "list-installed" -d 'List all installed kernels'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "config" -d 'Configure systemd-boot'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "set-default" -d 'Set the default kernel'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "set-timeout" -d 'Set the boot menu timeout'
complete -c sbf -n "__fish_sbf_using_subcommand help; and not __fish_seen_subcommand_from init update install-kernel remove-kernel select list-available list-installed config set-default set-timeout help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
