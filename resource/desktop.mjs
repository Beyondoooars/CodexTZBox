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
  const artifact = join(repository, 'CodexTimeZoneLauncher.exe');
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
    const sourceExe = join(release, 'codextimezonerev.exe');

    const copyWithLockRetry = async (target) => {
      try {
        await copyFile(sourceExe, target);
      } catch (copyErr) {
        if (copyErr.code === 'EBUSY' || copyErr.code === 'EPERM') {
          console.log(`目标文件正在运行，正在关闭以更新：${target}…`);
          await run('taskkill.exe', ['/F', '/IM', 'CodexTimeZoneLauncher.exe']).catch(() => {});
          await new Promise(r => setTimeout(r, 500));
          await copyFile(sourceExe, target);
        } else {
          throw copyErr;
        }
      }
    };

    await copyWithLockRetry(artifact);
    console.log(`已生成：${artifact}`);

    const toolkitDir = process.env.CODEX_TOOLKIT_DIR || 'D:\\Toolkits\\CodexTZBox';
    try {
      await mkdir(toolkitDir, { recursive: true });
      const toolkitArtifact = join(toolkitDir, 'CodexTimeZoneLauncher.exe');
      await copyWithLockRetry(toolkitArtifact);
      console.log(`已同步至工具箱：${toolkitArtifact}`);
    } catch (err) {
      console.error(`同步至工具箱失败：${err.message}`);
    }
  }
}

main().catch(error => { console.error(error.message); process.exitCode = 1; });
