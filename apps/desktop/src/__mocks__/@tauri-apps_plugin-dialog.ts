// Mock @tauri-apps/plugin-dialog for browser testing

export async function open(opts?: { directory?: boolean; multiple?: boolean }): Promise<string | null> {
  if (opts?.directory) {
    return "C:\\Next-Era\\Orqestra";
  }
  return null;
}

export async function save(_opts?: unknown): Promise<string | null> {
  return null;
}

export async function message(_opts?: unknown): Promise<void> {
  return;
}

export async function ask(_opts?: unknown): Promise<boolean> {
  return false;
}

export async function confirm(_opts?: unknown): Promise<boolean> {
  return false;
}
