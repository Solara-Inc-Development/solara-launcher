const { chromium } = require('playwright');
const { mkdir, writeFile } = require('node:fs/promises');
const assert = require('node:assert/strict');
(async () => {
  const base = process.env.SOLARA_UI_URL || 'http://127.0.0.1:1420';
  const browser = await chromium.launch({ headless: true, args: process.env.CI ? [] : ['--no-sandbox'] });
  try {
    await mkdir('checks', { recursive: true });
    const page = await browser.newPage({ viewport: { width: 1440, height: 950 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(`${base}/desktop.html`, { waitUntil: 'networkidle' });
    await page.getByRole('button', { name: 'Minecraft launcher', exact: true }).click();
    await page.getByRole('button', { name: 'Create your first instance' }).click();
    await page.getByRole('radio', { name: 'Fabric', exact: true }).click();
    await page.getByRole('combobox', { name: 'Minecraft version', exact: true }).click();
    await page.keyboard.press('Escape');
    assert.ok(await page.getByRole('dialog').isVisible(), 'Nested Escape closes dropdown before dialog');
    await page.keyboard.press('Escape');
    await page.getByRole('dialog').waitFor({state:'hidden'});
    for (const name of ['Solara services', 'Your servers', 'Explore mods and modpacks', 'Settings']) {
      await page.getByRole('button', { name, exact: true }).click();
    }
    await page.getByRole('button', { name: 'Soft iris', exact: true }).click();
    assert.equal(await page.locator('.shell').evaluate(e => e.style.getPropertyValue('--accent')), '#a6a3ff');
    await page.getByRole('button', { name: 'Solara cyan', exact: true }).click();
    await page.getByRole('checkbox', { name: 'Reduce motion', exact: true }).check();
    await page.getByRole('button', { name: 'Dashboard', exact: true }).click();
    assert.equal(await page.locator('.section-view').evaluate(e => getComputedStyle(e).animationName), 'none');
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.getByRole('checkbox', { name: 'Reduce motion', exact: true }).uncheck();
    await page.getByRole('button', { name: 'Downloads', exact: true }).click();
    await page.getByRole('heading', { name: 'Your downloads', exact: true }).waitFor();
    await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
    await page.setViewportSize({ width: 800, height: 620 });
    await page.getByRole('button', { name: 'Dashboard', exact: true }).click();
    assert.equal(await page.locator('.shell').evaluate(e => e.scrollWidth > innerWidth), false);
    const footer = await page.locator('.app-footer').boundingBox();
    assert.ok(footer && footer.y + footer.height <= 620);
    await page.screenshot({ path: 'checks/desktop-small.png', animations: 'disabled' });
    await page.setViewportSize({ width: 1440, height: 950 });
    await page.goto(base, { waitUntil: 'networkidle' });
    assert.equal(await page.locator('#seasons').count(), 0, 'Deferred feature remains hidden');
    await page.screenshot({ path: 'checks/website-desktop.png', fullPage: true, animations: 'disabled' });
    await page.setViewportSize({ width: 390, height: 844 });
    await page.screenshot({ path: 'checks/website-mobile.png', fullPage: true, animations: 'disabled' });
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    await page.getByRole('button', { name: 'Toggle navigation', exact: true }).click();
    await page.getByRole('link', { name: 'Features', exact: true }).click();
    assert.deepEqual(errors, []);
    await writeFile('checks/ui-results.json', JSON.stringify({ passed: true, desktopNavigation: true, customLoaderDropdown: true, nestedEscape: true, downloads: true, accent: true, reducedMotion: true, desktopSmall: true, mobileNavigation: true, seasonsDeferred: true, errors }, null, 2));
    console.log('Desktop/website interaction, accessibility and responsive checks passed.');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
