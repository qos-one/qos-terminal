"""Regenerate src/api_demo.rs from the server functions in src/api.rs (web demo client, see ../demo).

Run after adding or changing a server function:  python3 scripts/gen_api_demo.py
It also makes sure every server function in api.rs is excluded from the demo build.
"""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
API = ROOT / "src" / "api.rs"
OUT = ROOT / "src" / "api_demo.rs"
GUARD = '#[cfg(not(feature = "demo"))]'

HEADER = '''// Generated from api.rs by scripts/gen_api_demo.py — do not edit by hand.
// Web demo only: every function POSTs {argument name: value} to the same path as the server function and reads JSON back.

/// error type for the web demo (stands in for the fullstack ServerFnError)
#[derive(Debug, Clone, PartialEq)]
pub struct ServerFnError(pub String);
impl ServerFnError { pub fn new(m: impl std::fmt::Display) -> Self { ServerFnError(m.to_string()) } }
impl std::fmt::Display for ServerFnError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) } }
impl std::error::Error for ServerFnError {}

async fn demo_call<T: serde::de::DeserializeOwned>(path: &str, args: serde_json::Value) -> Result<T, ServerFnError> {
    let req = gloo_net::http::Request::post(path).header("x-qos-demo", "1").json(&args).map_err(ServerFnError::new)?;
    let res = req.send().await.map_err(|e| ServerFnError::new(format!("demo server not responding: {e}")))?;
    let ok = res.ok();
    let text = res.text().await.map_err(ServerFnError::new)?;
    if !ok {
        let msg = serde_json::from_str::<serde_json::Value>(&text).ok().and_then(|v| v.get("message").and_then(|m| m.as_str().map(String::from))).unwrap_or(text);
        return Err(ServerFnError(msg));
    }
    serde_json::from_str(if text.is_empty() { "null" } else { &text }).map_err(|e| ServerFnError::new(format!("{path}: {e}")))
}
'''


def main() -> None:
    lines = API.read_text().split("\n")
    out, fns = [], []
    for i, line in enumerate(lines):
        m = re.match(r'#\[(get|post)\("([^"]+)"\)\]$', line)
        if m:
            if not (out and out[-1] == GUARD):
                out.append(GUARD)
            sig = re.match(r"pub async fn (\w+)\((.*)\) -> (Result<.*>) \{", lines[i + 1])
            if not sig:
                raise SystemExit(f"api.rs:{i + 2}: server function signature must be on one line")
            name, args, ret = sig.groups()
            names = [a.split(":")[0].strip() for a in args.split(",") if a.strip()]
            body = "serde_json::json!({" + ", ".join(f'"{n}": {n}' for n in names) + "})"
            fns.append(f'pub async fn {name}({args}) -> {ret} {{ demo_call("{m.group(2)}", {body}).await }}')
        out.append(line)
    text = "\n".join(out)
    if 'include!("api_demo.rs")' not in text:
        text = text.rstrip("\n") + '\n\n// web demo (../demo): the same functions, calling the demo backend instead of server functions\n#[cfg(feature = "demo")]\ninclude!("api_demo.rs");\n'
    API.write_text(text)
    OUT.write_text(HEADER + "\n" + "\n".join(fns) + "\n")
    print(f"{len(fns)} functions → {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
