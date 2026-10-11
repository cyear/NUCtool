
(() => {
    "use strict";

    const PAGE_ID = "page-bios";

    const state = {
        initialized: false,
        rawText: "",
        entries: [],
        filteredEntries: [],
        selectedId: null,
        nextId: 1,
        loading: false,
    };

    const $ = (id) => document.getElementById(id);

    function escapeHtml(value) {
        return String(value ?? "").replace(/[&<>"']/g, (char) => ({
            "&": "&amp;",
            "<": "&lt;",
            ">": "&gt;",
            '"': "&quot;",
            "'": "&#39;",
        })[char]);
    }

    function normalizeText(value) {
        return String(value ?? "").replace(/\r\n?/g, "\n");
    }

    function stripComment(value) {
        return String(value ?? "").replace(/\s*\/\/.*$/, "").trim();
    }

    function setStatus(message, type = "normal") {
        const status = $("bios-status");
        const text = $("bios-status-text");

        if (!status || !text) return;

        status.dataset.state = type;
        text.textContent = message;
    }

    function setLoading(loading) {
        state.loading = loading;

        const button = $("bios-refresh-btn");

        if (button) {
            button.disabled = loading;
            button.innerHTML = loading
                ? '<span class="bios-btn-icon" aria-hidden="true">…</span> 正在读取 BIOS'
                : '<span class="bios-btn-icon" aria-hidden="true">↻</span> <span data-i18n="bios.readBios">读取 BIOS</span>';
        }

        if (loading) {
            setStatus("正在读取 BIOS NVRAM，请稍候……", "loading");
        }
    }

    function getInvoke() {
        if (typeof window.__TAURI__?.core?.invoke === "function") {
            return window.__TAURI__.core.invoke;
        }

        if (typeof window.__TAURI_INTERNALS__?.invoke === "function") {
            return window.__TAURI_INTERNALS__.invoke;
        }

        throw new Error("未找到 Tauri invoke 接口，请检查 Tauri 前端 API 配置。");
    }

    function getField(raw, names) {
        const lines = normalizeText(raw).split("\n");

        for (const name of names) {
            const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
            const pattern = new RegExp(
                "^\\s*" + escaped + "\\s*(?:=|:)\\s*(.*?)\\s*$",
                "i"
            );

            for (const line of lines) {
                if (/^\s*\/\//.test(line)) continue;

                const match = line.match(pattern);

                if (match) {
                    return stripComment(match[1]);
                }
            }
        }

        return "";
    }

    function getQuestion(raw) {
        const lines = normalizeText(raw).split("\n");

        for (const line of lines) {
            if (/^\s*\/\//.test(line)) continue;

            const match = line.match(
                /^\s*Setup Question\s*(?:=|:)\s*(.*?)\s*$/i
            );

            if (match) {
                return stripComment(match[1]) || "未命名配置项";
            }
        }

        return "未命名配置项";
    }

    function extractOptionsFromLine(line) {
        let text = String(line ?? "").trim();

        if (!text) return [];

        text = stripComment(text);

        if (!text) return [];

        const options = [];
        const pattern =
            /(\*)?\s*\[([^\]]+)\]\s*([^\[]*?)(?=\s*\*?\s*\[|$)/g;

        let match;

        while ((match = pattern.exec(text)) !== null) {
            const value = match[2].trim();
            const label = match[3].trim();

            if (!value) continue;

            options.push({
                value,
                label: label || value,
                selected: Boolean(match[1]),
            });

            if (match[0].length === 0) {
                pattern.lastIndex += 1;
            }
        }

        if (options.length > 0) {
            return options;
        }

        const pairPattern =
            /(?:^|\s)(\*)?\s*([^\s=:[\]]+)\s*[=:]\s*(.+?)\s*$/;

        const pair = text.match(pairPattern);

        if (pair) {
            return [{
                value: pair[2].trim(),
                label: pair[3].trim() || pair[2].trim(),
                selected: Boolean(pair[1]),
            }];
        }

        return [];
    }

    function normalizeOptionValue(value) {
        const text = String(value ?? "").trim();
        const match = text.match(/^\[([^\]]+)\]/);

        return match ? match[1].trim() : text;
    }

    function parseOptions(raw) {
        const lines = normalizeText(raw).split("\n");
        const options = [];
        let readingOptions = false;

        const knownFieldPattern =
            /^\s*(?:Setup Question|Question|Help String|Help|Token|Offset|Width|BIOS Default|Default|Value|Current Value|Options)\s*(?:=|:)/i;

        for (const originalLine of lines) {
            const trimmed = originalLine.trim();

            if (!trimmed || /^\s*\/\//.test(originalLine)) {
                continue;
            }

            const optionsMatch = originalLine.match(
                /^\s*Options\s*(?:=|:)\s*(.*?)\s*$/i
            );

            if (optionsMatch) {
                readingOptions = true;

                const inlineText = stripComment(optionsMatch[1]);

                if (inlineText) {
                    options.push(...extractOptionsFromLine(inlineText));
                }

                continue;
            }

            if (!readingOptions) continue;

            if (knownFieldPattern.test(originalLine)) {
                readingOptions = false;
                continue;
            }

            options.push(...extractOptionsFromLine(originalLine));
        }

        const unique = [];
        const indexByValue = new Map();

        for (const option of options) {
            const existingIndex = indexByValue.get(option.value);

            if (existingIndex === undefined) {
                indexByValue.set(option.value, unique.length);
                unique.push(option);
            } else if (option.selected) {
                unique[existingIndex] = {
                    ...unique[existingIndex],
                    selected: true,
                };
            }
        }

        return unique;
    }

    function parseNVRAM(text) {
        const normalized = normalizeText(text).replace(/^\uFEFF/, "");

        if (!normalized.trim()) {
            throw new Error("NVRAM 返回内容为空。");
        }

        const lines = normalized.split("\n");
        const starts = [];

        for (let i = 0; i < lines.length; i += 1) {
            if (/^\s*\/\//.test(lines[i])) continue;

            if (/^\s*Setup Question\s*(?:=|:)\s*.*$/i.test(lines[i])) {
                starts.push(i);
            }
        }

        console.log("[BIOS] Normalized line count:", lines.length);
        console.log("[BIOS] Valid question count:", starts.length);

        if (starts.length === 0) {
            throw new Error(
                "未找到 Setup Question 配置区块，请检查 NVRAM 导出格式。"
            );
        }

        const entries = [];

        for (let i = 0; i < starts.length; i += 1) {
            const start = starts[i];
            const end = i + 1 < starts.length ? starts[i + 1] : lines.length;
            const raw = lines.slice(start, end).join("\n").trim();

            if (!raw) continue;

            const question = getQuestion(raw);
            const help = getField(raw, ["Help String", "Help"]);
            const token = getField(raw, ["Token"]);
            const offset = getField(raw, ["Offset"]);
            const width = getField(raw, ["Width"]);
            const defaultValue = getField(raw, ["BIOS Default", "Default"]);
            const exportedValue = getField(raw, ["Value", "Current Value"]);
            const options = parseOptions(raw);

            let initialValue = exportedValue;
            let valueSource = "exported";

            if (!initialValue && options.length > 0) {
                const selectedOption = options.find((option) => option.selected);

                if (selectedOption) {
                    initialValue = selectedOption.value;
                    valueSource = "selected-option";
                }
            }

            if (!initialValue && defaultValue) {
                initialValue = normalizeOptionValue(defaultValue);
                valueSource = "bios-default";
            }

            entries.push({
                id: state.nextId++,
                question,
                help,
                token,
                offset,
                width,
                defaultValue,
                exportedValue,
                originalValue: initialValue,
                currentValue: initialValue,
                valueSource,
                options,
                raw,
            });
        }

        if (entries.length === 0) {
            throw new Error("没有解析出有效的 BIOS 配置记录。");
        }

        return entries;
    }

    function isModified(entry) {
        return entry.currentValue !== entry.originalValue;
    }

    function getSelectedEntry() {
        return state.entries.find(
            (entry) => entry.id === state.selectedId
        ) || null;
    }


    function buildModifiedNvramText() {
        const originalText = String(state.rawText ?? "");

        if (!originalText.trim()) {
            throw new Error("没有 NVRAM 原始数据，请先读取 BIOS。");
        }

        const normalized = normalizeText(originalText);
        const lines = normalized.split("\n");
        const starts = [];

        for (let i = 0; i < lines.length; i += 1) {
            if (/^\s*\/\//.test(lines[i])) continue;

            if (/^\s*Setup Question\s*(?:=|:)/i.test(lines[i])) {
                starts.push(i);
            }
        }

        if (starts.length !== state.entries.length) {
            throw new Error(
                `配置项数量不一致：原始文本 ${starts.length} 项，解析记录 ${state.entries.length} 项。已取消导出。`
            );
        }

        for (let i = 0; i < starts.length; i += 1) {
            const start = starts[i];
            const end = i + 1 < starts.length ? starts[i + 1] : lines.length;
            const entry = state.entries[i];

            if (!isModified(entry)) continue;

            // 查找明确的 Value / Current Value 字段。
            let valueLine = -1;

            for (let j = start; j < end; j += 1) {
                if (/^\s*\/\//.test(lines[j])) continue;

                if (/^\s*(?:Value|Current Value)\s*(?:=|:)/i.test(lines[j])) {
                    valueLine = j;
                    break;
                }
            }

            if (valueLine !== -1) {
                const match = lines[valueLine].match(
                    /^(\s*)(Value|Current Value)(\s*(?:=|:)\s*)(.*)$/i
                );

                if (!match) {
                    throw new Error(
                        `配置项「${entry.question}」的 Value 字段格式无法识别。`
                    );
                }

                const oldContent = match[4];
                const commentMatch = oldContent.match(/(\s*\/\/.*)$/);
                const comment = commentMatch ? commentMatch[1] : "";

                lines[valueLine] =
                    `${match[1]}${match[2]}${match[3]}${entry.currentValue}${comment}`;

                console.log("[BIOS] Updated value field:", {
                    question: entry.question,
                    value: entry.currentValue,
                });

                continue;
            }

            // 没有 Value 字段时，尝试修改 Options 中的 * 选中标记。
            if (entry.options.length > 0) {
                const targetValue = normalizeOptionValue(entry.currentValue);

                const targetOptionExists = entry.options.some(
                    (option) => normalizeOptionValue(option.value) === targetValue
                );

                if (!targetOptionExists) {
                    throw new Error(
                        `配置项「${entry.question}」的修改值「${entry.currentValue}」不在选项列表中，无法安全导出。`
                    );
                }

                let readingOptions = false;
                let selectedOptionFound = false;

                for (let j = start; j < end; j += 1) {
                    const line = lines[j];

                    if (/^\s*\/\//.test(line)) continue;

                    const optionsMatch = line.match(
                        /^(\s*Options\s*(?:=|:)\s*)(.*)$/i
                    );

                    if (optionsMatch) {
                        readingOptions = true;

                        const updated = updateOptionLine(
                            optionsMatch[2],
                            targetValue
                        );

                        lines[j] = optionsMatch[1] + updated.text;
                        selectedOptionFound ||= updated.selected;

                        continue;
                    }

                    if (!readingOptions) continue;

                    // 后续字段开始，选项区结束。
                    if (/^\s*(?:Setup Question|Question|Help String|Help|Token|Offset|Width|BIOS Default|Default|Value|Current Value)\s*(?:=|:)/i.test(line)) {
                        readingOptions = false;
                        continue;
                    }

                    const updated = updateOptionLine(line, targetValue);

                    lines[j] = updated.text;
                    selectedOptionFound ||= updated.selected;
                }

                if (!selectedOptionFound) {
                    throw new Error(
                        `配置项「${entry.question}」未能在原始 Options 文本中定位目标选项。`
                    );
                }

                console.log("[BIOS] Updated option selection:", {
                    question: entry.question,
                    value: entry.currentValue,
                });

                continue;
            }

            throw new Error(
                `配置项「${entry.question}」既没有 Value 字段，也没有可用的 Options 列表，无法安全导出。`
            );
        }

        return lines.join("\n");
    }

    /**
     * 修改一行中的选项标记。
     *
     * 支持：
     * Options = *[00]Disabled // comment
     *          *[01]Enabled
     *          [01]Enabled
     *
     * 保留选项文字、缩进和行内注释。
     */
    function updateOptionLine(line, targetValue) {
        const pattern = /(\*)?\s*(\[[^\]]+\])/g;

        let match;
        let result = "";
        let lastIndex = 0;
        let selected = false;

        while ((match = pattern.exec(line)) !== null) {
            const bracketValue = match[2].slice(1, -1).trim();
            const isTarget = normalizeOptionValue(bracketValue) === targetValue;

            result += line.slice(lastIndex, match.index);

            if (isTarget) {
                result += "*";
                selected = true;
            }

            result += match[2];
            lastIndex = pattern.lastIndex;
        }

        result += line.slice(lastIndex);

        return {
            text: result,
            selected,
        };
    }


    function updateButtonStates() {
        const hasData = state.entries.length > 0;

        [
            "bios-export-raw-btn",
            "bios-export-json-btn",
            "bios-reset-btn",
            "bios-copy-source-btn",
        ].forEach((id) => {
            const button = $(id);
            if (button) button.disabled = !hasData || state.loading;
        });

        const resetEntryButton = $("bios-detail-reset-btn");
        const selected = getSelectedEntry();

        if (resetEntryButton) {
            resetEntryButton.disabled =
                !selected || !isModified(selected) || state.loading;
        }

        const writeButton = $("bios-write-btn");

        if (writeButton) {
            writeButton.disabled = !hasData || state.loading;
        }
    }

    function updateStats() {
        const total = state.entries.length;

        const withOptions = state.entries.filter(
            (entry) => entry.options.length > 0
        ).length;

        const modified = state.entries.filter(isModified).length;

        if ($("bios-stat-total")) {
            $("bios-stat-total").textContent = String(total);
        }

        if ($("bios-stat-options")) {
            $("bios-stat-options").textContent = String(withOptions);
        }

        if ($("bios-stat-modified")) {
            $("bios-stat-modified").textContent = String(modified);
        }

        if ($("bios-stat-visible")) {
            $("bios-stat-visible").textContent =
                String(state.filteredEntries.length);
        }

        if ($("bios-results-label")) {
            $("bios-results-label").textContent =
                `${state.filteredEntries.length} 条匹配记录`;
        }

        if ($("bios-footer-count")) {
            $("bios-footer-count").textContent =
                `${state.filteredEntries.length} / ${total} 条记录`;
        }

        updateButtonStates();
    }

    function applyFilters() {
        const search = ($("bios-search")?.value || "").trim().toLowerCase();
        const filter = $("bios-filter")?.value || "all";

        state.filteredEntries = state.entries.filter((entry) => {
            const searchable = [
                entry.question,
                entry.help,
                entry.token,
                entry.offset,
                entry.width,
                entry.defaultValue,
                entry.exportedValue,
                entry.originalValue,
                entry.currentValue,
                ...entry.options.map(
                    (option) => `${option.value} ${option.label}`
                ),
            ].join(" ").toLowerCase();

            if (search && !searchable.includes(search)) return false;

            if (filter === "options" && entry.options.length === 0) {
                return false;
            }

            if (filter === "no-options" && entry.options.length > 0) {
                return false;
            }

            if (filter === "modified" && !isModified(entry)) {
                return false;
            }

            return true;
        });

        renderTable();
        updateStats();

        const showAll = $("bios-show-all-btn");

        if (showAll) {
            showAll.hidden =
                state.filteredEntries.length === state.entries.length;
        }
    }

    function renderTable() {
        const tbody = $("bios-table-body");
        if (!tbody) return;

        if (!state.filteredEntries.length) {
            tbody.innerHTML = `
                <tr class="bios-empty-row">
                    <td colspan="7">
                        <div class="bios-empty">
                            <span class="bios-empty-symbol" aria-hidden="true">⌕</span>
                            <strong>没有匹配的配置项</strong>
                            <p>请尝试更换搜索词或筛选条件。</p>
                        </div>
                    </td>
                </tr>
            `;
            return;
        }

        tbody.innerHTML = state.filteredEntries.map((entry) => {
            const selected = entry.id === state.selectedId;
            const modified = isModified(entry);
            const value = entry.currentValue || "—";

            return `
                <tr
                    class="bios-data-row${selected ? " is-selected" : ""}${modified ? " is-modified" : ""}"
                    data-entry-id="${entry.id}"
                    tabindex="0"
                    aria-selected="${selected ? "true" : "false"}"
                >
                    <td>${state.entries.indexOf(entry) + 1}</td>
                    <td class="bios-question-cell">
                        <span class="bios-question-name">${escapeHtml(entry.question)}</span>
                        ${entry.help
                    ? `<span class="bios-question-help">${escapeHtml(entry.help)}</span>`
                    : ""}
                    </td>
                    <td><code>${escapeHtml(entry.token || "—")}</code></td>
                    <td><code>${escapeHtml(entry.offset || "—")}</code></td>
                    <td>${escapeHtml(entry.width || "—")}</td>
                    <td class="bios-value-cell">${escapeHtml(value)}</td>
                    <td>
                        ${modified
                    ? '<span class="bios-state-tag is-modified">已修改</span>'
                    : '<span class="bios-state-tag">原始值</span>'}
                    </td>
                </tr>
            `;
        }).join("");
    }

    function renderDetails() {
        const entry = getSelectedEntry();
        const empty = $("bios-detail-empty");
        const content = $("bios-detail-content");

        if (!empty || !content) return;

        if (!entry) {
            empty.hidden = false;
            content.hidden = true;

            if ($("bios-detail-index")) {
                $("bios-detail-index").textContent = "未选择配置项";
            }

            return;
        }

        empty.hidden = true;
        content.hidden = false;

        const index = $("bios-detail-index");

        if (index) {
            index.textContent =
                `配置项 ${state.entries.indexOf(entry) + 1} / ${state.entries.length}`;
        }

        if ($("bios-detail-question")) {
            $("bios-detail-question").textContent = entry.question;
        }

        if ($("bios-detail-help")) {
            $("bios-detail-help").textContent = entry.help || "没有帮助文本。";
        }

        if ($("bios-detail-token")) {
            $("bios-detail-token").textContent = entry.token || "—";
        }

        if ($("bios-detail-offset")) {
            $("bios-detail-offset").textContent = entry.offset || "—";
        }

        if ($("bios-detail-width")) {
            $("bios-detail-width").textContent = entry.width || "—";
        }

        if ($("bios-detail-default")) {
            $("bios-detail-default").textContent = entry.defaultValue || "—";
        }

        if ($("bios-detail-raw")) {
            $("bios-detail-raw").textContent = entry.raw;
        }

        const modifiedBadge = $("bios-detail-modified");

        if (modifiedBadge) {
            modifiedBadge.hidden = !isModified(entry);
        }

        renderEditor(entry);
        renderOptions(entry);
        updateButtonStates();
    }

    function renderEditor(entry) {
        const container = $("bios-editor-control");
        if (!container) return;

        if (entry.options.length > 0) {
            const currentExists = entry.options.some(
                (option) => option.value === entry.currentValue
            );

            const optionsHtml = entry.options.map((option) => {
                const selected = option.value === entry.currentValue;

                return `
                    <option
                        value="${escapeHtml(option.value)}"
                        ${selected ? "selected" : ""}
                    >${escapeHtml(option.label)}</option>
                `;
            }).join("");

            container.innerHTML = `
                <select id="bios-detail-value" aria-label="本地值预览">
                    ${currentExists ? "" : `
                        <option value="${escapeHtml(entry.currentValue)}" selected>
                            ${escapeHtml(entry.currentValue || "(空值)")}
                        </option>
                    `}
                    ${optionsHtml}
                </select>
            `;
        } else {
            container.innerHTML = `
                <input
                    id="bios-detail-value"
                    type="text"
                    value="${escapeHtml(entry.currentValue)}"
                    aria-label="本地值预览"
                    autocomplete="off"
                />
            `;
        }

        const control = $("bios-detail-value");

        if (!control) {
            console.error("[BIOS] 编辑控件没有生成，请检查 bios-editor-control 元素。");
            return;
        }

        control.addEventListener("change", () => {
            entry.currentValue = control.value;
            applyFilters();
            renderDetails();
        });

        if (control.tagName === "INPUT") {
            control.addEventListener("input", () => {
                entry.currentValue = control.value;

                const badge = $("bios-detail-modified");

                if (badge) {
                    badge.hidden = !isModified(entry);
                }

                updateStats();
                renderTable();
            });
        }

        const hint = $("bios-editor-hint");

        if (hint) {
            if (entry.options.length > 0) {
                hint.textContent =
                    "选择选项以更新本地预览，不会直接写入 BIOS。";
            } else {
                hint.textContent =
                    "没有解析到可用选项列表。编辑仅影响本地预览。";
            }

            if (entry.valueSource === "selected-option") {
                hint.textContent +=
                    " 初始值根据导出文本中标记的选项推断。";
            } else if (entry.valueSource === "bios-default") {
                hint.textContent +=
                    " 初始值来自 BIOS 默认值，并非经验证的实时值。";
            }
        }
    }

    function renderOptions(entry) {
        const container = $("bios-detail-options");
        if (!container) return;

        if (!entry.options.length) {
            container.innerHTML =
                '<span class="bios-muted">没有可用的选项列表。</span>';
            return;
        }

        container.innerHTML = entry.options.map((option) => {
            const current = option.value === entry.currentValue;

            return `
                <span class="bios-option-chip${current ? " is-current" : ""}">
                    ${escapeHtml(option.value)} — ${escapeHtml(option.label)}
                </span>
            `;
        }).join("");
    }

    async function readBios() {
        if (state.loading) return;

        setLoading(true);

        try {
            const invoke = getInvoke();
            const text = await invoke("read_bios_nvram");

            if (typeof text !== "string" || !text.trim()) {
                throw new Error("Rust 命令未返回有效的 NVRAM 文本。");
            }

            state.nextId = 1;

            const entries = parseNVRAM(text);

            console.table(entries.slice(0, 10).map((entry) => ({
                question: entry.question,
                exportedValue: entry.exportedValue,
                previewValue: entry.currentValue,
                valueSource: entry.valueSource,
                defaultValue: entry.defaultValue,
                optionsCount: entry.options.length,
                options: JSON.stringify(entry.options),
            })));

            state.rawText = text;
            state.entries = entries;
            state.selectedId = entries[0]?.id ?? null;

            const sourceText = $("bios-source-text");

            if (sourceText) {
                sourceText.textContent = text;
            }

            const sourceInfo = $("bios-source-info");

            if (sourceInfo) {
                sourceInfo.textContent =
                    `${text.length.toLocaleString()} 个字符 · ${entries.length} 个配置项`;
            }

            const copySourceButton = $("bios-copy-source-btn");

            if (copySourceButton) {
                copySourceButton.disabled = false;
            }

            applyFilters();
            renderDetails();

            setStatus(`成功读取 ${entries.length} 个 BIOS 配置项。`, "success");
        } catch (error) {
            console.error("[BIOS] Read failed:", error);

            const message =
                error instanceof Error ? error.message : String(error);

            setStatus(`读取 BIOS NVRAM 失败：${message}`, "error");
        } finally {
            setLoading(false);
            updateButtonStates();
        }
    }

    function downloadText(
        filename,
        text,
        mime = "text/plain;charset=utf-8"
    ) {
        const blob = new Blob([text], { type: mime });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");

        link.href = url;
        link.download = filename;
        link.style.display = "none";

        document.body.appendChild(link);
        link.click();
        link.remove();

        setTimeout(() => URL.revokeObjectURL(url), 1000);
    }

    /*
     * 导出 NVRAM：
     * 先将本地修改合并进原始文本，再交给 Rust 保存。
     */
    async function exportRaw() {
        if (!state.rawText || !state.rawText.trim()) {
            setStatus("没有可导出的 NVRAM 数据，请先读取 BIOS。", "error");
            return;
        }

        try {
            const modifiedText = buildModifiedNvramText();
            const invoke = getInvoke();

            const savedPath = await invoke("export_nvram", {
                text: modifiedText,
            });

            setStatus(
                `NVRAM 已导出：${savedPath || "保存完成"}`,
                "success"
            );
        } catch (error) {
            console.error("[BIOS] Modified NVRAM export failed:", error);

            setStatus(
                `导出 NVRAM 失败：${error instanceof Error ? error.message : String(error)}`,
                "error"
            );
        }
    }

    function exportRecords() {
        const records = state.entries.map((entry) => ({
            question: entry.question,
            help: entry.help,
            token: entry.token,
            offset: entry.offset,
            width: entry.width,
            biosDefault: entry.defaultValue,
            exportedValue: entry.exportedValue,
            originalPreviewValue: entry.originalValue,
            previewValue: entry.currentValue,
            valueSource: entry.valueSource,
            modifiedLocally: isModified(entry),
            options: entry.options,
            raw: entry.raw,
        }));

        downloadText(
            "bios-config-records.json",
            JSON.stringify(records, null, 2),
            "application/json;charset=utf-8"
        );

        setStatus("已导出 BIOS 配置记录 JSON。", "success");
    }

    function resetEdits() {
        for (const entry of state.entries) {
            entry.currentValue = entry.originalValue;
        }

        applyFilters();
        renderDetails();

        setStatus("已重置所有本地预览修改。", "success");
    }

    function resetSelectedEntry() {
        const entry = getSelectedEntry();
        if (!entry) return;

        entry.currentValue = entry.originalValue;

        applyFilters();
        renderDetails();

        setStatus("已恢复所选配置项的原始预览值。", "success");
    }

    async function copyText(text) {
        try {
            if (navigator.clipboard && window.isSecureContext) {
                await navigator.clipboard.writeText(text);
            } else {
                const textarea = document.createElement("textarea");

                textarea.value = text;
                textarea.style.position = "fixed";
                textarea.style.opacity = "0";

                document.body.appendChild(textarea);
                textarea.select();

                const success = document.execCommand("copy");
                textarea.remove();

                if (!success) {
                    throw new Error("复制命令未成功执行。");
                }
            }

            setStatus("已复制到剪贴板。", "success");
        } catch (error) {
            console.error("[BIOS] Copy failed:", error);
            setStatus("复制失败，请手动选择并复制文本。", "error");
        }
    }

    function copySelectedDetails() {
        const entry = getSelectedEntry();
        if (!entry) return;

        const text = [
            `Question: ${entry.question}`,
            `Help String: ${entry.help}`,
            `Token: ${entry.token}`,
            `Offset: ${entry.offset}`,
            `Width: ${entry.width}`,
            `BIOS Default: ${entry.defaultValue}`,
            `Exported Value: ${entry.exportedValue}`,
            `Original Preview Value: ${entry.originalValue}`,
            `Preview Value: ${entry.currentValue}`,
            `Value Source: ${entry.valueSource}`,
            `Modified Locally: ${isModified(entry)}`,
            "",
            "Raw entry:",
            entry.raw,
        ].join("\n");

        copyText(text);
    }

    function selectEntry(id) {
        const entry = state.entries.find((item) => item.id === id);
        if (!entry) return;

        state.selectedId = id;

        renderTable();
        renderDetails();
    }

    /*
     * 写入 BIOS：
     * 按要求仅调用 Rust 命令，不传递任何参数。
     */
    async function writeBios() {
        const confirmed = window.confirm(
            "警告：写入 BIOS NVRAM 可能导致系统无法启动或配置异常。\n\n" +
            "是否继续调用 BIOS 写入命令？"
        );

        if (!confirmed) {
            setStatus("已取消 BIOS 写入。", "normal");
            return;
        }

        const button = $("bios-write-btn");

        if (button) {
            button.disabled = true;
        }

        try {
            const invoke = getInvoke();
            async function writeBiosNvram() {
                const password = window.prompt("请输入 BIOS 管理员密码：");

                // 用户取消输入时不执行写入
                if (password === null) {
                    return;
                }

                if (!password.trim()) {
                    window.alert("管理员密码不能为空");
                    return;
                }

                try {
                    await invoke("write_bios_nvram", {
                        password: password
                    });

                    window.alert("BIOS 写入成功");
                } catch (error) {
                    console.error("写入 BIOS 失败：", error);
                    window.alert(`写入 BIOS 失败：${error}`);
                }
            }
            writeBiosNvram();
            setStatus(
                typeof result === "string" && result.length > 0
                    ? `BIOS 写入命令已完成：${result}`
                    : "BIOS 写入命令已完成。",
                "success"
            );
        } catch (error) {
            console.error("[BIOS] BIOS write failed:", error);

            setStatus(
                `BIOS 写入失败：${error instanceof Error ? error.message : String(error)}`,
                "error"
            );
        } finally {
            if (button) {
                button.disabled = state.entries.length === 0 || state.loading;
            }
        }
    }

    function bindEvent(id, eventName, handler) {
        const element = $(id);

        if (!element) {
            console.warn(`[BIOS] 未找到元素 #${id}，跳过事件绑定。`);
            return;
        }

        element.addEventListener(eventName, handler);
    }

    function bindEvents() {
        bindEvent("bios-refresh-btn", "click", readBios);
        bindEvent("bios-export-raw-btn", "click", exportRaw);
        bindEvent("bios-export-json-btn", "click", exportRecords);
        bindEvent("bios-reset-btn", "click", resetEdits);
        bindEvent("bios-detail-reset-btn", "click", resetSelectedEntry);
        bindEvent("bios-detail-copy-btn", "click", copySelectedDetails);
        bindEvent("bios-write-btn", "click", writeBios);

        bindEvent("bios-search", "input", applyFilters);
        bindEvent("bios-filter", "change", applyFilters);

        bindEvent("bios-clear-search", "click", () => {
            const search = $("bios-search");

            if (search) {
                search.value = "";
                applyFilters();
                search.focus();
            }
        });

        bindEvent("bios-show-all-btn", "click", () => {
            const search = $("bios-search");
            const filter = $("bios-filter");

            if (search) search.value = "";
            if (filter) filter.value = "all";

            applyFilters();
        });

        bindEvent("bios-table-body", "click", (event) => {
            const row = event.target.closest("[data-entry-id]");
            if (!row) return;

            selectEntry(Number(row.dataset.entryId));
        });

        bindEvent("bios-table-body", "keydown", (event) => {
            if (event.key !== "Enter" && event.key !== " ") return;

            const row = event.target.closest("[data-entry-id]");
            if (!row) return;

            event.preventDefault();
            selectEntry(Number(row.dataset.entryId));
        });

        bindEvent("bios-toggle-source-btn", "click", () => {
            const content = $("bios-source-content");
            const button = $("bios-toggle-source-btn");

            if (!content || !button) return;

            const expanded = !content.hidden;

            content.hidden = expanded;
            button.setAttribute("aria-expanded", String(!expanded));
            button.textContent = expanded ? "显示源数据" : "隐藏源数据";
        });

        bindEvent("bios-copy-source-btn", "click", () => {
            if (state.rawText) {
                copyText(state.rawText);
            }
        });
    }

    function initBiosPage() {
        const page = $(PAGE_ID);

        if (!page) {
            return;
        }

        if (page.dataset.biosInitialized === "true") {
            return;
        }

        page.dataset.biosInitialized = "true";
        state.initialized = true;

        bindEvents();
        updateStats();
        updateButtonStates();

        console.log("[BIOS] Page initialized.");
    }

    window.initBiosPage = initBiosPage;

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", initBiosPage, {
            once: true,
        });
    } else {
        initBiosPage();
    }
})();
