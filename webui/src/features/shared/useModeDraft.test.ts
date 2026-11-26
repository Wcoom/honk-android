import {afterEach, expect, it, vi} from 'vitest';
import {readSavedGlobalTarget, saveGlobalTarget} from './useModeDraft';
import {storageKeys} from '../../api/storage';

afterEach(() => {
  vi.unstubAllGlobals();
});

it('saves and reads the preferred global outbound independently and tolerates storage errors', () => {
  const store = new Map<string, string>();
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
    removeItem: (key: string) => void store.delete(key)
  });

  expect(readSavedGlobalTarget()).toBe('');
  saveGlobalTarget('proxy');
  expect(readSavedGlobalTarget()).toBe('proxy');
  expect(store.get(storageKeys.globalOutbound)).toContain('proxy');

  saveGlobalTarget('auto');
  expect(readSavedGlobalTarget()).toBe('auto');

  saveGlobalTarget('');
  expect(readSavedGlobalTarget()).toBe('');

  const raise = () => {
    throw new Error('storage blocked');
  };
  vi.stubGlobal('localStorage', {getItem: raise, setItem: raise, removeItem: raise});
  expect(readSavedGlobalTarget()).toBe('');
  expect(() => saveGlobalTarget('proxy')).not.toThrow();
});
