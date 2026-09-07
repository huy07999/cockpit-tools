import React, { useCallback, useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import {
  CheckCircle2,
  Github,
  KeyRound,
  Moon,
  RefreshCw,
  Router,
  Sun,
  Users,
} from "lucide-react";
import "../../src/styles/base.css";
import "./styles.css";

type Platform = "cursor" | "copilot";

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
};

const TOKEN_KEY = "cockpit-web-token";

function App() {
  const [platform, setPlatform] = useState<Platform>("cursor");
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [health, setHealth] = useState<Health | null>(null);
  const [token, setToken] = useState(() => sessionStorage.getItem(TOKEN_KEY) ?? "");
  const [tokenDraft, setTokenDraft] = useState(token);
  const [loading, setLoading] = useState(true);
  const [switching, setSwitching] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [dark, setDark] = useState(() => localStorage.getItem("cockpit-web-theme") === "dark");

  useEffect(() => {
    document.documentElement.dataset.theme = dark ? "dark" : "light";
    localStorage.setItem("cockpit-web-theme", dark ? "dark" : "light");
  }, [dark]);

  const request = useCallback(
    async <T,>(path: string, init?: RequestInit): Promise<T> => {
      const headers = new Headers(init?.headers);
      headers.set("Accept", "application/json");
      if (init?.body) headers.set("Content-Type", "application/json");
      if (token) headers.set("Authorization", `Bearer ${token}`);
      const response = await fetch(path, { ...init, headers });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(payload.error ?? `Request failed (${response.status})`);
      }
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
      const result = await request<{ accounts: Account[] }>(`/api/accounts?platform=${platform}`);
      setAccounts(result.accounts);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }, [platform, request, token]);

  useEffect(() => {
    void load();
  }, [load]);

  const saveToken = () => {
    const value = tokenDraft.trim();
    sessionStorage.setItem(TOKEN_KEY, value);
    setError("");
    setNotice("");
    setToken(value);
  };

  const switchAccount = async (account: Account) => {
    if (!account.can_switch || switching) return;
    if (!window.confirm(`Switch Cursor to ${account.email}?`)) return;
    setSwitching(account.id);
    setError("");
    setNotice("");
    try {
      const result = await request<{ message: string }>("/api/switch", {
        method: "POST",
        body: JSON.stringify({ platform, account: account.id }),
      });
      setNotice(result.message);
      await load();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSwitching(null);
    }
  };

  const activeCount = useMemo(
    () => accounts.filter((account) => account.status !== "banned").length,
    [accounts],
  );

  return (
    <div className="web-shell">
      <aside className="web-sidebar">
        <div className="brand-mark"><Router size={22} /></div>
        <div className="brand-copy"><strong>Cockpit</strong><span>WebUI</span></div>
        <nav>
          <button className={platform === "cursor" ? "active" : ""} onClick={() => setPlatform("cursor")}>
            <span className="cursor-glyph">C</span><span>Cursor</span>
          </button>
          <button className={platform === "copilot" ? "active" : ""} onClick={() => setPlatform("copilot")}>
            <Github size={19} /><span>GitHub Copilot</span>
          </button>
        </nav>
        <div className="sidebar-foot">
          <span className={`status-dot ${health?.status === "ok" ? "online" : ""}`} />
          API {health?.status === "ok" ? "online" : "connecting"}
        </div>
      </aside>

      <main>
        <header>
          <div>
            <p className="eyebrow">SELF-HOSTED ACCOUNT CONTROL</p>
            <h1>{platform === "cursor" ? "Cursor Accounts" : "GitHub Copilot Accounts"}</h1>
            <p>Manage accounts stored on this Cockpit host from your browser.</p>
          </div>
          <div className="header-actions">
            <button className="icon-button" onClick={() => setDark((value) => !value)} aria-label="Toggle theme">
              {dark ? <Sun size={18} /> : <Moon size={18} />}
            </button>
            <button className="refresh-button" onClick={() => void load()} disabled={loading}>
              <RefreshCw size={17} className={loading ? "spin" : ""} /> Refresh
            </button>
          </div>
        </header>

        {health?.auth_required && (!token || error === "invalid bearer token") && (
          <section className="token-panel">
            <KeyRound size={22} />
            <div><strong>API token required</strong><p>Enter the token configured on the Cockpit server.</p></div>
            <input type="password" value={tokenDraft} onChange={(event) => setTokenDraft(event.target.value)} onKeyDown={(event) => event.key === "Enter" && saveToken()} placeholder="Bearer token" />
            <button onClick={saveToken}>Connect</button>
          </section>
        )}

        <section className="stats-grid">
          <article><Users size={20} /><div><span>Total accounts</span><strong>{accounts.length}</strong></div></article>
          <article><CheckCircle2 size={20} /><div><span>Available</span><strong>{activeCount}</strong></div></article>
          <article><Router size={20} /><div><span>Server</span><strong>v{health?.version ?? "…"}</strong></div></article>
        </section>

        {error && <div className="message error">{error}</div>}
        {notice && <div className="message success">{notice}</div>}

        <section className="accounts-panel">
          <div className="panel-heading">
            <div><h2>Account pool</h2><p>Tokens stay on the server and are never returned by this API.</p></div>
            {platform === "copilot" && <span className="read-only-badge">View only</span>}
          </div>

          {loading ? (
            <div className="empty-state"><RefreshCw className="spin" /> Loading accounts…</div>
          ) : accounts.length === 0 ? (
            <div className="empty-state"><Users /> No {platform} accounts found on this host.</div>
          ) : (
            <div className="account-list">
              {accounts.map((account) => (
                <article className="account-row" key={account.id}>
                  <div className="avatar">{(account.name || account.email || "?").slice(0, 1).toUpperCase()}</div>
                  <div className="account-main">
                    <strong>{account.name || account.email}</strong>
                    <span>{account.email}</span>
                    <div className="tag-row">
                      {account.plan && <span>{account.plan}</span>}
                      {account.status && <span className={account.status === "banned" ? "danger" : ""}>{account.status}</span>}
                      {account.tags.map((tag) => <span key={tag}>{tag}</span>)}
                    </div>
                  </div>
                  <div className="account-meta">
                    <small>Last used</small>
                    <span>{account.last_used ? new Date(account.last_used * 1000).toLocaleString() : "Never"}</span>
                  </div>
                  <button className="switch-button" disabled={!account.can_switch || switching === account.id} onClick={() => void switchAccount(account)}>
                    {account.can_switch ? (switching === account.id ? "Switching…" : "Switch") : "Unavailable"}
                  </button>
                </article>
              ))}
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode><App /></React.StrictMode>,
);
