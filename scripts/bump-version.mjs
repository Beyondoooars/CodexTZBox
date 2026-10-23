// 发布时在 CI 里递增版本号：四处版本字段（package.json、package-lock.json、
// tauri.conf.json、Cargo.toml）与 Cargo.lock 里的自身条目必须一起改，
// 否则 `tauri build -- --locked` 会因为 lock 与 manifest 不一致而失败。
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const resource = path.join(root, 'resource')

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'))
}

function writeJson(file, value) {
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}

const packagePath = path.join(resource, 'package.json')
const current = readJson(packagePath).version
const raw = (process.argv[2] || 'patch').trim().toLowerCase()
const arg = raw.replace(/^v/, '')
const [major, minor, patch] = current.split('.').map((n) => parseInt(n, 10) || 0)

let target
if (arg === 'patch') target = `${major}.${minor}.${patch + 1}`
else if (arg === 'minor') target = `${major}.${minor + 1}.0`
else if (arg === 'major') target = `${major + 1}.0.0`
else if (/^\d+\.\d+\.\d+$/.test(arg)) target = arg
else {
  console.error(`[bump-version] 无效的版本参数：${raw}；请使用 patch、minor、major 或 X.Y.Z`)
  process.exit(1)
}

console.log(`[bump-version] v${current} -> v${target}`)

const pkg = readJson(packagePath)
pkg.version = target
writeJson(packagePath, pkg)

const lockPath = path.join(resource, 'package-lock.json')
if (fs.existsSync(lockPath)) {
  const npmLock = readJson(lockPath)
  npmLock.version = target
  if (npmLock.packages && npmLock.packages['']) npmLock.packages[''].version = target
  writeJson(lockPath, npmLock)
}

const tauriPath = path.join(resource, 'src-tauri', 'tauri.conf.json')
const tauriConf = readJson(tauriPath)
tauriConf.version = target
writeJson(tauriPath, tauriConf)

const cargoPath = path.join(resource, 'src-tauri', 'Cargo.toml')
let cargo = fs.readFileSync(cargoPath, 'utf8')
if (!/^version = ".*?"/m.test(cargo)) {
  console.error('[bump-version] Cargo.toml 中找不到版本号')
  process.exit(1)
}
cargo = cargo.replace(/^version = ".*?"/m, `version = "${target}"`)
fs.writeFileSync(cargoPath, cargo, 'utf8')

// Cargo.lock 只改 codextzbox 自己那一条，依赖的版本必须保持原样。
const cargoLockPath = path.join(resource, 'src-tauri', 'Cargo.lock')
let cargoLock = fs.readFileSync(cargoLockPath, 'utf8')
const selfEntry = /(\[\[package\]\]\nname = "codextzbox"\nversion = ")[^"]+(")/
if (!selfEntry.test(cargoLock)) {
  console.error('[bump-version] Cargo.lock 中找不到 codextzbox 条目')
  process.exit(1)
}
cargoLock = cargoLock.replace(selfEntry, `$1${target}$2`)
fs.writeFileSync(cargoLockPath, cargoLock, 'utf8')

console.log('[bump-version] 已同步 package.json / package-lock.json / tauri.conf.json / Cargo.toml / Cargo.lock')
