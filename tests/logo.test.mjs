import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function readPngDimensions(buf) {
  assert.equal(buf.subarray(0, 8).toString('hex'), '89504e470d0a1a0a', 'valid PNG signature');
  return {
    width: buf.readUInt32BE(16),
    height: buf.readUInt32BE(20),
  };
}

function readIcoSizes(buf) {
  assert.equal(buf.readUInt16LE(0), 0, 'ICO reserved word is 0');
  assert.equal(buf.readUInt16LE(2), 1, 'ICO type is 1');
  const count = buf.readUInt16LE(4);
  const sizes = [];
  for (let i = 0; i < count; i++) {
    const offset = 6 + i * 16;
    const w = buf[offset] === 0 ? 256 : buf[offset];
    const h = buf[offset + 1] === 0 ? 256 : buf[offset + 1];
    sizes.push([w, h]);
  }
  return sizes;
}

test('official bordered Solara logo assets are present and valid', () => {
  const svgPath = path.join(root, 'public/logo.svg');
  const png192Path = path.join(root, 'public/logo-192.png');
  const png512Path = path.join(root, 'public/logo-512.png');

  const svg = fs.readFileSync(svgPath, 'utf8');
  assert.match(svg, /<rect[^>]*stroke="#1f4a66"/i, 'official bordered logo.svg keeps the rounded frame border');
  assert.match(svg, /<circle[^>]*stroke="url\(#solara-g\)"/i, 'official logo.svg keeps the gradient ring');
  assert.match(svg, /#3dd6f5/i);
  assert.match(svg, /#ffb55a/i);

  assert.deepEqual(readPngDimensions(fs.readFileSync(png192Path)), { width: 192, height: 192 });
  assert.deepEqual(readPngDimensions(fs.readFileSync(png512Path)), { width: 512, height: 512 });

  const websiteSvg = '/var/www/solara-inc-website/public/logo.svg';
  if (fs.existsSync(websiteSvg)) {
    assert.equal(
      svg.replace(/\r\n/g, '\n'),
      fs.readFileSync(websiteSvg, 'utf8').replace(/\r\n/g, '\n'),
      'public/logo.svg matches official website bordered logo',
    );
  }
});

test('desktop icon.png and multi-size icon.ico are generated from official bordered logo', () => {
  const iconPng = readPngDimensions(fs.readFileSync(path.join(root, 'apps/app/icons/icon.png')));
  assert.deepEqual(iconPng, { width: 512, height: 512 });

  const icoSizes = readIcoSizes(fs.readFileSync(path.join(root, 'apps/app/icons/icon.ico')));
  assert.ok(icoSizes.length >= 6, `expected multi-size ICO, got ${icoSizes.length} entries`);
  for (const required of [16, 24, 32, 48, 64, 128, 256]) {
    assert.ok(
      icoSizes.some(([w, h]) => w === required && h === required),
      `icon.ico must include ${required}x${required}`,
    );
  }
});

test('SolaraIcon in @modrinth/assets matches the official bordered logo.svg and is used in TitleBar and SplashScreen', () => {
  const solaraIconSvg = fs.readFileSync(path.join(root, 'packages/assets/solara/logo.svg'), 'utf8');
  const publicLogoSvg = fs.readFileSync(path.join(root, 'public/logo.svg'), 'utf8');
  assert.equal(solaraIconSvg.replace(/\r\n/g, '\n'), publicLogoSvg.replace(/\r\n/g, '\n'));

  const titleBar = fs.readFileSync(path.join(root, 'apps/app-frontend/src/app/shell/title-bar/index.vue'), 'utf8');
  const splash = fs.readFileSync(path.join(root, 'apps/app-frontend/src/components/ui/SplashScreen.vue'), 'utf8');
  assert.match(titleBar, /SolaraIcon/);
  assert.match(titleBar, /Solara Launcher/);
  assert.match(splash, /SolaraIcon/);
  assert.match(splash, /Solara Launcher/);
});
