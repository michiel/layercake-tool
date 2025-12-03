export type AttributeValue = string | number;
export type AttributesMap = Record<string, AttributeValue>;

const isInteger = (value: unknown): value is number =>
  typeof value === 'number' && Number.isInteger(value);

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
