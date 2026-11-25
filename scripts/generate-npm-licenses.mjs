// 生成 npm 生产依赖的开源许可证清单（HTML），供打包时附带。
// 输出: src-tauri/third-party-licenses/npm.html
import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync, realpathSync } from 'node:fs';
import { join, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const nodeModules = join(root, 'node_modules');
const outDir = join(root, 'src-tauri', 'third-party-licenses');

const LICENSE_FILES = [
  'LICENSE', 'LICENSE.md', 'LICENSE.txt', 'LICENSE-MIT', 'LICENCE', 'LICENCE.md',
  'LICENSE.APACHE', 'LICENSE.BSD', 'COPYING', 'COPYING.md', 'NOTICE', 'NOTICE.md',
];

// 只遍历真实的包目录（pnpm 的实际包都在 .pnpm 下，顶层是符号链接）
const seen = new Map(); // realpath -> {name, version, license, homepage, licenseText}

function readLicenseFile(pkgDir) {
  for (const f of LICENSE_FILES) {
    const p = join(pkgDir, f);
    if (existsSync(p)) return readFileSync(p, 'utf8');
  }
  return null;
}

function collect(pkgDir) {
  const pkgJsonPath = join(pkgDir, 'package.json');
  if (!existsSync(pkgJsonPath)) return;
  let pkg;
  try {
    pkg = JSON.parse(readFileSync(pkgJsonPath, 'utf8'));
  } catch {
    return;
  }
  if (!pkg.name || !pkg.license) return;
  let real;
  try {
    real = realpathSync(pkgDir);
  } catch {
    real = pkgDir;
  }
  if (seen.has(real)) return;
  seen.set(real, {
    name: pkg.name,
    version: pkg.version ?? '',
    license: Array.isArray(pkg.license) ? pkg.license.join(', ') : pkg.license,
    homepage: pkg.homepage ?? pkg.repository?.url ?? '',
    licenseText: readLicenseFile(pkgDir),
  });
}

function walk(dir, depth = 0) {
  let entries;
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return;
  }
  for (const e of entries) {
    if (e.name === '.bin' || e.name === '.pnpm-store') continue;
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name.startsWith('@')) {
        walk(p, depth); // @scope/pkg
      } else if (depth === 0 || dir.includes('.pnpm')) {
        collect(p);
        if (p.includes('.pnpm')) walk(p, depth + 1); // .pnpm/<pkg>/node_modules/<dep>
      }
    } else if (e.isSymbolicLink()) {
      try {
        collect(realpathSync(p));
      } catch {
        /* 悬空链接 */
      }
    }
  }
}

walk(nodeModules);

const pkgs = [...seen.values()].sort((a, b) =>
  a.name.localeCompare(b.name) || a.version.localeCompare(b.version)
);

const esc = (s) =>
  String(s ?? '').replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');

const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>smtc2web — Third-party npm licenses</title>
<style>body{font-family:sans-serif;max-width:960px;margin:2rem auto;padding:0 1rem;line-height:1.5}h2{border-bottom:1px solid #ccc;padding-bottom:.2rem;margin-bottom:.5rem}pre{background:#f6f6f6;padding:1rem;overflow-x:auto;font-size:.8rem;white-space:pre-wrap}</style>
</head>
<body>
<h1>smtc2web — Third-party npm dependency licenses</h1>
<p>This application uses the following npm packages (${pkgs.length} packages). Their licenses are reproduced below.</p>
${pkgs
  .map(
    (p) => `<section>
<h2>${esc(p.name)} ${esc(p.version)}</h2>
<p>License: <strong>${esc(p.license)}</strong>${p.homepage ? `<br>Homepage: ${esc(p.homepage)}` : ''}</p>
${p.licenseText ? `<pre>${esc(p.licenseText)}</pre>` : ''}
</section>`
  )
  .join('\n')}
</body>
</html>
`;

mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, 'npm.html'), html);
console.log(`npm licenses: wrote ${pkgs.length} packages to src-tauri/third-party-licenses/npm.html`);
