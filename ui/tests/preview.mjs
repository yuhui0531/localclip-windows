// 代码识别与普通文本边界回归；只使用生成的正文。by AI.Coding
import assert from 'node:assert/strict';
import {renderToStaticMarkup} from 'react-dom/server';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { build } from 'esbuild';
const root = await mkdtemp(join(tmpdir(), 'localclip-preview-'));
try {
  const output = join(root, 'preview.mjs');
  await build({ entryPoints: [new URL('../src/preview.tsx', import.meta.url).pathname], bundle: true, platform: 'node', format: 'esm', outfile: output });
  const { highlighted, sizeLabel, TextPreview, ContentPreview } = await import(pathToFileURL(output).href);
  for (const text of ['function generated() { return true; }', 'fn main() { println!("hello"); }', '{"generated":true,"index":1}', 'SELECT * FROM users WHERE id = 1;']) {
    assert.ok(highlighted(text).language);
    assert.match(highlighted(text).html, /hljs-/);
  }
  for (const text of ['C:\\Users\\Public\\sample.rs', '/tmp/generated.rs\n/tmp/another.py', 'https://example.com/path?a=1', '这是一段正常的中文文字，不应该识别为代码。']) {
    assert.equal(highlighted(text).language, undefined);
  }
  assert.equal(sizeLabel({contentCount:1,contentBytes:10,unknownCount:1,folderCount:0}), '大小未知');
  assert.equal(sizeLabel({contentCount:1,contentBytes:68,unknownCount:0,folderCount:0}), '68 B');
  // 长代码必须同时保留高亮前缀与原始全文；尾部作为文本转义，不能变成可执行 HTML。
  const long = 'function generated() { return true; }\n'.repeat(700) + '<script>END_OF_FULL_TEXT</script>';
  const longMarkup = renderToStaticMarkup(TextPreview({text:long}));
  assert.match(longMarkup, /hljs-/);
  assert.ok(longMarkup.endsWith('&lt;script&gt;END_OF_FULL_TEXT&lt;/script&gt;</pre>'));
  const richCode = renderToStaticMarkup(ContentPreview({text:'function generated() { return true; }',html:'<pre>function generated() { return true; }</pre>'}));
  assert.match(richCode, /hljs-/);
  assert.doesNotMatch(richCode, /<iframe/);
  const richProse = renderToStaticMarkup(ContentPreview({text:'这是一段正常的中文文字。',html:'<b>这是一段正常的中文文字。</b>'}));
  assert.match(richProse, /<iframe/);
  assert.match(richProse, /sandbox=""/);
  const boundary='function generated() { return true; }\n'.padEnd(16_383,' ')+'😀END';
  const boundaryMarkup=renderToStaticMarkup(TextPreview({text:boundary}));
  assert.ok(boundaryMarkup.endsWith('😀END</pre>'));
  assert.equal(highlighted(long),highlighted(long.slice(0,16_384)+'different tail'));
  console.log('15 preview/highlighting cases passed');
} finally {
  await rm(root, { recursive: true, force: true });
}
