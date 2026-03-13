import os from 'os';
import path from 'path';
import { spawn, spawnSync } from 'child_process';
import { fileURLToPath } from 'url';

// keep track of the tauri-driver process we start
let tauriDriver;
let exit = false;

const __dirname = fileURLToPath(new URL('.', import.meta.url));

export const mochaHooks = {
  async beforeAll() {
    // set timeout to 2 minutes to allow the program to build if it needs to
    this.timeout(120000);
  
    // ensure the app has been built
    spawnSync('bun', ['run', 'tauri', 'build', '--config', 'src-tauri/tauri.conf.test.json', '--debug', '--no-bundle'], {
      cwd: path.resolve(__dirname, '..'),
      stdio: 'inherit',
      shell: true,
    });

    // start tauri-driver
    tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    );
    tauriDriver.on('error', (error) => {
      console.error('tauri-driver error:', error);
      process.exit(1);
    });
    tauriDriver.on('exit', (code) => {
      if (!exit) {
        console.error('tauri-driver exited with code:', code);
        process.exit(1);
      }
    });

  },

  async afterAll() {
    // stop the webdriver session
    await closeTauriDriver();
  }
};

async function closeTauriDriver() {
  exit = true;
  // kill the tauri-driver process
  tauriDriver.kill();
}

function onShutdown(fn) {
  const cleanup = () => {
    try {
      fn();
    } finally {
      process.exit();
    }
  };

  process.on('exit', cleanup);
  process.on('SIGINT', cleanup);
  process.on('SIGTERM', cleanup);
  process.on('SIGHUP', cleanup);
  process.on('SIGBREAK', cleanup);
}

onShutdown(() => {
  closeTauriDriver();
});
