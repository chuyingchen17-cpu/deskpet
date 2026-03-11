import { useMemo } from 'react';

export function useSessionId(): string {
  return useMemo(() => {
    const key = 'desktop_pet_session_id';
    const existing = localStorage.getItem(key);
    if (existing) {
      return existing;
    }
    const generated = crypto.randomUUID();
    localStorage.setItem(key, generated);
    return generated;
  }, []);
}
