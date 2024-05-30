conf_default = {$conf_path} 不存在！正在產生模板...
conf_old = 偵測到舊的設定檔，正在更新...
edit_conf = 在繼續操作前，您可能需要修改 {$conf_path}。
empty_list = 核心列表為空
invalid_esp = ESP_MOUNTPOINT 不正確
invalid_index = 核心編號不正確
no_kernel = 找不到核心
invalid_kernel_filename = 核心檔案名稱不正確
info_path_not_exist =
    systemd-boot-friend 似乎尚未初始化。執行 `systemd-boot-friend init` 即可安裝
    並設定 systemd-boot。
err_path_not_exist = {$path} 不存在
skip_incomplete_kernel = 已跳過不完整的核心 {$kernel} ...
skip_unidentified_kernel = 已跳過不明核心 {$kernel} ...
no_space = 裝置上已無多餘空間
edit_bootarg = 請使用任意文字編輯器編輯 {$config} 中的 `BOOTARG=` 項目
invalid_dirname = 目錄名稱不正確：
require_default = {$conf_path} 中必須包含 "default" （預設）開機引數設定

create_folder = 正在建立 friend 資料夾結構...
note_copy_files = 注意：systemd-boot-friend 將把核心檔案複製到您的 EFI 系統分割區
install = 正在登記核心 {$kernel} ...
install_ucode = 偵測到 intel-ucode。正在登記...
no_overwrite = 檔案未作修改。
overwrite = 正在覆寫 {$entry} ...
create_entry = 正在建立開機選項 {$kernel} ...
remove_kernel = 正在刪除核心 {$kernel} ...
remove_entry = 正在刪除開機選項 {$kernel} ...
set_default = 正在將 {$kernel} 設為預設開機選項...
remove_default = 正在刪除預設開機選項 {$kernel} ...
init = 正在安裝並初始化 systemd-boot ...
notice_init =
    systemd-boot-friend 即將安裝及初始化 systemd-boot，並將其設定為預設 EFI 開機選項。完
    成後，您依舊可以從 EFI 開機管理程式中存取其他開機載入器，如 GRUB 或 Windows開機管理
    器。
update = 正在更新開機選項 ...
skip_update = 您可以隨時執行 `systemd-boot-friend update` 以登記開機選項。
notice_empty_bootarg =
    systemd-boot-friend 在您的設定檔中偵測到了空的 `BOOTARG=` 項目，這有可能導致系統開機
    失敗。
current_bootarg = 偵測到了目前使用的開機引數（核心命令列）：
current_root = 偵測到了目前的根目錄分割區：{$root}
note_list_available = "*" 表示已登記的核心
note_list_installed = "*" 表示預設核心

ask_overwrite = {$entry} 已存在。是否覆寫該檔案？
ask_set_default = 是否將 {$kernel} 設為預設開機選項？
select_install = 要登記開機選項的核心
select_remove = 要從開機選單移除的核心
select_remove = 要在開機選單登記或移除的核心
select_default = 預設核心
ask_init = 是否安裝並初始化 systemd-boot？
prompt_update =
    systemd-boot 已成功初始化。是否要讓 systemd-boot-friend 搜尋 `{$src_path}` 中的核心
    並將其登記至 systemd-boot 設定檔中？
ask_update = 是否安裝所有核心並登記開機選項？
ask_empty_bootarg = 是否自動產生開機引數？
ask_current_bootarg = 是否將上述開機引數設為 systemd-boot 預設開機引數？
ask_current_root = 是否將 `root={$root} rw` 設為 systemd-boot 預設開機引數？
input_timeout = 開機選單顯示時長（秒）
