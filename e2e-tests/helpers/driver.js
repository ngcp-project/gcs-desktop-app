import path from 'path';
import { Builder, Capabilities } from 'selenium-webdriver';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

const application = path.resolve(
  __dirname,
  '..',
  '..',
  'src-tauri',
  'target',
  'debug',
  'Interface'
);

export async function createDriver() {
  const capabilities = new Capabilities();
  capabilities.set('tauri:options', { application });
  capabilities.setBrowserName('wry');

  // start the webdriver client
  return new Builder()
    .withCapabilities(capabilities)
    .usingServer('http://127.0.0.1:4444/')
    .build();
};
