import { spawn } from 'node:child_process';
import { copyFile, lstat, mkdir } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const resource = dirname(fileURLToPath(import.meta.url));
const repository = dirname(resource);
const environment = join(repository, 'environment');
const args = process.argv.slice(2);
const action = args[0] || 'build';
const configured = args.includes('--configured');

function run(command, commandArgs, env = process.env) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, commandArgs, { cwd: resource, env, stdio: 'inherit' });
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0) resolve();
      else reject(new Error(`${command} 执行失败：${signal || code}`));
    });
  });
}

async function main() {
  if (!['build', 'dev', 'start'].includes(action)) {
    throw new Error('请使用 npm run build、dev 或 start。');
  }
  if (process.platform !== 'win32') {
    throw new Error(`当前程序仅面向 Windows 系统构建与运行；当前系统为 ${process.platform}。`);
  }
  const artifact = join(repository, 'CodexTZBox.exe');
  if (action === 'start') {
    await lstat(artifact).catch(() => { throw new Error('请先执行 npm run build。'); });
    const child = spawn(artifact, [], { cwd: repository, detached: true, stdio: 'ignore' });
    await new Promise((resolve, reject) => { child.once('spawn', resolve); child.once('error', reject); });
    child.unref();
    return;
  }
  if (!configured) {
    await run('powershell.exe', ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', join(resource, 'backend/win/desktop.ps1'), action]);
    return;
  }
  const env = { ...process.env };
  env.npm_config_cache = join(environment, 'npm-cache');
  env.CARGO_HOME = join(environment, 'cargo');
  env.CARGO_TARGET_DIR = join(environment, 'cargo-target');
  if (env.CODEX_TZ_PROXY) env.HTTP_PROXY = env.HTTPS_PROXY = env.CODEX_TZ_PROXY;
  if (env.CARGO_BUILD_TARGET) throw new Error('平台构建使用本机目标，请先取消 CARGO_BUILD_TARGET。');
  const cli = join(resource, 'node_modules/@tauri-apps/cli/tauri.js');
  await lstat(cli).catch(() => { throw new Error('缺少依赖，请在 resource 目录先执行 npm install。'); });
  const icons = ['icons/icon.ico'];
  const tauriArgs = [cli, action, '--config', JSON.stringify({ bundle: { icon: icons } })];
  if (action === 'build') tauriArgs.push('--no-bundle', '--', '--locked');
  if (action === 'dev') {
    tauriArgs.push(
      '--additional-watch-folders', join(resource, 'backend/win'),
      '--additional-watch-folders', join(resource, 'backend/common.rs'),
      '--additional-watch-folders', join(resource, 'backend/zones.json')
    );
  }
  await run(process.execPath, tauriArgs, env);
  if (action === 'build') {
    const release = join(env.CARGO_TARGET_DIR, 'release');
    const sourceExe = join(release, 'codextzbox.exe');

    const copyWithLockRetry = async (target) => {
      // 目标被占用时最多尝试 5 次，但绝不强制结束进程：Windows 会锁住运行中 exe 的镜像，
      // 强制 kill 既可能丢配置，也与产品承诺的“绝不强杀”一致。用户需自行退出成品。
      for (let attempt = 1; attempt <= 5; attempt += 1) {
        try {
          await copyFile(sourceExe, target);
          return;
        } catch (copyErr) {
          if (copyErr.code !== 'EBUSY' && copyErr.code !== 'EPERM') throw copyErr;
          if (attempt === 5) {
            throw new Error(`目标文件被占用（已尝试 5 次），请先退出正在运行的 CodexTZBox.exe 后重试：${target}`);
          }
          console.log(`目标文件正在使用，稍后重试（${attempt}/5）：${target}…`);
          await new Promise(r => setTimeout(r, 1500));
        }
      }
    };

    await copyWithLockRetry(artifact);
    console.log(`已生成：${artifact}`);

    // 只在显式设置 CODEX_TOOLKIT_DIR 时才额外同步一份：程序里不写死任何本机路径，
    // 换机器、CI 构建或开源分发时行为都一致；未设置就只产出仓库根目录的 exe。
    const toolkitDir = (process.env.CODEX_TOOLKIT_DIR || '').trim();
    if (toolkitDir) {
      // 工具箱里的成品是用户实际运行的那份，同步失败必须让构建失败：
      // 只打印一行警告会让 build 报成功、工具箱却留着旧版本。
      const toolkitArtifact = join(toolkitDir, 'CodexTZBox.exe');
      await mkdir(toolkitDir, { recursive: true });
      await copyWithLockRetry(toolkitArtifact);
      console.log(`已同步至工具箱：${toolkitArtifact}`);
    }
  }
}

main().catch(error => { console.error(error.message); process.exitCode = 1; });
