# GSC Driver Stutter Fix — NUC X15

**Language / Язык:** [🇬🇧 English](#-english) · [🇷🇺 Русский](#-русский)

---

<a id="english"></a>

# 🇬🇧 English

> **Important:** This guide addresses audio crackling and image freezes caused by the driver for `Graphics System Controller Firmware Interface` (GSC). Verified on LAPAC71H/LAPAC71G with Arc A730M and Arc A550M.

**Contents:**
- [Warning](#warning)
- [Step 1: Download DDU](#step-1-download-ddu-display-driver-uninstaller)
- [Step 2: Download driver 31.0.101.5590](#step-2-download-driver-3101015590)
- [Step 3: Remove old drivers with DDU](#step-3-remove-old-drivers-with-ddu)
- [Step 4: Install driver 31.0.101.5590](#step-4-install-driver-3101015590)
- [Step 5: Update GPU drivers to latest](#step-5-update-gpu-drivers-to-the-latest-version)
- [Step 6: Roll back the GSC driver](#step-6-roll-back-the-gsc-driver)
- [Step 7: Block Windows from updating GSC](#step-7-block-windows-from-updating-the-gsc-driver)
- [Verification](#how-to-verify-the-fix-worked)
- [FAQ](#faq)

## Warning

The following steps are **not mandatory**, but are recommended for safety:

- **Backup:** Save your important data before you start.
- **BitLocker:** If enabled, **find your recovery key**. You may need it when booting into Safe Mode.
- **Internet:** Disconnect from the internet (Wi-Fi/Ethernet) during the process to prevent Windows from auto-updating drivers.
- **Password:** Make sure you remember your account password (not the PIN), as Windows 11 Safe Mode may ask for it.

## Step 1: Download DDU (Display Driver Uninstaller)

1. Go to the official website: **https://www.wagnardsoft.com**.
2. Find **Display Driver Uninstaller (DDU)** and download the **Portable / Self-Extracting** version.
3. Extract the archive to a convenient folder.

## Step 2: Download driver 31.0.101.5590

1. Open the download page: https://www.comss.ru/download/page.php?id=13872  
   *(Intel no longer hosts old drivers on their site, unfortunately.)*
2. Download `Intel-Graphics-Driver-31.0.101.5590.exe` (~866 MB).
3. **Do not install it yet** — first remove the old drivers.

## Step 3: Remove old drivers with DDU

1. **Disconnect from the internet.**
2. Run **DDU.exe**.
3. Select:
   - **Device type:** GPU
   - **Select device:** Intel
4. Click **Clean and restart**.
5. The PC will reboot into **Safe Mode** and clean Intel drivers.
6. **Do not reconnect to the internet** until you install the new driver.

## Step 4: Install driver 31.0.101.5590

1. Run the downloaded installer.
2. Follow the instructions.
3. Select **"Clean Install"** if prompted.
4. Reboot.

> ✅ **Check:** Device Manager → "System devices" → "Intel(R) Graphics System Controller Firmware Interface". Version should be `31.0.101.5590`.

## Step 5: Update GPU drivers to the latest version

1. Reconnect to the internet.
2. Install the latest Intel Arc/Iris Xe driver: https://www.intel.com/content/www/us/en/download/785597/intel-arc-graphics-windows.html
3. Reboot.

## Step 6: Roll back the GSC driver

1. Open **Device Manager** (Win + X → Device Manager).
2. Expand **"System devices"**.
3. Find **"Intel(R) Graphics System Controller Firmware Interface"**.
4. Right-click → **"Update driver"**.
5. Select **"Browse my computer for drivers"**.
6. Click **"Let me pick from a list of available drivers"**.
7. Select version **`31.0.101.5590`**.
8. Click **"Next"** and wait.
9. Reboot (optional).

> ✅ **Check:** Event Viewer → Windows Logs → System. Filter by source `GSCx64`. No events should be present.

## Step 7: Block Windows from updating the GSC driver

### Method 1: Group Policy (recommended)

1. Press **Win + R**, type `gpedit.msc`, Enter.
2. Go to: **Computer Configuration → Administrative Templates → Windows Components → Windows Update → Manage updates offered from Windows Update**.
3. Find **"Do not include drivers with Windows Updates"**.
4. Set to **"Enabled"** → **OK**.
5. Reboot.

### Method 2: System Settings

1. Press **Win + R**, type `sysdm.cpl`, Enter.
2. **"Hardware"** tab → **"Device Installation Settings"**.
3. Select **"No (your device might not work as expected)"**.
4. Save.

## How to verify the fix worked

1. Open **Event Viewer** → Windows Logs → System.
2. Filter by source `GSCx64`.
3. No events (or only events from before the rollback).
4. Run a game / heavy app for 10–15 minutes.
5. No audio crackling, no image freezes → fixed.

## FAQ

**Driver `31.0.101.5590` doesn't appear in the list?**
- Make sure you downloaded the correct version.
- Try: "Update driver" → "Browse my computer" → "Let me pick from a list" → "Have Disk" → point to the driver folder.

**Stutter doesn't go away after rollback?**
- The issue might be caused by other factors (thermals, power, RAM).
- Try disabling the GSC device in Device Manager (temporary workaround).

**Is it safe to disable GSC?**
- GSC handles GPU firmware updates and some security tasks. Disabling removes stutter but blocks firmware updates.

**Why does Windows 11 24H2 update GSC?**
- Windows 11 24H2 force-updates all drivers, including GSC. Group Policy blocks this.

## Note

After a major Windows update, the GSC driver may be replaced again. Repeat **Step 6** to roll it back. Windows keeps old driver files on disk.

---

<a id="-русский"></a>

# 🇷🇺 Русский

> **Важно:** Эта инструкция решает проблему с прерыванием звука и фризами изображения, вызванную драйвером `Graphics System Controller Firmware Interface` (GSC). Проверено на LAPAC71H/LAPAC71G с Arc A730M и Arc A550M.

**Содержание:**
- [Предупреждение](#предупреждение)
- [Шаг 1: Скачай DDU](#шаг-1-скачай-ddu-display-driver-uninstaller)
- [Шаг 2: Скачай драйвер 31.0.101.5590](#шаг-2-скачай-драйвер-3101015590)
- [Шаг 3: Удали старые драйверы через DDU](#шаг-3-удали-старые-драйверы-через-ddu)
- [Шаг 4: Установи драйвер 31.0.101.5590](#шаг-4-установи-драйвер-3101015590)
- [Шаг 5: Обнови драйверы видеокарты](#шаг-5-обнови-драйверы-видеокарты-до-актуальной-версии)
- [Шаг 6: Замени драйвер для GSC](#шаг-6-замени-драйвер-для-gsc-на-старый)
- [Шаг 7: Запрети Windows обновлять GSC](#шаг-7-запрети-windows-обновлять-драйвер-gsc)
- [Проверка](#как-проверить-что-проблема-решена)
- [FAQ](#частые-вопросы)

## Предупреждение

Следующие шаги **не обязательны**, но рекомендуются для безопасности:

- **Резервная копия:** Сохрани важные данные.
- **BitLocker:** Если включён, **найди ключ восстановления**.
- **Интернет:** Отключи интернет на время работы.
- **Пароль:** Помни пароль от учётной записи (не PIN).

## Шаг 1: Скачай DDU (Display Driver Uninstaller)

1. Открой **https://www.wagnardsoft.com**.
2. Скачай **Portable / Self-Extracting** версию.
3. Распакуй архив.

## Шаг 2: Скачай драйвер 31.0.101.5590

1. Открой https://www.comss.ru/download/page.php?id=13872  
   *(Intel больше не хранит старые драйверы на своём сайте.)*
2. Скачай `Intel-Graphics-Driver-31.0.101.5590.exe` (~866 МБ).
3. **Пока не устанавливай.**

## Шаг 3: Удали старые драйверы через DDU

1. **Отключи интернет.**
2. Запусти **DDU.exe**.
3. Выбери:
   - **Device type:** GPU
   - **Select device:** Intel
4. Нажми **Clean and restart**.
5. ПК перезагрузится в **безопасном режиме**.
6. **Не подключай интернет**, пока не установишь новый драйвер.

## Шаг 4: Установи драйвер 31.0.101.5590

1. Запусти скачанный установщик.
2. Следуй инструкциям.
3. Выбери **Clean Install**, если предложат.
4. Перезагрузи ПК.

> ✅ **Проверка:** Диспетчер устройств → «Системные устройства» → «Intel(R) Graphics System Controller Firmware Interface». Версия — `31.0.101.5590`.

## Шаг 5: Обнови драйверы видеокарты до актуальной версии

1. Подключи интернет.
2. Установи последнюю версию: https://www.intel.com/content/www/us/en/download/785597/intel-arc-graphics-windows.html
3. Перезагрузи.

## Шаг 6: Замени драйвер для GSC на старый

1. Открой **Диспетчер устройств**.
2. Разверни **«Системные устройства»**.
3. Найди **«Intel(R) Graphics System Controller Firmware Interface»**.
4. ПКМ → **«Обновить драйвер»**.
5. **«Найти драйверы на этом компьютере»**.
6. **«Выбрать драйвер из списка доступных»**.
7. Выбери версию **`31.0.101.5590`**.
8. **«Далее»** и дождись установки.
9. Перезагрузи (опционально).

> ✅ **Проверка:** Просмотр событий → Журналы Windows → Система. Фильтр по `GSCx64`. Событий быть не должно.

## Шаг 7: Запрети Windows обновлять драйвер GSC

### Способ 1: Групповая политика (рекомендуется)

1. **Win + R** → `gpedit.msc` → Enter.
2. **Конфигурация компьютера → Административные шаблоны → Компоненты Windows → Windows Update → Управление предложениями обновлений Windows**.
3. Найди **«Не включать драйверы в обновления Windows»**.
4. **Включено** → **OK**.
5. Перезагрузи.

### Способ 2: Параметры системы

1. **Win + R** → `sysdm.cpl` → Enter.
2. Вкладка **«Оборудование»** → **«Параметры установки устройств»**.
3. **«Нет»**.
4. Сохрани.

## Как проверить, что проблема решена

1. Просмотр событий → Журналы Windows → Система.
2. Фильтр по `GSCx64`.
3. Событий нет.
4. Запусти игру на 10–15 минут.
5. Нет треска и фризов → решено.

## Частые вопросы

**Драйвер `31.0.101.5590` не отображается в списке?**
- Убедись, что скачал правильную версию.
- Попробуй: «Обновить драйвер» → «Найти на этом компьютере» → «Выбрать из списка» → «Установить с диска» → укажи папку.

**Статтеры не исчезли?**
- Возможны другие причины (нагрев, питание, RAM).
- Попробуй отключить устройство GSC в Диспетчере устройств (временно).

**Безопасно ли отключать GSC?**
- GSC отвечает за прошивку и безопасность. Отключение убирает статтеры, но блокирует обновление прошивки.

**Почему Windows 11 24H2 обновляет GSC?**
- Windows 11 24H2 принудительно обновляет все драйверы. Блокировка через групповую политику предотвращает это.

---

## Примечание

После крупного обновления Windows драйвер GSC может снова обновиться. Повтори **Шаг 6**. Windows сохраняет старые файлы драйверов.