conf_default = {$conf_path} is missing! Generating a template ...
conf_old = Old configuration detected, updating ...
edit_conf = You may need to edit {$conf_path} before continuing
empty_list = Empty kernel list
invalid_esp = Invalid ESP_MOUNTPOINT
invalid_index = Invalid kernel index
no_kernel = No kernel found
invalid_kernel_filename = Invalid kernel filename
info_path_not_exist =
    It seems that you have not initialized systemd-boot-friend yet.
    systemd-boot-friend can help you install and configure systemd-boot.
    Simply execute `systemd-boot-friend init`.
err_path_not_exist = {$path} not found
skip_incomplete_kernel = Skipping incomplete kernel {$kernel} ...
skip_unidentified_kernel = Skipping unidentified kernel {$kernel} ...
no_space = No space left on device
edit_bootarg = Please use your favorite text editor to edit `BOOTARG=` entry in {$config}

create_folder = Creating folder structure for friend ...
note_copy_files = Note: systemd-boot-friend will copy Kernel file(s) to your EFI System Partition
install = Installing kernel {$kernel} ...
install_ucode = intel-ucode detected. Installing ...
no_overwrite = Doing nothing on this file.
overwrite = Overwriting {$entry} ...
create_entry = Creating boot entry {$kernel} ...
remove_kernel = Removing kernel {$kernel} ...
remove_entry = Removing boot entry {$kernel} ...
set_default = Setting {$kernel} as default boot entry ...
remove_default = Removing default boot entry {$kernel} ...
init = Installing and initializing systemd-boot ...
prompt_init =
    systemd-boot-friend will now install and initialize systemd-boot, which will
    become the default EFI boot option on your system. If you already have GRUB or
    other bootloaders (such as Windows Boot Manager) installed, they will remain
    accessible from your EFI Boot Manager.
update = Updating boot entries ...
prompt_update =
    Successfully initialized systemd-boot. Would you like systemd-boot-friend to
    search your {$src_path} directory for kernels and register them in systemd-boot
    configuration? If not, you could always do so by running
    `systemd-boot-friend update`.
prompt_empty_bootarg =
    systemd-boot-friend detected an empty `BOOTARG=` field in your configuration.
    This may cause system boot failures.
prompt_current_bootarg = Detected current boot arguments (kernel command line):
prompt_current_root = Detected current root partition: {$root}

ask_overwrite = {$entry} already exists. Overwrite?
ask_set_default = Set {$kernel} as default boot entry?
select_install = Kernel(s) to register as boot entry(s)
select_remove = Kernel(s) to remove from the boot menu
select_default = Default kernel to boot from
ask_init = Proceed with installing and initializing systemd-boot?
ask_update = Proceed with searching and creating boot entries?
ask_empty_bootarg = Generate the boot arguments?
ask_current_bootarg = Use as default systemd-boot boot arguments?
ask_current_root = Use as default systemd-boot boot arguments? (root={$root} rw)
input_timeout = Boot menu timeout (seconds)