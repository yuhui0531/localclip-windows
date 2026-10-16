// 代码识别与普通文本边界回归；只使用生成的正文。by AI.Coding
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { build } from 'esbuild';
const root = await mkdtemp(join(tmpdir(), 'localclip-preview-'));
try {
  const output = join(root, 'preview.mjs');
  await build({ entryPoints: [new URL('../src/preview.tsx', import.meta.url).pathname], bundle: true, platform: 'node', format: 'esm', outfile: output });
  const { highlighted, sizeLabel } = await import(pathToFileURL(output).href);
  for (const text of ['function generated() { return true; }', 'fn main() { println!("hello"); }', '{"generated":true,"index":1}', 'SELECT * FROM users WHERE id = 1;']) {
    assert.ok(highlighted(text).language);
    assert.match(highlighted(text).html, /hljs-/);
  }
  for (const text of ['C:\\Users\\Public\\sample.rs', '/tmp/generated.rs\n/tmp/another.py', 'https://example.com/path?a=1', '这是一段正常的中文文字，不应该识别为代码。']) {
    assert.equal(highlighted(text).language, undefined);
  }
  assert.equal(sizeLabel({contentCount:1,contentBytes:10,unknownCount:1,folderCount:0}), '大小未知');
  assert.equal(sizeLabel({contentCount:1,contentBytes:68,unknownCount:0,folderCount:0}), '68 B');
  console.log('10 preview/highlighting cases passed');
} finally {
  await rm(root, { recursive: true, force: true });
}
