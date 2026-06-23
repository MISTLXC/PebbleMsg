import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getPopupConfig, updatePopupConfig, showTestPopup } from "@/lib/api";
import type { PopupConfig } from "@/lib/api";
import { useToastStore } from "@/stores/toast.store";

const DEFAULT_CONFIG: PopupConfig = {
  width: 380,
  height: 200,
  position_x: null,
  position_y: null,
  duration_ms: 8000,
  show_sender: true,
  show_subject: true,
  show_snippet: true,
  show_time: true,
  max_popups: 1,
  font_size_sender: 13,
  font_size_subject: 13,
  font_size_snippet: 12,
  font_size_time: 11,
  auto_launch: true,
  minimize_on_startup: false,
};

const sectionTitle: React.CSSProperties = {
  fontSize: "14px", fontWeight: 600, marginBottom: "16px", marginTop: "32px",
};
const cardStyle: React.CSSProperties = {
  border: "1px solid var(--color-border)",
  borderRadius: "8px",
  padding: "14px",
  display: "flex",
  flexDirection: "column",
  gap: "12px",
};
const rangeGroup: React.CSSProperties = {
  display: "flex", gap: "12px",
};
const rangeItem: React.CSSProperties = {
  flex: 1, display: "flex", flexDirection: "column", gap: "4px",
};
const rangeLabel: React.CSSProperties = {
  fontSize: "12px", fontWeight: 600, color: "var(--color-text-secondary)",
};
const checkboxRow: React.CSSProperties = {
  display: "flex", alignItems: "center", gap: "8px", cursor: "pointer",
  fontSize: "13px", color: "var(--color-text-primary)",
};
const inputStyle: React.CSSProperties = {
  padding: "6px 10px", borderRadius: "6px", border: "1px solid var(--color-border)",
  backgroundColor: "var(--color-bg)", color: "var(--color-text-primary)",
  fontSize: "13px", width: "100px",
};
const selectStyle: React.CSSProperties = {
  padding: "4px 6px", borderRadius: "5px", border: "1px solid var(--color-border)",
  backgroundColor: "var(--color-bg)", color: "var(--color-text-primary)",
  fontSize: "12px", width: "72px",
};

const FONT_SIZE_OPTIONS = [9, 10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24];

function numVal(v: string, fallback: number, min: number, max: number): number {
  const n = parseInt(v, 10);
  return isNaN(n) ? fallback : Math.max(min, Math.min(max, n));
}

export default function PopupTab() {
  const { t } = useTranslation();
  const addToast = useToastStore((s) => s.addToast);
  const [config, setConfig] = useState<PopupConfig>(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [previewing, setPreviewing] = useState(false);

  useEffect(() => {
    let cancelled = false;
    getPopupConfig()
      .then((cfg) => { if (!cancelled) setConfig(cfg); })
      .catch(() => {})
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, []);

  const update = useCallback(<K extends keyof PopupConfig>(key: K, value: PopupConfig[K]) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  }, []);

  const handleSave = useCallback(async () => {
    setSaving(true);
    try {
      await updatePopupConfig(config);
      addToast({ message: t("popup.saved", "Saved"), type: "success" });
    } catch {
      addToast({ message: t("popup.saveFailed", "Save failed"), type: "error" });
    } finally { setSaving(false); }
  }, [config, addToast, t]);

  const handlePreview = useCallback(async () => {
    setPreviewing(true);
    try {
      await updatePopupConfig(config);
      await showTestPopup();
      addToast({ message: t("popup.previewSent", "Test popup sent"), type: "success" });
    } catch {
      addToast({ message: t("popup.previewFailed", "Failed"), type: "error" });
    } finally { setPreviewing(false); }
  }, [config, addToast, t]);

  if (loading) {
    return <div style={{ padding: "16px", fontSize: "13px", color: "var(--color-text-secondary)" }}>
      {t("common.loading", "Loading...")}
    </div>;
  }

  return (
    <div>
      <h3 style={{ ...sectionTitle, marginTop: 0 }}>{t("popup.title", "Popup Notifications")}</h3>

      <h3 style={sectionTitle}>{t("popup.dimensions", "Window Size")}</h3>
      <div style={cardStyle}>
        <div style={rangeGroup}>
          <div style={rangeItem}>
            <span style={rangeLabel}>{t("popup.width", "Width (px)")}</span>
            <input type="number" value={config.width}
              onChange={(e) => update("width", numVal(e.target.value, 380, 200, 800))}
              style={inputStyle} min={200} max={800} step={10} />
          </div>
          <div style={rangeItem}>
            <span style={rangeLabel}>{t("popup.height", "Height (px)")}</span>
            <input type="number" value={config.height}
              onChange={(e) => update("height", numVal(e.target.value, 200, 120, 500))}
              style={inputStyle} min={120} max={500} step={10} />
          </div>
        </div>
        <div style={rangeGroup}>
          <div style={rangeItem}>
            <span style={rangeLabel}>{t("popup.positionX", "Position X")}</span>
            <input type="number" value={config.position_x ?? ""}
              onChange={(e) => update("position_x", e.target.value ? parseInt(e.target.value, 10) : null)}
              style={{ ...inputStyle, width: "100%" }} placeholder={t("popup.auto", "auto")} />
          </div>
          <div style={rangeItem}>
            <span style={rangeLabel}>{t("popup.positionY", "Position Y")}</span>
            <input type="number" value={config.position_y ?? ""}
              onChange={(e) => update("position_y", e.target.value ? parseInt(e.target.value, 10) : null)}
              style={{ ...inputStyle, width: "100%" }} placeholder={t("popup.auto", "auto")} />
          </div>
        </div>
      </div>

      <h3 style={sectionTitle}>{t("popup.behavior", "Behavior")}</h3>
      <div style={cardStyle}>
        <div style={rangeGroup}>
          <div style={{ ...rangeItem, flex: 2 }}>
            <span style={rangeLabel}>{t("popup.duration", "Duration")}</span>
            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
              <input type="range" min={1} max={180} step={1}
                value={config.duration_ms === 0 ? 180 : Math.min(config.duration_ms / 1000, 180)}
                onChange={(e) => {
                  const val = numVal(e.target.value, 8, 1, 180);
                  update("duration_ms", val === 180 ? 0 : val * 1000);
                }}
                style={{ flex: 1, accentColor: "var(--color-accent)" }} />
              <span style={{ fontSize: "13px", fontWeight: 600, color: "var(--color-text-primary)", minWidth: "48px", textAlign: "right" }}>
                {config.duration_ms === 0 ? t("popup.forever", "无限") : (config.duration_ms / 1000) + "s"}
              </span>
            </div>
          </div>
        </div>
        <div style={rangeGroup}>
          <div style={rangeItem}>
            <span style={rangeLabel}>{t("popup.maxPopups", "Max popups")}</span>
            <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
              <input type="range" min={1} max={5} step={1}
                value={config.max_popups}
                onChange={(e) => update("max_popups", numVal(e.target.value, 1, 1, 5))}
                style={{ flex: 1, accentColor: "var(--color-accent)" }} />
              <span style={{ fontSize: "13px", fontWeight: 600, color: "var(--color-text-primary)", minWidth: "48px", textAlign: "right" }}>
                {config.max_popups}
              </span>
            </div>
          </div>
        </div>
        <div style={{ fontSize: "11px", color: "var(--color-text-secondary)" }}>
          {t("popup.rollHint", "When max is reached, the oldest popup is closed and replaced.")}
        </div>
      </div>

      <h3 style={sectionTitle}>{t("popup.system", "System")}</h3>
      <div style={cardStyle}>
        <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
          <label style={checkboxRow}>
            <input type="checkbox" checked={config.auto_launch}
              onChange={(e) => update("auto_launch", e.target.checked)} />
            <span>{t("popup.autoLaunch", "开机自启动")}</span>
          </label>
          <label style={checkboxRow}>
            <input type="checkbox" checked={config.minimize_on_startup}
              onChange={(e) => update("minimize_on_startup", e.target.checked)} />
            <span>{t("popup.minimizeOnStartup", "启动时最小化")}</span>
          </label>
        </div>
      </div>

      <h3 style={sectionTitle}>{t("popup.content", "Content")}</h3>
      <div style={cardStyle}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px", justifyContent: "space-between" }}>
          <label style={checkboxRow}>
            <input type="checkbox" checked={config.show_sender}
              onChange={(e) => update("show_sender", e.target.checked)} />
            <span>{t("popup.showSender", "Sender name")}</span>
          </label>
          <select value={config.font_size_sender}
            onChange={(e) => update("font_size_sender", parseInt(e.target.value, 10))}
            style={selectStyle}>
            {FONT_SIZE_OPTIONS.map(sz => <option key={sz} value={sz}>{sz}px</option>)}
          </select>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "8px", justifyContent: "space-between" }}>
          <label style={checkboxRow}>
            <input type="checkbox" checked={config.show_time}
              onChange={(e) => update("show_time", e.target.checked)} />
            <span>{t("popup.showTime", "Received time")}</span>
          </label>
          <select value={config.font_size_time}
            onChange={(e) => update("font_size_time", parseInt(e.target.value, 10))}
            style={selectStyle}>
            {FONT_SIZE_OPTIONS.map(sz => <option key={sz} value={sz}>{sz}px</option>)}
          </select>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "8px", justifyContent: "space-between" }}>
          <label style={checkboxRow}>
            <input type="checkbox" checked={config.show_subject}
              onChange={(e) => update("show_subject", e.target.checked)} />
            <span>{t("popup.showSubject", "Subject")}</span>
          </label>
          <select value={config.font_size_subject}
            onChange={(e) => update("font_size_subject", parseInt(e.target.value, 10))}
            style={selectStyle}>
            {FONT_SIZE_OPTIONS.map(sz => <option key={sz} value={sz}>{sz}px</option>)}
          </select>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: "8px", justifyContent: "space-between" }}>
          <label style={checkboxRow}>
            <input type="checkbox" checked={config.show_snippet}
              onChange={(e) => update("show_snippet", e.target.checked)} />
            <span>{t("popup.showSnippet", "Snippet preview")}</span>
          </label>
          <select value={config.font_size_snippet}
            onChange={(e) => update("font_size_snippet", parseInt(e.target.value, 10))}
            style={selectStyle}>
            {FONT_SIZE_OPTIONS.map(sz => <option key={sz} value={sz}>{sz}px</option>)}
          </select>
        </div>
      </div>

      <div style={{ display: "flex", gap: "10px", marginTop: "24px" }}>
        <button type="button" onClick={handleSave} disabled={saving}
          style={{
            padding: "9px 20px", borderRadius: "6px",
            border: "1px solid var(--color-border)", backgroundColor: "var(--color-bg)",
            color: "var(--color-text-primary)", fontSize: "13px", fontWeight: 500,
            cursor: saving ? "not-allowed" : "pointer", opacity: saving ? 0.7 : 1,
          }}>
          {saving ? t("common.saving", "Saving...") : t("common.save", "Save")}
        </button>
        <button type="button" onClick={handlePreview} disabled={previewing}
          style={{
            padding: "9px 20px", borderRadius: "6px", border: "none",
            backgroundColor: "var(--color-accent)", color: "#fff",
            fontSize: "13px", fontWeight: 600,
            cursor: previewing ? "not-allowed" : "pointer", opacity: previewing ? 0.7 : 1,
          }}>
          {previewing ? t("popup.previewing", "Sending...") : t("popup.preview", "Preview")}
        </button>
      </div>
    </div>
  );
}
