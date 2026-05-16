import type { ConvertResponse, DocumentContent, RegenerateResponse } from './types';
import { API_ROUTES } from './api-contract';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:5505';

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

export async function regenerateXml(document: DocumentContent): Promise<RegenerateResponse> {
  const res = await fetch(`${API_BASE}${API_ROUTES.regenerate}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ document })
  });

  const data = await res.json();

  if (!res.ok || !data.success) {
    throw new Error(data.error ?? `Server error: ${res.status}`);
  }

  return data as RegenerateResponse;
}
