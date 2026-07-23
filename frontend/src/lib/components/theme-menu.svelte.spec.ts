import { page } from 'vitest/browser';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { setMode, userPrefersMode } from 'mode-watcher';

import ThemeMenu from './theme-menu.svelte';

beforeEach(() => {
  setMode('system');
});

afterEach(() => {
  setMode('system');
});

describe('主题菜单', () => {
  it('支持暗色模式和跟随系统模式', async () => {
    render(ThemeMenu);

    await page.getByRole('button', { name: '切换主题' }).click();
    await page.getByText('Dark', { exact: true }).click();
    expect(userPrefersMode.current).toBe('dark');

    await page.getByRole('button', { name: '切换主题' }).click();
    await page.getByText('System', { exact: true }).click();
    expect(userPrefersMode.current).toBe('system');
  });
});
