#!/usr/bin/env node

import fs from 'node:fs/promises';
import path from 'node:path';
import {JSDOM} from 'jsdom';

const ROOT = path.resolve('docs');
const MERMAID_FENCE = /```mermaid\s*\n([\s\S]*?)```/g;

async function walk(dir) {
  const entries = await fs.readdir(dir, {withFileTypes: true});
  const files = [];

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await walk(fullPath)));
      continue;
    }

    if (entry.isFile() && (entry.name.endsWith('.md') || entry.name.endsWith('.mdx'))) {
      files.push(fullPath);
    }
  }

  return files;
}

function installDomGlobals() {
  const dom = new JSDOM('<!doctype html><html><body></body></html>', {
    url: 'http://localhost/',
  });

  const {window} = dom;
  globalThis.window = window;
  globalThis.document = window.document;
  Object.defineProperty(globalThis, 'navigator', {
    value: window.navigator,
    configurable: true,
  });
  globalThis.Element = window.Element;
  globalThis.HTMLElement = window.HTMLElement;
  globalThis.SVGElement = window.SVGElement;
  globalThis.Node = window.Node;
  globalThis.getComputedStyle = window.getComputedStyle;

  return dom;
}

async function main() {
  const dom = installDomGlobals();
  const mermaid = (await import('mermaid')).default;

  mermaid.initialize({startOnLoad: false});

  const files = await walk(ROOT);
  const failures = [];
  let checked = 0;

  for (const file of files) {
    const content = await fs.readFile(file, 'utf8');
    const matches = [...content.matchAll(MERMAID_FENCE)];

    for (let i = 0; i < matches.length; i += 1) {
      const diagram = matches[i][1].trim();
      checked += 1;

      try {
        await mermaid.parse(diagram, {suppressErrors: false});
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        failures.push({
          file: path.relative(process.cwd(), file),
          block: i + 1,
          message,
        });
      }
    }
  }

  dom.window.close();

  if (failures.length > 0) {
    console.error(`Mermaid validation failed in ${failures.length} block(s):`);
    for (const failure of failures) {
      console.error(`- ${failure.file} (block ${failure.block})`);
      console.error(`  ${failure.message}`);
    }
    process.exit(1);
  }

  console.log(`Mermaid validation passed (${checked} block(s) checked).`);
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  console.error(message);
  process.exit(1);
});
