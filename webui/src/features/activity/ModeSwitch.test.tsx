import {expect, it, vi} from 'vitest';
import {renderToStaticMarkup} from 'react-dom/server';
import {ModeApply, ModeCards} from './ModeSwitch';
import {LangContext} from '../../i18n';

it.each([
  {writable: true, dirty: true, incomplete: false, busy: false, disabled: false},
  {writable: true, dirty: false, incomplete: false, busy: false, disabled: true},
  {writable: true, dirty: true, incomplete: true, busy: false, disabled: true},
  {writable: true, dirty: true, incomplete: false, busy: true, disabled: true},
  {writable: false, dirty: true, incomplete: false, busy: false, disabled: null}
])('standalone Global Apply respects $writable, $dirty, $incomplete and $busy', state => {
  const noop = () => {};
  const markup = renderToStaticMarkup(
    <LangContext.Provider value="en">
      <ModeCards
        part="global"
        model={{
          ...state,
          mode: 'global',
          target: 'gaming',
          targetText: 'gaming',
          status: 'Read-only',
          readOnly: !state.writable,
          modes: [],
          targets: [{id: 'gaming', label: 'gaming'}],
          reasons: {mode: null, global: null},
          error: null,
          retry: noop,
          pick: noop,
          pickTarget: noop,
          apply: noop
        }}
      />
    </LangContext.Provider>
  );
  const apply = [...markup.matchAll(/<button\b[^>]*>.*?<\/button>/gs)].map(match => match[0]).find(button => button.includes('Apply'));
  if (state.disabled === null) expect(apply).toBeUndefined();
  else {
    expect(apply).toBeDefined();
    expect(/\bdisabled=""|aria-disabled="true"/.test(apply!)).toBe(state.disabled);
  }
});

it('keeps Mode card Apply and Global card Apply independently enabled by modeDirty and targetDirty', () => {
  const noop = () => {};
  const base = {
    mode: 'global',
    target: 'gaming',
    targetText: 'gaming',
    writable: true,
    dirty: true,
    incomplete: false,
    targetIncomplete: false,
    busy: false,
    status: 'Read-only',
    readOnly: false,
    modes: [
      ['rule', 'Rule'],
      ['direct', 'Direct'],
      ['global', 'Global']
    ] as Array<[string, string]>,
    targets: [{id: 'gaming', label: 'gaming'}],
    reasons: {mode: null, global: null},
    error: null,
    retry: noop,
    pick: noop,
    pickTarget: noop,
    apply: noop,
    applyMode: noop,
    applyTarget: noop
  };
  const applyButton = (part: 'mode' | 'global', modeDirty: boolean, targetDirty: boolean) => {
    const markup = renderToStaticMarkup(
      <LangContext.Provider value="en">
        <ModeCards part={part} model={{...base, modeDirty, targetDirty}} />
      </LangContext.Provider>
    );
    return [...markup.matchAll(/<button\b[^>]*>.*?<\/button>/gs)].map(match => match[0]).find(button => button.includes('Apply'))!;
  };
  // Only targetDirty: Global Apply is enabled, Mode Apply stays disabled.
  expect(/\bdisabled=""|aria-disabled="true"/.test(applyButton('global', false, true))).toBe(false);
  expect(/\bdisabled=""|aria-disabled="true"/.test(applyButton('mode', false, true))).toBe(true);
  // Only modeDirty: Mode Apply is enabled, Global Apply stays disabled.
  expect(/\bdisabled=""|aria-disabled="true"/.test(applyButton('mode', true, false))).toBe(false);
  expect(/\bdisabled=""|aria-disabled="true"/.test(applyButton('global', true, false))).toBe(true);
});

it('respects targetWritable independently of config mode writable', () => {
  const noop = () => {};
  const markup = renderToStaticMarkup(
    <LangContext.Provider value="en">
      <ModeCards
        part="global"
        model={{
          mode: 'rule',
          target: 'auto',
          targetText: 'auto',
          writable: false,
          targetWritable: true,
          dirty: true,
          modeDirty: false,
          targetDirty: true,
          incomplete: false,
          targetIncomplete: false,
          busy: false,
          status: 'Read-only',
          readOnly: true,
          modes: [],
          targets: [{id: 'auto', label: 'auto'}],
          reasons: {mode: null, global: null},
          error: null,
          retry: noop,
          pick: noop,
          pickTarget: noop,
          apply: noop
        }}
      />
    </LangContext.Provider>
  );
  const apply = [...markup.matchAll(/<button\b[^>]*>.*?<\/button>/gs)].map(match => match[0]).find(button => button.includes('Apply'));
  expect(apply).toBeDefined();
  expect(/\bdisabled=""|aria-disabled="true"/.test(apply!)).toBe(false);
});

import type {ReactElement} from 'react';
import {hookHarness} from '../../store/testHelpers';
const control = vi.hoisted(() => ({testing: false}));
vi.mock('react', async original => {
  const actual = await original<typeof import('react')>();
  return {
    ...actual,
    useState: (initial: unknown) => (control.testing ? hookHarness.hooks.useState(initial) : actual.useState(initial))
  };
});
vi.mock('../../i18n', async original => {
  const actual = await original<typeof import('../../i18n')>();
  return {
    ...actual,
    useT: () => (key: Parameters<typeof actual.translate>[1], params: Parameters<typeof actual.translate>[2]) => actual.translate('en', key, params)
  };
});
it('applies global-scoped node policy selection immediately without opening a Global mode confirm dialog', () => {
  control.testing = true;
  hookHarness.reset();
  const applyTarget = vi.fn();
  const model = {
    mode: 'rule',
    modes: [['rule', 'Rule']] as Array<[string, string]>,
    writable: true,
    dirty: true,
    incomplete: false,
    busy: false,
    apply: vi.fn(),
    applyTarget
  };
  const read = () =>
    hookHarness.render(() => ModeApply({model, scope: 'global'})) as ReactElement<{
      children: [ReactElement<{onPress: () => void}>, ReactElement<{isOpen: boolean}>];
    }>;
  try {
    read().props.children[0].props.onPress();
    expect(applyTarget).toHaveBeenCalledOnce();
    expect(read().props.children[1].props.isOpen).toBe(false);
  } finally {
    control.testing = false;
  }
});

it.each(['direct', 'global', 'rule'])('confirms %s mode before applying, with cancellation preserving the draft', mode => {
  control.testing = true;
  hookHarness.reset();
  const apply = vi.fn();
  const model = {mode, modes: [[mode, mode]] as Array<[string, string]>, writable: true, dirty: true, incomplete: false, busy: false, apply};
  const read = () =>
    hookHarness.render(() => ModeApply({model})) as ReactElement<{
      children: [ReactElement<{onPress: () => void}>, ReactElement<{isOpen: boolean; title: string; onCancel: () => void; onConfirm: () => void}>];
    }>;
  try {
    read().props.children[0].props.onPress();
    if (mode === 'rule') expect(apply).toHaveBeenCalledOnce();
    else {
      expect(apply).not.toHaveBeenCalled();
      expect(read().props.children[1].props).toMatchObject({isOpen: true, title: `Apply ${mode} mode?`});
      read().props.children[1].props.onCancel();
      expect(apply).not.toHaveBeenCalled();
      expect(read().props.children[1].props.isOpen).toBe(false);
      read().props.children[0].props.onPress();
      read().props.children[1].props.onConfirm();
      expect(apply).toHaveBeenCalledOnce();
    }
  } finally {
    control.testing = false;
  }
});
