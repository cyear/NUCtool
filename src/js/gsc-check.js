// ============================================================
// GSC Driver Check
// ============================================================

const GSC_RECOMMENDED_VERSION = '31.0.101.5590';

const INVOKE = window.__TAURI__.core.invoke;

/**
 * Driver check UI initialization.
 * Call once after initI18n().
 */
export async function initGscCheck() {
    bindFixButton();
    await updateGscCheck();
}

/**
 * Reads driver versions and updates the UI.
 */
async function updateGscCheck() {
    const el = document.getElementById('gsc-check');
    if (!el) return;

    const badge = document.getElementById('gsc-badge');
    const versions = document.getElementById('gsc-versions');
    const installed = document.getElementById('gsc-installed');
    const actions = document.getElementById('gsc-actions');

    try {
        // 1. Check for ARC GPU driver presence. If not present, hide the GSC check block.
        const arcInfo = await INVOKE('get_sys_arc_gpu_driver');
        if (!arcInfo) {
            el.hidden = true;
            return;
        }
        el.hidden = false;

        // 2. Read the GSC driver version
        const gscVersion = await INVOKE('get_sys_gsc_driver');

        if (!gscVersion) {
            setState({ el, badge, versions, actions }, 'notfound');
            return;
        }

        installed.textContent = gscVersion;

        if (gscVersion === GSC_RECOMMENDED_VERSION) {
            setState({ el, badge, versions, actions }, 'ok');
        } else {
            installed.classList.add('gsc-version-value--warn');
            setState({ el, badge, versions, actions }, 'warning');
        }
    } catch (err) {
        console.error('[GSC] check failed:', err);
        setState({ el, badge, versions, actions }, 'error');
    }
}

/**
 * Switches the visual state of the block.
 */
function setState({ el, badge, versions, actions }, kind) {
    badge.className = 'settings-status';
    badge.hidden = false;
    versions.hidden = false;
    actions.hidden = true;

    el.querySelector('.gsc-version-value--warn')
        ?.classList.remove('gsc-version-value--warn');

    const labelKey = `system.gsc.${kind}`;
    badge.textContent = window.t ? window.t(labelKey) : labelKey;

    switch (kind) {
        case 'ok':
            badge.classList.add('enabled');
            break;

        case 'warning':
            badge.classList.add('warning');
            actions.hidden = false;
            break;

        case 'notfound':
            badge.classList.add('disabled');
            versions.hidden = true;
            break;

        case 'error':
            badge.classList.add('error');
            versions.hidden = true;
            break;
    }
}

/**
 * Handler for the "How to fix" button.
 */
function bindFixButton() {
    const btn = document.getElementById('gsc-how-to-fix');
    if (!btn) return;

    btn.addEventListener('click', async () => {
        const url = 'https://github.com/cyear/NUCtool/blob/main/docs/gsc-stutter-fix.md';

        try {
            if (window.__TAURI__.opener?.openUrl) {
                await window.__TAURI__.opener.openUrl(url);
            } else {
                window.open(url, '_blank');
            }
        } catch (e) {
            console.error('[GSC] failed to open URL:', e);
        }
    });
}