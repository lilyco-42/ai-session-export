import fs from 'node:fs';
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const WebSocket = require('/home/lilyco/code/ego-lite-linux/package/ego-linux-host/node_modules/ws');
const endpoint = 'http://127.0.0.1:9222/json/new?http://127.0.0.1:5188/';
const response = await fetch(endpoint, { method: 'PUT' });
if (!response.ok) throw new Error('new tab failed: ' + response.status);
const target = await response.json();
const ws = new WebSocket(target.webSocketDebuggerUrl);
let id = 0;
const pending = new Map();
const events = [];
ws.on('message', raw => {
  const message = JSON.parse(raw.toString());
  if (message.id && pending.has(message.id)) {
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(JSON.stringify(message.error))); else resolve(message.result);
  } else {
    events.push(message);
  }
});
await new Promise((resolve, reject) => { ws.once('open', resolve); ws.once('error', reject); });
const call = (method, params = {}) => new Promise((resolve, reject) => {
  const callId = ++id;
  pending.set(callId, { resolve, reject });
  ws.send(JSON.stringify({ id: callId, method, params }));
});
const evaluate = async expression => {
  const result = await call('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
};
await call('Runtime.enable');
await call('Page.enable');
await call('Emulation.setDeviceMetricsOverride', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
await call('Page.navigate', { url: 'http://127.0.0.1:5188/' });
await new Promise(resolve => setTimeout(resolve, 3500));
const sourceText = await evaluate('document.body.innerText.slice(0, 1200)');
const clickExport = await evaluate('(function(){var candidates=[].slice.call(document.querySelectorAll("button"));var target=candidates.find(function(button){return button.textContent && button.textContent.includes("导出队列");});if(!target)return false;target.click();return true;})()');
await new Promise(resolve => setTimeout(resolve, 1500));
const exportText = await evaluate('document.body.innerText.slice(0, 2500)');
const metrics = await evaluate('({scrollWidth: document.documentElement.scrollWidth, innerWidth: window.innerWidth, rows: document.querySelectorAll("tbody tr").length})');
const file = '/home/lilyco/文档/deepseek-harness/default-workspace/ai-session-export/evidence/cdp-live-1440.png';
const shot = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
fs.writeFileSync(file, Buffer.from(shot.data, 'base64'));
console.log(JSON.stringify({ sourceText, clickExport, exportText, metrics, screenshot: file }, null, 2));
ws.close();
