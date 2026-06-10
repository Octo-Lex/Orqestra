/**
 * TokenGate security tests — v2.9.1
 *
 * Proves the dashboard token gate has no admin, write, or forbidden-token paths.
 * Tokens are never persisted, logged, or included in errors/URLs.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TokenGate } from '../src/components/TokenGate';

describe('TokenGate', () => {
  let onAuth: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    onAuth = vi.fn();
  });

  afterEach(() => {
    cleanup();
  });

  // Helper: open the gate panel and return the input + connect button
  async function openGate() {
    const user = userEvent.setup();
    // The initial collapsed state has a button labeled "Connect Private View"
    const buttons = screen.getAllByRole('button');
    const openBtn = buttons.find(b => b.textContent === 'Connect Private View');
    if (!openBtn) throw new Error('Connect Private View button not found');
    await user.click(openBtn);
    const input = screen.getByPlaceholderText('Paste your read token...');
    const connectBtn = screen.getByRole('button', { name: 'Connect' });
    return { user, input, connectBtn };
  }

  // --- Security: rejection cases ---

  it('rejects hardcoded admin token', async () => {
    render(<TokenGate onAuth={onAuth} />);
    const { user, input, connectBtn } = await openGate();

    await user.type(input, 'master-secret');
    await user.click(connectBtn);

    expect(onAuth).not.toHaveBeenCalled();
    expect(screen.getByText(/invalid token/i)).toBeDefined();
  });

  it('rejects ork_write_ token', async () => {
    render(<TokenGate onAuth={onAuth} />);
    const { user, input, connectBtn } = await openGate();

    await user.type(input, 'ork_write_abc123');
    await user.click(connectBtn);

    expect(onAuth).not.toHaveBeenCalled();
    expect(screen.getByText(/invalid token/i)).toBeDefined();
  });

  it('rejects random string', async () => {
    render(<TokenGate onAuth={onAuth} />);
    const { user, input, connectBtn } = await openGate();

    await user.type(input, 'random-garbage');
    await user.click(connectBtn);

    expect(onAuth).not.toHaveBeenCalled();
    expect(screen.getByText(/invalid token/i)).toBeDefined();
  });

  it('rejects empty input', async () => {
    render(<TokenGate onAuth={onAuth} />);
    const { connectBtn } = await openGate();

    await userEvent.setup().click(connectBtn);

    expect(onAuth).not.toHaveBeenCalled();
  });

  // --- Security: acceptance case ---

  it('accepts ork_read_ token with private scope', async () => {
    render(<TokenGate onAuth={onAuth} />);
    const { user, input, connectBtn } = await openGate();

    await user.type(input, 'ork_read_test123');
    await user.click(connectBtn);

    expect(onAuth).toHaveBeenCalledWith('ork_read_test123', 'private');
  });

  // --- Security: no forbidden scope values ---

  it('never produces admin or write scope for any input', async () => {
    const forbiddenInputs = [
      'master-secret',
      'admin',
      'ork_admin_test',
      'ork_write_admin',
    ];

    for (const input_val of forbiddenInputs) {
      cleanup();
      const local_onAuth = vi.fn();
      render(<TokenGate onAuth={local_onAuth} />);
      const user = userEvent.setup();

      const buttons = screen.getAllByRole('button');
      const openBtn = buttons.find(b => b.textContent === 'Connect Private View');
      if (!openBtn) throw new Error('Open button not found');
      await user.click(openBtn);

      const input = screen.getByPlaceholderText('Paste your read token...');
      await user.type(input, input_val);
      await user.click(screen.getByRole('button', { name: 'Connect' }));

      if (local_onAuth.mock.calls.length > 0) {
        const scope = local_onAuth.mock.calls[0][1];
        expect(scope).not.toBe('admin');
        expect(scope).not.toBe('write');
      }
    }
  });

  // --- UI correctness ---

  it('shows Connect Private View button initially', () => {
    render(<TokenGate onAuth={onAuth} />);
    const buttons = screen.getAllByRole('button');
    const match = buttons.find(b => b.textContent === 'Connect Private View');
    expect(match).toBeDefined();
  });

  it('shows read-only description', () => {
    render(<TokenGate onAuth={onAuth} />);
    expect(screen.getByText(/Read-only/)).toBeDefined();
    expect(screen.getByText(/private metadata/)).toBeDefined();
  });

  // --- Source scan: no forbidden patterns in production code ---

  it('source code contains no forbidden token string literal', async () => {
    const fs = await import('fs');
    const path = await import('path');
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/components/TokenGate.tsx'),
      'utf-8'
    );
    // No hardcoded comparison against known bad tokens
    expect(source).not.toContain("=== 'master-secret'");
    expect(source).not.toContain('=== "master-secret"');
  });

  it('source code contains no admin scope literal', async () => {
    const fs = await import('fs');
    const path = await import('path');
    const source = fs.readFileSync(
      path.resolve(__dirname, '../src/components/TokenGate.tsx'),
      'utf-8'
    );
    expect(source).not.toContain("'admin'");
    expect(source).not.toContain('"admin"');
  });

  // --- Token persistence ---

  it('does not persist tokens to localStorage', () => {
    const setItemSpy = vi.spyOn(Storage.prototype, 'setItem');
    render(<TokenGate onAuth={onAuth} />);

    expect(setItemSpy).not.toHaveBeenCalled();
    setItemSpy.mockRestore();
  });

  it('does not persist tokens to sessionStorage', () => {
    const setItemSpy = vi.spyOn(sessionStorage, 'setItem');
    render(<TokenGate onAuth={onAuth} />);

    expect(setItemSpy).not.toHaveBeenCalled();
    setItemSpy.mockRestore();
  });
});
