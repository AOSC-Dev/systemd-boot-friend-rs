conf_default = {$conf_path} 不存在！建立默認配置檔中...
empty_list = 列表為空
invalid_esp = 非法的 ESP_MOUNTPOINT
invalid_num = 非法的內核序號
no_kernel = 找不到內核
invalid_kernel_filename = 非法的內核檔案名
info_path_not_exist = {$path} 不存在。無事可做。
    如果您想使用 systemd-boot，請首先執行 `systemd-boot-friend init`。
    如果您的 EFI 掛載點不在 {$esp}，請編輯 {$conf}。
err_path_not_exist = {$path} 不存在

initialize = 初始化 systemd-boot 中...
create_folder = 建立 friend 資料夾結構中...
install = 安裝 {$kernel} 至 {$path} 中...
install_ucode = 偵測到 intel-ucode。安裝中...
ask_overwrite = {$entry} 已存在。是否覆寫？
no_overwrite = 檔案未作修改。
overwrite = 覆寫 {$entry} 中...
create_entry = 於 {$path} 建立 {$kernel} 啟動項目中...
remove_kernel = 刪除 {$kernel} 內核檔案中...
remove_entry = 刪除 {$kernel} 啟動項目中...