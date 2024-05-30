conf_default = {$conf_path} 不存在！正在生成配置样例...
conf_old = 检测到旧的配置文件，正在更新...
edit_conf = 在继续操作前，您可能需要修改 {$conf_path}
empty_list = 内核列表为空
invalid_esp = ESP_MOUNTPOINT 不正确
invalid_index = 内核序号不正确
no_kernel = 找不到内核
invalid_kernel_filename = 内核文件名不正确
info_path_not_exist =
    systemd-boot-friend 似乎尚未初始化。运行 `systemd-boot-friend init` 即可安装并配
    置 systemd-boot。
err_path_not_exist = {$path} 不存在
skip_incomplete_kernel = 已跳过不完整的内核 {$kernel} ...
skip_unidentified_kernel = 已跳过不明内核 {$kernel} ...
no_space = 设备上没有空间
edit_bootarg = 请使用任意文本编辑器编辑 {$config} 中的 `BOOTARG=` 条目
invalid_dirname = 目录名不正确：
require_default = {$conf_path} 中必须包含 "default" （默认）启动参数配置

create_folder = 正在建立 friend 目录结构...
note_copy_files = 注意：systemd-boot-friend 将把内核文件复制到您的 EFI 系统分区
install = 正在登记内核 {$kernel} ...
install_ucode = 检测到 intel-ucode。正在登记...
no_overwrite = 文件未作修改。
overwrite = 正在覆盖 {$entry} ...
create_entry = 正在建立启动项 {$kernel} ...
remove_kernel = 正在移除内核 {$kernel} ...
remove_entry = 正在移除启动项 {$kernel} ...
set_default = 正在将 {$kernel} 设为默认启动项...
remove_default = 正在移除启动项 {$kernel} ...
init = 正在安装并初始化 systemd-boot ...
notice_init =
    systemd-boot-friend 即将安装及初始化 systemd-boot，并将其设置为默认 EFI 启动项。完成
    后，您依旧可以从 EFI 启动管理器中访问其他启动引导器，如 GRUB 或 Windows 启动管理器。
update = 正在更新启动项 ...
skip_update = 您可以随时运行 `systemd-boot-friend update` 来登记启动项。
notice_empty_bootarg =
    systemd-boot-friend 在您的配置中检测到了空的 `BOOTARG=` 条目，这有可能导致系统启
    动失败。
current_bootarg = 检测到了当前使用的启动参数（内核命令行）：
current_root = 检测到了当前的根目录分区： {$root}
note_list_available = "*" 表示已登记的内核
note_list_installed = "*" 表示默认内核

ask_overwrite = {$entry} 已存在。是否覆盖该文件？
ask_set_default = 是否将 {$kernel} 设为默认启动项？
select_install = 要登记启动项的内核
select_remove = 要从启动菜单移除的内核
select_remove = 要在启动菜单登记或移除的内核
select_default = 默认内核
ask_init = 是否安装并初始化 systemd-boot-friend？
prompt_update =
    systemd-boot 已成功初始化。是否要让 systemd-boot-friend 搜索 `{$src_path}` 中的内核
    并将其登记至 systemd-boot 配置中？
ask_update = 是否安装所有内核并登记启动项？
ask_empty_bootarg = 是否自动生成启动参数？
ask_current_bootarg = 是否将上述启动参数设为 systemd-boot 默认启动参数？
ask_current_root = 是否将 `root={$root} rw` 设为 systemd-boot 默认启动参数？
input_timeout = 启动菜单显示时长（秒）
