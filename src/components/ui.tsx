import React, { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { AlertTriangle, CheckCircle2, Info, X, XCircle } from "lucide-react";

type BtnProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "danger" | "ghost" | "default";
  size?: "sm" | "lg";
  icon?: React.ReactNode;
  iconOnly?: boolean;
  ref?: React.Ref<HTMLButtonElement>;
};
export function Button({ variant = "default", size, icon, iconOnly, className = "", children, ...rest }: BtnProps) {
  const cls = ["btn", variant !== "default" && variant, size, iconOnly && "icon", className].filter(Boolean).join(" ");
  return (
    <button className={cls} {...rest}>
      {icon}
      {!iconOnly && children}
    </button>
  );
}

export function Field({ label, hint, hintKind, children, className = "", style }: {
  label?: React.ReactNode; hint?: React.ReactNode; hintKind?: "warn" | "danger"; children: React.ReactNode; className?: string; style?: React.CSSProperties;
}) {
  return (
    <div className={"field " + className} style={style}>
      {label && <label>{label}</label>}
      {children}
      {hint && <div className={"hint " + (hintKind ?? "")}>{hint}</div>}
    </div>
  );
}

export function TextInput({ value, onChange, maxLength, placeholder, className = "", onCommit, ...rest }: {
  value: string; onChange: (v: string) => void; maxLength?: number; placeholder?: string; className?: string; onCommit?: () => void;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange">) {
  return (
    <input
      className={"input " + className}
      value={value}
      placeholder={placeholder}
      spellCheck={false}
      onChange={(e) => {
        let v = e.target.value;
        if (maxLength && [...v].length > maxLength) v = [...v].slice(0, maxLength).join("");
        onChange(v);
      }}
      onBlur={onCommit}
      onKeyDown={(e) => { if (e.key === "Enter") onCommit?.(); }}
      {...rest}
    />
  );
}

/** Integer input that only commits valid values; shows a warning above `warnAbove`. */
export function NumberInput({ value, onChange, min = 0, max = 0xffffffff, step = 1, warnAbove, hex, className = "", suffix, width }: {
  value: number; onChange: (v: number) => void; min?: number; max?: number; step?: number; warnAbove?: number; hex?: boolean;
  className?: string; suffix?: string; width?: number;
}) {
  const fmt = (v: number) => (hex ? v.toString(16).toUpperCase() : String(v));
  const [text, setText] = useState(fmt(value));
  const [focus, setFocus] = useState(false);
  useEffect(() => { if (!focus) setText(fmt(value)); }, [value, focus, hex]);
  const parse = (t: string) => {
    const n = hex ? parseInt(t || "0", 16) : Number(t.replace(/[,\s_]/g, "") || "0");
    return Number.isFinite(n) ? n : NaN;
  };
  const n = parse(text);
  const invalid = Number.isNaN(n) || n < min || n > max || (!hex && !Number.isInteger(n));
  const warn = !invalid && warnAbove !== undefined && n > warnAbove;
  const commit = () => {
    if (!invalid) onChange(n);
    else setText(fmt(value));
  };
  return (
    <div className="input-wrap" style={{ width }}>
      <input
        className={`input num ${invalid ? "invalid" : warn ? "warn" : ""} ${className}`}
        value={focus ? text : hex ? text : Number(value).toLocaleString()}
        onFocus={() => { setFocus(true); setText(fmt(value)); }}
        onBlur={() => { setFocus(false); commit(); }}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") (e.target as HTMLInputElement).blur();
          if (e.key === "ArrowUp" || e.key === "ArrowDown") {
            e.preventDefault();
            const base = Number.isNaN(n) ? value : n;
            const next = Math.min(max, Math.max(min, base + (e.key === "ArrowUp" ? step : -step) * (e.shiftKey ? 10 : 1)));
            setText(fmt(next));
            onChange(next);
          }
        }}
        style={suffix ? { paddingRight: 34 } : undefined}
      />
      {suffix && <span className="suffix">{suffix}</span>}
    </div>
  );
}

export function Select<T extends string | number>({ value, onChange, options, className = "", style, disabled }: {
  value: T; onChange: (v: T) => void; options: { value: T; label: string }[]; className?: string; style?: React.CSSProperties; disabled?: boolean;
}) {
  const has = options.some((o) => o.value === value);
  return (
    <select
      className={"select " + className}
      style={style}
      disabled={disabled}
      value={String(value)}
      onChange={(e) => {
        const raw = e.target.value;
        const o = options.find((x) => String(x.value) === raw);
        onChange((o ? o.value : (typeof value === "number" ? Number(raw) : raw)) as T);
      }}
    >
      {!has && <option value={String(value)}>{String(value)} (unknown)</option>}
      {options.map((o) => (
        <option key={String(o.value)} value={String(o.value)}>{o.label}</option>
      ))}
    </select>
  );
}

export function Switch({ on, onChange, disabled }: { on: boolean; onChange: (v: boolean) => void; disabled?: boolean }) {
  return <button className={"switch " + (on ? "on" : "")} disabled={disabled} onClick={() => onChange(!on)} role="switch" aria-checked={on} />;
}

export function ToggleRow({ title, desc, on, onChange, disabled }: { title: React.ReactNode; desc?: React.ReactNode; on: boolean; onChange: (v: boolean) => void; disabled?: boolean }) {
  return (
    <div className="toggle-row">
      <div className="txt">
        <div className="t">{title}</div>
        {desc && <div className="d">{desc}</div>}
      </div>
      <Switch on={on} onChange={onChange} disabled={disabled} />
    </div>
  );
}

export function Check({ checked, onChange, label, title }: { checked: boolean; onChange: (v: boolean) => void; label?: React.ReactNode; title?: string }) {
  return (
    <label className="check" title={title}>
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} />
      {label}
    </label>
  );
}

export function Segmented<T extends string | number>({ value, onChange, options }: {
  value: T; onChange: (v: T) => void; options: { value: T; label: React.ReactNode; icon?: React.ReactNode }[];
}) {
  return (
    <div className="segmented">
      {options.map((o) => (
        <button key={String(o.value)} className={o.value === value ? "on" : ""} onClick={() => onChange(o.value)}>
          {o.icon}{o.label}
        </button>
      ))}
    </div>
  );
}

export function Tabs<T extends string>({ value, onChange, tabs }: { value: T; onChange: (v: T) => void; tabs: { value: T; label: React.ReactNode; icon?: React.ReactNode }[] }) {
  return (
    <div className="tabs">
      {tabs.map((t) => (
        <button key={t.value} className={t.value === value ? "on" : ""} onClick={() => onChange(t.value)}>{t.icon}{t.label}</button>
      ))}
    </div>
  );
}

export function Card({ title, desc, actions, children, className = "", bodyClass = "card-body", style }: {
  title?: React.ReactNode; desc?: React.ReactNode; actions?: React.ReactNode; children?: React.ReactNode; className?: string; bodyClass?: string; style?: React.CSSProperties;
}) {
  return (
    <section className={"card " + className} style={style}>
      {(title || actions) && (
        <div className="card-head">
          <h2>{title}{desc && <div className="desc">{desc}</div>}</h2>
          {actions}
        </div>
      )}
      <div className={bodyClass}>{children}</div>
    </section>
  );
}

export function Modal({ title, onClose, children, footer, size, icon }: {
  title: React.ReactNode; onClose: () => void; children: React.ReactNode; footer?: React.ReactNode; size?: "wide" | "xwide"; icon?: React.ReactNode;
}) {
  useEffect(() => {
    const k = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [onClose]);
  return createPortal(
    <div className="backdrop" onMouseDown={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className={"modal " + (size ?? "")}>
        <div className="modal-head">
          {icon}
          <h2>{title}</h2>
          <Button variant="ghost" iconOnly size="sm" icon={<X />} onClick={onClose} />
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-foot">{footer}</div>}
      </div>
    </div>,
    document.body,
  );
}

export function Badge({ kind, children }: { kind?: "accent" | "warn" | "danger" | "info"; children: React.ReactNode }) {
  return <span className={"badge " + (kind ?? "")}>{children}</span>;
}

export function Callout({ kind = "info", children, title }: { kind?: "info" | "warn" | "danger" | "ok"; title?: React.ReactNode; children?: React.ReactNode }) {
  const icon = kind === "warn" ? <AlertTriangle /> : kind === "danger" ? <XCircle /> : kind === "ok" ? <CheckCircle2 /> : <Info />;
  return (
    <div className={"callout " + kind}>
      {icon}
      <div className="col" style={{ gap: 2 }}>
        {title && <div className="bold">{title}</div>}
        {children && <div className="small muted">{children}</div>}
      </div>
    </div>
  );
}

export const Spinner = () => <div className="spinner" />;

export function Empty({ icon, title, children }: { icon?: React.ReactNode; title: React.ReactNode; children?: React.ReactNode }) {
  return (
    <div className="empty">
      {icon}
      <div className="bold" style={{ color: "var(--text-2)" }}>{title}</div>
      {children && <div className="small">{children}</div>}
    </div>
  );
}

export function Loading() {
  return <div className="loading-page"><Spinner /></div>;
}

export type MenuEntry = { label: string; icon?: React.ReactNode; onClick: () => void; danger?: boolean; disabled?: boolean } | "sep";

/** Dropdown menu opened from a button. */
export function MenuButton({ label, icon, items, variant, size, align = "left" }: {
  label?: React.ReactNode; icon?: React.ReactNode; items: MenuEntry[]; variant?: BtnProps["variant"]; size?: "sm"; align?: "left" | "right";
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLButtonElement>(null);
  return (
    <>
      <Button ref={ref} variant={variant} size={size} icon={icon} iconOnly={!label} onClick={() => setOpen(true)}>{label}</Button>
      {open && <Popover anchor={ref.current} items={items} onClose={() => setOpen(false)} align={align} />}
    </>
  );
}

export function Popover({ anchor, items, onClose, align = "left", at }: {
  anchor?: HTMLElement | null; items: MenuEntry[]; onClose: () => void; align?: "left" | "right"; at?: { x: number; y: number };
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ x: number; y: number }>({ x: -9999, y: -9999 });
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const w = el.offsetWidth, h = el.offsetHeight;
    let x = 0, y = 0;
    if (at) { x = at.x; y = at.y; }
    else if (anchor) {
      const r = anchor.getBoundingClientRect();
      x = align === "right" ? r.right - w : r.left;
      y = r.bottom + 4;
    }
    x = Math.max(8, Math.min(x, window.innerWidth - w - 8));
    if (y + h > window.innerHeight - 8) y = Math.max(8, (at ? at.y : anchor?.getBoundingClientRect().top ?? 0) - h - 4);
    setPos({ x, y });
  }, [anchor, at, align]);
  useEffect(() => {
    const d = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) onClose(); };
    const k = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    setTimeout(() => window.addEventListener("mousedown", d), 0);
    window.addEventListener("keydown", k);
    return () => { window.removeEventListener("mousedown", d); window.removeEventListener("keydown", k); };
  }, [onClose]);
  return createPortal(
    <div className="popover" ref={ref} style={{ left: pos.x, top: pos.y }}>
      {items.map((it, i) =>
        it === "sep" ? <div key={i} className="menu-sep" /> : (
          <button key={i} className={"menu-item " + (it.danger ? "danger" : "")} disabled={it.disabled}
            onClick={() => { onClose(); it.onClick(); }}>
            {it.icon}{it.label}
          </button>
        ),
      )}
    </div>,
    document.body,
  );
}

export function Stat({ label, value, icon }: { label: React.ReactNode; value: React.ReactNode; icon?: React.ReactNode }) {
  return (
    <div className="card stat">
      <div className="k">{icon}{label}</div>
      <div className="v">{value}</div>
    </div>
  );
}

/** Text input with a live character counter (UTF-16 units, like the game). */
export function CountedInput({ value, onChange, max, soft, multiline, rows, placeholder }: {
  value: string; onChange: (v: string) => void; max: number; soft?: number; multiline?: boolean; rows?: number; placeholder?: string;
}) {
  const units = (s: string) => s.length; // JS strings are UTF-16
  const n = units(value);
  const set = (v: string) => {
    if (units(v) > max) v = v.slice(0, max);
    onChange(v);
  };
  const cls = n > max ? "danger" : soft && n > soft ? "warn" : "";
  return (
    <div className="col" style={{ gap: 4 }}>
      {multiline ? (
        <textarea className="textarea" rows={rows ?? 4} value={value} placeholder={placeholder} onChange={(e) => set(e.target.value)} spellCheck={false} />
      ) : (
        <input className="input" value={value} placeholder={placeholder} onChange={(e) => set(e.target.value)} spellCheck={false} />
      )}
      <div className={"hint tiny " + cls} style={{ textAlign: "right", color: cls ? undefined : "var(--text-3)" }}>
        {n}/{max}{soft && n > soft ? ` · may not fit on one line in game (≈${soft})` : ""}
      </div>
    </div>
  );
}
