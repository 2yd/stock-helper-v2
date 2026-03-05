#!/usr/bin/env node

/**
 * 一键同步修改所有版本号
 *
 * 用法:
 *   npm run bump 0.2.0
 *   npm run bump patch       # 0.1.0 → 0.1.1
 *   npm run bump minor       # 0.1.0 → 0.2.0
 *   npm run bump major       # 0.1.0 → 1.0.0
 *
 * 会同时修改:
 *   - package.json
 *   - src-tauri/tauri.conf.json
 *   - src-tauri/Cargo.toml
 */

import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, '..');

const arg = process.argv[2];

if (!arg) {
  console.error('用法: npm run bump <version|patch|minor|major>');
  console.error('示例: npm run bump 0.2.0');
  console.error('      npm run bump patch');
  process.exit(1);
}

// 读取当前版本
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf-8'));
const currentVersion = pkg.version;
const [major, minor, patch] = currentVersion.split('.').map(Number);

// 计算新版本
let newVersion;
switch (arg) {
  case 'patch':
    newVersion = `${major}.${minor}.${patch + 1}`;
    break;
  case 'minor':
    newVersion = `${major}.${minor + 1}.0`;
    break;
  case 'major':
    newVersion = `${major + 1}.0.0`;
    break;
  default:
    if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(arg)) {
      console.error(`无效的版本号: ${arg}`);
      process.exit(1);
    }
    newVersion = arg;
}

console.log(`\n📦 版本升级: ${currentVersion} → ${newVersion}\n`);

// 1. package.json
const pkgPath = resolve(root, 'package.json');
pkg.version = newVersion;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
console.log(`  ✅ package.json`);

// 2. src-tauri/tauri.conf.json
const tauriConfPath = resolve(root, 'src-tauri/tauri.conf.json');
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
console.log(`  ✅ src-tauri/tauri.conf.json`);

// 3. src-tauri/Cargo.toml (只替换 [package] 下的 version)
const cargoPath = resolve(root, 'src-tauri/Cargo.toml');
let cargo = readFileSync(cargoPath, 'utf-8');
// 去掉预发布后缀写入 Cargo.toml（Cargo 版本号有自己的规范）
const cargoVersion = newVersion.replace(/-.*$/, '');
cargo = cargo.replace(
  /^(version\s*=\s*)"[^"]*"/m,
  `$1"${cargoVersion}"`
);
writeFileSync(cargoPath, cargo);
console.log(`  ✅ src-tauri/Cargo.toml${cargoVersion !== newVersion ? ` (${cargoVersion})` : ''}`);

console.log(`\n🎉 完成! 记得提交后合并到 master 即可自动发布。\n`);
