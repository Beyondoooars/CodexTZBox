#!/usr/bin/env node
// 发布编排：本地触发云端发布工作流，并跟踪到 Release 产出。
//
// 背景：正式产物只在 GitHub Actions 里构建，本地不做发布。但「发一个新版本」
// 需要好几步（确认工作区干净、对齐远端、触发 workflow、等两个 job、核对产物），
// 手动点容易漏。这个脚本把它固化成一条命令。
//
// 用法：
//   node scripts/release.mjs patch          # 0.1.4 -> 0.1.5
//   node scripts/release.mjs minor
//   node scripts/release.mjs major
//   node scripts/release.mjs 0.2.3          # 指定版本
//   node scripts/release.mjs patch --dry-run
//
// 选项：
//   --dry-run        只做前置检查并打印将要做什么，不触发
//   --timeout <分钟>  等待上限，默认 45
//
// 退出码：0 成功 / 1 失败 / 2 前置检查未通过

import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const WORKFLOW = 'build-desktop.yml'

// ---------------------------------------------------------------- 参数

const args = process.argv.slice(2)
const dryRun = args.includes('--dry-run')

// --timeout 后面跟的是它的值，不能当成位置参数。
const timeoutIndex = args.indexOf('--timeout')
const timeoutMinutes = timeoutIndex >= 0 ? Number(args[timeoutIndex + 1]) : 45
if (timeoutIndex >= 0 && (!Number.isFinite(timeoutMinutes) || timeoutMinutes <= 0)) {
  fail('--timeout 需要一个正整数分钟数')
}

const positional = args.filter(
  (a, i) => !a.startsWith('--') && !(timeoutIndex >= 0 && i === timeoutIndex + 1),
)
const bump = (positional[0] || 'patch').trim()
if (!/^(patch|minor|major|\d+\.\d+\.\d+)$/i.test(bump)) {
  fail(`无效的版本参数「${bump}」；请使用 patch、minor、major 或 X.Y.Z`)
}

// ---------------------------------------------------------------- 工具

function fail(message, code = 1) {
  console.error(`\x1b[31m✗\x1b[0m ${message}`)
  process.exit(code)
}

function info(message) {
  console.log(`\x1b[36m›\x1b[0m ${message}`)
}

function ok(message) {
  console.log(`\x1b[32m✓\x1b[0m ${message}`)
}

/** 运行命令并返回 stdout；失败时抛出带 stderr 的错误。
    不开 shell：gh / git 都是可执行文件，Windows 的 CreateProcess 会自动补
    .exe；开 shell 反而要处理转义，Node 也会告警。 */
function run(cmd, cmdArgs, options = {}) {
  const result = spawnSync(cmd, cmdArgs, {
    cwd: root,
    encoding: 'utf8',
    ...options,
  })
  if (result.error) {
    throw new Error(`${cmd} 无法执行：${result.error.message}${cmd === 'gh' ? '（是否已安装并登录？）' : ''}`)
  }
  if (result.status !== 0 && !options.allowFailure) {
    const detail = (result.stderr || result.stdout || '').trim()
    throw new Error(`${cmd} ${cmdArgs.join(' ')} 失败${detail ? `：${detail}` : ''}`)
  }
  return (result.stdout || '').trim()
}

function gh(ghArgs, options = {}) {
  return run('gh', ghArgs, options)
}

/** 本地用 bump-version.mjs 预演结果，不修改任何文件。 */
function previewVersion() {
  const pkg = JSON.parse(fs.readFileSync(path.join(root, 'resource/package.json'), 'utf8'))
  const [major, minor, patch] = pkg.version.split('.').map((n) => parseInt(n, 10) || 0)
  const arg = bump.toLowerCase().replace(/^v/, '')
  if (arg === 'patch') return `${major}.${minor}.${patch + 1}`
  if (arg === 'minor') return `${major}.${minor + 1}.0`
  if (arg === 'major') return `${major + 1}.0.0`
  return arg
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/** main 上最近一次手动触发（workflow_dispatch）的运行 id，没有则返回空串。 */
function latestDispatchRunId() {
  const raw = gh([
    'run', 'list',
    '--workflow', WORKFLOW,
    '--branch', 'main',
    '--event', 'workflow_dispatch',
    '--limit', '1',
    '--json', 'databaseId',
  ], { allowFailure: true })
  if (!raw) return ''
  const [latest] = JSON.parse(raw)
  return latest?.databaseId ? String(latest.databaseId) : ''
}

// ---------------------------------------------------------------- 前置检查

function preflight() {
  const problems = []

  if (process.platform !== 'win32' && process.platform !== 'linux' && process.platform !== 'darwin') {
    problems.push(`不支持的系统：${process.platform}`)
  }

  // gh 可用且已登录
  try {
    gh(['auth', 'status'])
  } catch {
    problems.push('gh 未登录或不可用，请先运行 gh auth login')
  }

  // 推送 workflow 文件需要 workflow scope，缺了会在最后一步才被 GitHub 拒绝，
  // 而那时提交已经做完了。这里提前拦下来。
  try {
    const status = spawnSync('gh', ['auth', 'status'], { cwd: root, encoding: 'utf8' })
    const text = `${status.stdout || ''}${status.stderr || ''}`
    if (/Token scopes:/.test(text) && !/'workflow'/.test(text)) {
      problems.push(
        '当前 gh 凭据缺少 workflow scope，无法推送 .github/workflows/ 下的改动。\n' +
        '    修复：gh auth refresh -s workflow',
      )
    }
  } catch {
    /* 上面的检查已经报过 gh 不可用 */
  }

  // 在 main 分支
  let branch = ''
  try {
    branch = run('git', ['rev-parse', '--abbrev-ref', 'HEAD'])
  } catch {
    problems.push('当前目录不是 git 仓库')
  }
  if (branch && branch !== 'main') {
    problems.push(`当前在 ${branch} 分支；发布必须从 main 触发`)
  }

  // 工作区干净（未跟踪文件不算，构建产物本来就被忽略）
  try {
    const dirty = run('git', ['status', '--porcelain', '--untracked-files=no'])
    if (dirty) problems.push(`工作区有未提交的改动：\n${dirty}`)
  } catch {
    /* 上面已记录 */
  }

  // 与远端同步
  if (branch === 'main') {
    try {
      run('git', ['fetch', 'origin', 'main', '--quiet'])
      const behind = run('git', ['rev-list', '--count', 'HEAD..origin/main'])
      const ahead = run('git', ['rev-list', '--count', 'origin/main..HEAD'])
      if (Number(behind) > 0) problems.push(`本地落后 origin/main ${behind} 个提交，请先 git pull --ff-only`)
      if (Number(ahead) > 0) problems.push(`本地领先 origin/main ${ahead} 个提交，请先推送`)
    } catch {
      problems.push('无法访问远端 origin/main')
    }
  }

  // 目标标签不能已存在
  const target = previewVersion()
  try {
    const existing = gh(['release', 'view', `v${target}`, '--json', 'tagName'], { allowFailure: true })
    if (existing) problems.push(`Release v${target} 已存在，请换一个版本号`)
  } catch {
    /* 不存在时 gh 以非 0 退出，正是我们要的；真正的异常留给云端报错 */
  }

  if (problems.length) {
    console.error('\x1b[31m前置检查未通过：\x1b[0m')
    for (const p of problems) console.error(`  · ${p}`)
    process.exit(2)
  }
  return target
}

// ---------------------------------------------------------------- 主流程

async function main() {
  const current = JSON.parse(fs.readFileSync(path.join(root, 'resource/package.json'), 'utf8')).version
  const target = previewVersion()

  info(`当前版本 v${current}，目标版本 v${target}`)

  // workflow_dispatch 触发的是「默认分支上的」工作流定义。本地领先时，
  // 发出去的一定是旧定义，等于白发布一次。
  try {
    const ahead = Number(run('git', ['rev-list', '--count', 'origin/main..HEAD']))
    if (ahead > 0) {
      fail(`本地领先 origin/main ${ahead} 个提交。发布必须基于已推送的代码，请先 git push。`)
    }
  } catch {
    /* preflight 会再查一次远端同步 */
  }

  preflight()
  ok('前置检查通过')

  if (dryRun) {
    ok(`--dry-run：将触发 ${WORKFLOW}，version=${bump}`)
    return
  }

  // 触发。workflow_dispatch 的 workflow 必须已在默认分支上存在。
  // 先记下当前最新一次运行：dispatch 的 run id 不会从 gh 返回，只能靠
  // 「触发前不存在的那个」来认领，不能盲取最新一条（那可能是别人的构建）。
  const before = latestDispatchRunId()

  info(`触发 ${WORKFLOW}（version=${bump}）…`)
  gh(['workflow', 'run', WORKFLOW, '--ref', 'main', '-f', `version=${bump}`])

  let runId = ''
  for (let i = 0; i < 20; i += 1) {
    await sleep(3000)
    const latest = latestDispatchRunId()
    if (latest && latest !== before) { runId = latest; break }
  }
  if (!runId) fail('触发了工作流但找不到对应的运行记录')
  ok(`已触发，运行 ID ${runId}`)

  // 跟踪状态
  const deadline = Date.now() + timeoutMinutes * 60_000
  let finished = false
  let lastStatus = ''
  while (!finished && Date.now() < deadline) {
    const raw = gh(['run', 'view', runId, '--json', 'status,conclusion,jobs'], { allowFailure: true })
    if (raw) {
      const data = JSON.parse(raw)
      const status = data.status === 'completed' ? `${data.status}/${data.conclusion}` : data.status
      if (status !== lastStatus) {
        lastStatus = status
        info(`运行状态：${status}`)
      }
      if (data.status === 'completed') {
        finished = true
        if (data.conclusion !== 'success') {
          console.error('\n各 job 结果：')
          for (const job of data.jobs || []) {
            console.error(`  · ${job.name}: ${job.conclusion || job.status}`)
          }
          fail(`发布失败（${data.conclusion}）。查看详情：gh run view ${runId} --log-failed`)
        }
      }
    }
    if (!finished) await sleep(15_000)
  }

  if (!finished) {
    fail(`等待超时（${timeoutMinutes} 分钟）。用 gh run view ${runId} --web 查看进度`)
  }

  // 核对 Release 产物
  info(`核对 v${target} 的 Release 资产…`)
  let release = ''
  for (let i = 0; i < 10; i += 1) {
    release = gh(['release', 'view', `v${target}`, '--json', 'tagName,assets,url'], { allowFailure: true })
    if (release) break
    await sleep(5000)
  }
  if (!release) fail(`运行成功但找不到 v${target} 的 Release`)

  const { assets, url } = JSON.parse(release)
  const names = (assets || []).map((a) => a.name)
  for (const required of ['CodexTZBox.exe', 'SHA256SUMS.txt']) {
    if (!names.includes(required)) fail(`Release 缺少资产 ${required}`)
  }

  ok(`发布完成：v${target}`)
  console.log(`  资产：${names.join('、')}`)
  console.log(`  地址：${url}`)
}

main().catch((error) => fail(error.message))
