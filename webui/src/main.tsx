import React, { useCallback, useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import {
  Check,
  ChevronRight,
  CircleUserRound,
  CloudCog,
  KeyRound,
  Moon,
  Pencil,
  RefreshCw,
  Router,
  Search,
  ShieldCheck,
  Sparkles,
  Sun,
  Trash2,
  X,
} from "lucide-react";
import "../../src/styles/base.css";
import "./styles.css";
import antigravityIcon from "../../src/assets/icons/antigravity-app.png";
import claudeIcon from "../../src/assets/icons/claude.png";
import codebuddyIcon from "../../src/assets/icons/codebuddy.png";
import codexIcon from "../../src/assets/icons/codex.svg";
import copilotIcon from "../../src/assets/icons/github-copilot.svg";
import cursorIcon from "../../src/assets/icons/cursor-menu.png";
import kiroIcon from "../../src/assets/icons/kiro-menu.png";
import qoderIcon from "../../src/assets/icons/qoder.png";
import traeCnIcon from "../../src/assets/icons/trae-cn.png";
import traeIcon from "../../src/assets/icons/trae.png";
import traeSoloCnIcon from "../../src/assets/icons/trae-solo-cn.png";
import traeSoloIcon from "../../src/assets/icons/trae-solo.png";
import windsurfIcon from "../../src/assets/icons/windsurf.svg";
import workbuddyIcon from "../../src/assets/icons/workbuddy.png";
import zcodeIcon from "../../src/assets/icons/zcode.png";
import zedIcon from "../../src/assets/icons/zed.png";

type Provider = {
  id: string;
  label: string;
  can_switch: boolean;
};

type Account = {
  id: string;
  email: string;
  name?: string;
  plan?: string;
  tags: string[];
  status?: string;
  last_used: number;
  can_switch: boolean;
};

type Health = {
  status: string;
  version: string;
  auth_required: boolean;
  providers: number;
};

const TOKEN_KEY = "cockpit-web-token";
const THEME_KEY = "cockpit-web-theme";

const providerIcons: Record<string, string> = {
  antigravity: antigravityIcon,
  codex: codexIcon,
  claude: claudeIcon,
  zed: zedIcon,
  "github-copilot": copilotIcon,
  windsurf: windsurfIcon,
  kiro: kiroIcon,
  cursor: cursorIcon,
  codebuddy: codebuddyIcon,
  "codebuddy-cn": codebuddyIcon,
  qoder: qoderIcon,
  zcode: zcodeIcon,
  trae: traeIcon,
  "trae-solo": traeSoloIcon,
  "trae-cn": traeCnIcon,
  "trae-solo-cn": traeSoloCnIcon,
  workbuddy: workbuddyIcon,
};

function providerIcon(provider: Provider, className = "provider-icon") {
  const icon = providerIcons[provider.id];
  if (!icon) return <Sparkles className={className} aria-hidden="true" />;
  return <img className={className} src={icon} alt="" />;
}

function initials(account: Account) {
  const value = account.name || account.email || account.id;
  return value
    .split(/[\s@._-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase())
    .join("") || "?";
}

function formatLastUsed(value: number) {
  if (!value) return "Chưa có dữ liệu";
  const milliseconds = value < 10_000_000_000 ? value * 1000 : value;
  const date = new Date(milliseconds);
  return Number.isNaN(date.getTime())
    ? "Không xác định"
    : new Intl.DateTimeFormat("vi-VN", {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(date);
}

function App() {
  const [providers, setProviders] = useState<Provider[]>([]);
  const [providerId, setProviderId] = useState("antigravity");
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [health, setHealth] = useState<Health | null>(null);
  const [token, setToken] = useState(() => sessionStorage.getItem(TOKEN_KEY) ?? "");
  const [tokenDraft, setTokenDraft] = useState(token);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [tagDraft, setTagDraft] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [dark, setDark] = useState(() => localStorage.getItem(THEME_KEY) === "dark");

  useEffect(() => {
    document.documentElement.dataset.theme = dark ? "dark" : "light";
    localStorage.setItem(THEME_KEY, dark ? "dark" : "light");
  }, [dark]);

  const request = useCallback(
    async <T,>(path: string, init?: RequestInit): Promise<T> => {
      const headers = new Headers(init?.headers);
      headers.set("Accept", "application/json");
      if (init?.body) headers.set("Content-Type", "application/json");
      if (token) headers.set("Authorization", `Bearer ${token}`);
      const response = await fetch(path, { ...init, headers });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(payload.error ?? `Request failed (${response.status})`);
      return payload as T;
    },
    [token],
  );

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const server = await request<Health>("/api/health");
      setHealth(server);
      if (server.auth_required && !token) {
        setAccounts([]);
        return;
      }
      const providerList = await request<Provider[]>("/api/providers");
      setProviders(providerList);
      const selected = providerList.some((provider) => provider.id === providerId)
        ? providerId
        : providerList[0]?.id;
      if (!selected) {
        setAccounts([]);
        return;
      }
      if (selected !== providerId) setProviderId(selected);
      const result = await request<{ accounts: Account[] }>(
        `/api/accounts?platform=${encodeURIComponent(selected)}`,
      );
      setAccounts(result.accounts);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
      setAccounts([]);
    } finally {
      setLoading(false);
    }
  }, [providerId, request, token]);

  useEffect(() => {
    void load();
  }, [load]);

  const selectedProvider = providers.find((provider) => provider.id === providerId) ?? providers[0];
  const visibleAccounts = useMemo(() => {
    const needle = query.trim().toLocaleLowerCase();
    if (!needle) return accounts;
    return accounts.filter((account) =>
      [account.email, account.name, account.plan, account.status, ...account.tags]
        .filter(Boolean)
        .some((value) => value!.toLocaleLowerCase().includes(needle)),
    );
  }, [accounts, query]);

  const saveToken = () => {
    const value = tokenDraft.trim();
    if (value) sessionStorage.setItem(TOKEN_KEY, value);
    else sessionStorage.removeItem(TOKEN_KEY);
    setError("");
    setNotice("");
    setToken(value);
  };

  const switchAccount = async (account: Account) => {
    if (!selectedProvider || !account.can_switch || working) return;
    if (!window.confirm(`Chuyển ${selectedProvider.label} sang ${account.email}?`)) return;
    setWorking(account.id);
    setError("");
    setNotice("");
    try {
      const result = await request<{ message: string }>("/api/switch", {
        method: "POST",
        body: JSON.stringify({ platform: selectedProvider.id, account: account.id }),
      });
      setNotice(result.message);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setWorking(null);
    }
  };

  const startEditing = (account: Account) => {
    setEditing(account.id);
    setTagDraft(account.tags.join(", "));
  };

  const saveTags = async (account: Account) => {
    if (!selectedProvider || working) return;
    const tags = tagDraft.split(",").map((tag) => tag.trim()).filter(Boolean);
    setWorking(account.id);
    setError("");
    setNotice("");
    try {
      const result = await request<{ message: string }>(
        `/api/accounts/${encodeURIComponent(selectedProvider.id)}/${encodeURIComponent(account.id)}/tags`,
        { method: "PATCH", body: JSON.stringify({ tags }) },
      );
      setNotice(result.message);
      setEditing(null);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setWorking(null);
    }
  };

  const deleteAccount = async (account: Account) => {
    if (!selectedProvider || working) return;
    if (!window.confirm(`Đưa ${account.email} vào web-trash? Bạn có thể khôi phục thủ công.`)) return;
    setWorking(account.id);
    setError("");
    setNotice("");
    try {
      const result = await request<{ message: string }>(
        `/api/accounts/${encodeURIComponent(selectedProvider.id)}/${encodeURIComponent(account.id)}`,
        { method: "DELETE" },
      );
      setNotice(result.message);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setWorking(null);
    }
  };

  const authLocked = Boolean(health?.auth_required && !token);
  const activeAccounts = accounts.filter((account) => account.status?.toLowerCase() !== "banned").length;

  return (
    <div className="app-shell">
      <aside className="side-nav" aria-label="Providers">
        <div className="brand-logo" title="Cockpit Tools"><Router size={23} /></div>
        <nav className="provider-nav">
          {providers.map((provider) => (
            <button
              key={provider.id}
              className={`nav-item ${provider.id === providerId ? "active" : ""}`}
              onClick={() => { setProviderId(provider.id); setQuery(""); setEditing(null); }}
              title={provider.label}
              aria-label={provider.label}
            >
              {providerIcon(provider, "nav-provider-icon")}
              <span className="tooltip">{provider.label}</span>
            </button>
          ))}
        </nav>
        <div className={`server-indicator ${health?.status === "ok" ? "online" : ""}`} title="API status" />
      </aside>

      <main className="workspace">
        <header className="topbar">
          <div className="page-heading">
            <div className="heading-icon">{selectedProvider ? providerIcon(selectedProvider) : <CloudCog />}</div>
            <div>
              <p className="eyebrow">ACCOUNT MANAGER</p>
              <h1>{selectedProvider?.label ?? "Cockpit Tools"}</h1>
              <p>Quản lý tài khoản được lưu trên thiết bị Android này.</p>
            </div>
          </div>
          <div className="topbar-actions">
            <div className="api-chip"><span className="status-dot" />API online · v{health?.version ?? "—"}</div>
            <button className="icon-button" onClick={() => setDark((value) => !value)} title="Đổi giao diện">
              {dark ? <Sun size={18} /> : <Moon size={18} />}
            </button>
            <button className="icon-button" onClick={() => void load()} disabled={loading} title="Làm mới">
              <RefreshCw className={loading ? "spin" : ""} size={18} />
            </button>
          </div>
        </header>

        {health?.auth_required && (
          <section className="token-card">
            <div className="token-copy"><KeyRound size={19} /><div><strong>Bearer token</strong><span>Token chỉ được giữ trong session của trình duyệt.</span></div></div>
            <input type="password" value={tokenDraft} onChange={(event) => setTokenDraft(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") saveToken(); }} placeholder="Nhập COCKPIT_WEB_TOKEN" />
            <button className="primary-button" onClick={saveToken}>Kết nối</button>
          </section>
        )}

        {!authLocked && selectedProvider && (
          <>
            <section className="summary-grid">
              <article><CircleUserRound /><div><span>Tổng tài khoản</span><strong>{accounts.length}</strong></div></article>
              <article><ShieldCheck /><div><span>Khả dụng</span><strong>{activeAccounts}</strong></div></article>
              <article><CloudCog /><div><span>Chuyển trên Android</span><strong>{selectedProvider.can_switch ? "Hỗ trợ" : "Chỉ quản lý"}</strong></div></article>
            </section>

            <section className="accounts-card">
              <div className="card-heading">
                <div><h2>Accounts</h2><p>Dữ liệu nhạy cảm như access token không được gửi tới WebUI.</p></div>
                <label className="search-box"><Search size={17} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Tìm tài khoản..." /></label>
              </div>

              {error && <div className="message error">{error}</div>}
              {notice && <div className="message success"><Check size={16} />{notice}</div>}

              <div className="account-list">
                {loading ? (
                  <div className="empty-state"><RefreshCw className="spin" size={20} />Đang tải dữ liệu…</div>
                ) : visibleAccounts.length === 0 ? (
                  <div className="empty-state"><CircleUserRound size={24} /><strong>Không tìm thấy tài khoản</strong><span>Thêm tài khoản từ Cockpit Tools hoặc kiểm tra thư mục dữ liệu.</span></div>
                ) : visibleAccounts.map((account) => (
                  <article className="account-row" key={account.id}>
                    <div className="avatar">{initials(account)}</div>
                    <div className="account-main">
                      <div className="account-title"><strong>{account.name || account.email}</strong>{account.plan && <span className="plan-badge">{account.plan}</span>}</div>
                      <span>{account.email}</span>
                      {editing === account.id ? (
                        <div className="tag-editor">
                          <input autoFocus value={tagDraft} onChange={(event) => setTagDraft(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") void saveTags(account); if (event.key === "Escape") setEditing(null); }} placeholder="work, vip, backup" />
                          <button onClick={() => void saveTags(account)} disabled={working === account.id}><Check size={15} /></button>
                          <button onClick={() => setEditing(null)}><X size={15} /></button>
                        </div>
                      ) : (
                        <div className="tag-list">
                          {account.tags.map((tag) => <span key={tag}>{tag}</span>)}
                          <button onClick={() => startEditing(account)} title="Sửa tag"><Pencil size={11} />{account.tags.length === 0 ? "Thêm tag" : ""}</button>
                        </div>
                      )}
                    </div>
                    <div className="account-meta"><span>{account.status || "Ready"}</span><small>{formatLastUsed(account.last_used)}</small></div>
                    <div className="row-actions">
                      <button className="danger-button" onClick={() => void deleteAccount(account)} disabled={working === account.id} title="Đưa vào web-trash"><Trash2 size={16} /></button>
                      <button className="switch-button" onClick={() => void switchAccount(account)} disabled={!account.can_switch || Boolean(working)} title={account.can_switch ? "Chuyển tài khoản" : "Provider này cần ứng dụng desktop để chuyển tài khoản"}>
                        {working === account.id ? <RefreshCw className="spin" size={15} /> : account.can_switch ? "Switch" : "Desktop"}<ChevronRight size={15} />
                      </button>
                    </div>
                  </article>
                ))}
              </div>
            </section>

            <footer className="security-note"><ShieldCheck size={15} />WebUI chỉ đọc metadata an toàn · Xóa là chuyển vào <code>web-trash</code> · Mặc định chỉ bind localhost</footer>
          </>
        )}

        {authLocked && (
          <section className="locked-state"><div><KeyRound size={28} /></div><h2>WebUI đang được bảo vệ</h2><p>Nhập bearer token ở phía trên để tải danh sách {health?.providers ?? 18} provider.</p></section>
        )}
      </main>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode><App /></React.StrictMode>,
);
