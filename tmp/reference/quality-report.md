# Quality Report

Last refined: 2026-07-03.

Scope: `tmp/reference/**`. This report records editorial standards and current
validation commands for the reference folder. It intentionally avoids volatile
inventory counts.

## Current Status

| Check | Status |
|---|---|
| Local Markdown links | Validate with the Node command in [Validation Commands](#validation-commands). |
| Captured-source language | Use provenance labels; do not imply captured labels are local files. |
| Count-heavy claims | Avoid unless regenerated in the same pass and marked as local. |
| Implementation status | Phrase absence as "no equivalent identified in the local pass" unless source inspection proves more. |
| Examples | Keep as adaptation sketches unless backed by an existing fixture or test. |

## Editorial Standard

- Keep indexes short and self-contained.
- Prefer local links over unsupported assumptions.
- Use `Signal`, `Store`, `Cell`, and `Graph` for normalized design language;
  use `Engram`, `Substrate`, and other captured terms only when quoting source
  labels.
- Mark proposed crates/modules as proposed until they exist.
- Do not claim VCG truthfulness, crash safety, tamper evidence, cost savings,
  or model-quality gains without local implementation and benchmark evidence.
- For security, listener, auth, secret, sandbox, approval, or network claims,
  point to the owning subsystem docs before implementation.

## Continued Attention

| Area | Watch for |
|---|---|
| Routing and cost | Learned routing must not override privacy or high-risk rules. |
| Memory dedup | HDC similarity should produce candidates before hard merges. |
| Background learning | Derived memories need taint, source links, confidence, and budget caps. |
| Gate verification | Caller-level tests are required when a helper gates a side effect. |
| Web projections | SSE/WebSocket reconnect behavior needs idempotency coverage. |
| Captured citations | Bibliographic details should be verified before public citation use. |

## Validation Commands

```sh
node - <<'NODE'
const fs = require('fs');
const path = require('path');
const files = fs.readdirSync('tmp/reference', {recursive: true})
  .filter(f => f.endsWith('.md'))
  .map(f => path.join('tmp/reference', f));
let broken = [];
for (const file of files) {
  const text = fs.readFileSync(file, 'utf8');
  const re = /\[[^\]]*\]\(([^)]+)\)/g;
  for (let m; (m = re.exec(text)); ) {
    const raw = m[1].split('#')[0];
    if (!raw || /^[a-z]+:/i.test(raw) || raw.startsWith('mailto:')) continue;
    const resolved = path.normalize(path.join(path.dirname(file), decodeURIComponent(raw)));
    if (!fs.existsSync(resolved)) broken.push(`${file}: ${m[1]} -> ${resolved}`);
  }
}
if (broken.length) {
  console.error(broken.join('\n'));
  process.exit(1);
}
console.log(`checked ${files.length} markdown files`);
NODE

rg -n "todo marker|fixme marker" tmp/reference
git diff --check -- tmp/reference
```
