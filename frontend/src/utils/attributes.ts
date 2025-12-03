export type AttributeValue = string | number;
export type AttributesMap = Record<string, AttributeValue>;

const isInteger = (value: unknown): value is number =>
  typeof value === 'number' && Number.isInteger(value);

const VALID_TOKEN = /^[A-Za-z0-9_-]+$/;

export const sanitizeAttributes = (value?: any): AttributesMap => {
  if (!value || typeof value !== 'object') return {};
  const entries: [string, AttributeValue][] = [];
  Object.entries(value as Record<string, any>).forEach(([key, val]) => {
    if (!key || typeof key !== 'string') return;
    if (typeof val === 'string') {
      entries.push([key, val]);
    } else if (isInteger(val)) {
      entries.push([key, val]);
    }
  });
  return Object.fromEntries(entries);
};

export const attributesToInlineString = (
  attrs?: AttributesMap,
  separator = '; '
): string => {
  const entries = Object.entries(attrs || {});
  if (!entries.length) return '';
  return entries
    .map(([key, value]) => `${key}:${value}`)
    .join(separator)
    .trim();
};

export const attributesToMultiline = (attrs?: AttributesMap): string =>
  attributesToInlineString(attrs, '\n');

export const attributesToJson = (attrs?: AttributesMap): string => {
  const clean = sanitizeAttributes(attrs);
  if (Object.keys(clean).length === 0) return '';
  return JSON.stringify(clean);
};

export const parseAttributesJson = (raw?: string | null): AttributesMap => {
  if (!raw) return {};
  const trimmed = raw.trim();
  if (!trimmed) return {};
  try {
    const parsed = JSON.parse(trimmed);
    return sanitizeAttributes(parsed);
  } catch {
    return {};
  }
};

export type ParsedAttributesInline =
  | { ok: true; value: AttributesMap }
  | { ok: false; error: string };

export const parseAttributesInline = (raw: string): ParsedAttributesInline => {
  const text = raw.trim();
  if (!text) return { ok: true, value: {} };

  const parts = text.split(';').map(p => p.trim()).filter(Boolean);
  const map: AttributesMap = {};

  for (const part of parts) {
    const [key, ...rest] = part.split(':');
    const valueRaw = rest.join(':'); // allow colons in value only via join (not encouraged)
    const keyTrimmed = key.trim();
    if (!keyTrimmed) {
      return { ok: false, error: 'Attribute keys cannot be empty' };
    }
    if (!VALID_TOKEN.test(keyTrimmed)) {
      return { ok: false, error: `Invalid key "${keyTrimmed}" (use letters, numbers, _, -)` };
    }

    const valueTrimmed = valueRaw.trim();
    if (valueTrimmed.length > 0 && !VALID_TOKEN.test(valueTrimmed)) {
      return { ok: false, error: `Invalid value "${valueTrimmed}" (use letters, numbers, _, -)` };
    }

    if (valueTrimmed === '') {
      map[keyTrimmed] = '';
    } else if (/^-?\d+$/.test(valueTrimmed)) {
      map[keyTrimmed] = parseInt(valueTrimmed, 10);
    } else {
      map[keyTrimmed] = valueTrimmed;
    }
  }

  return { ok: true, value: map };
};
