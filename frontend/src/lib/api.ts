import type { ConvertResponse } from './types';
import { API_ROUTES } from './api-contract';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3001';

export async function convertDocx(file: File): Promise<ConvertResponse> {
  const form = new FormData();
  form.append('file', file);

  const res = await fetch(`${API_BASE}${API_ROUTES.convert}`, {
    method: 'POST',
    body: form
  });

  const data = await res.json();

  if (!res.ok || !data.success) {
    throw new Error(data.error ?? `Server error: ${res.status}`);
  }

  return data as ConvertResponse;
}
