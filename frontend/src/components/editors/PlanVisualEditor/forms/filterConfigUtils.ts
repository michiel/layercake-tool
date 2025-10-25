import {
  QueryFilterConfig,
} from '../../../../types/plan-dag';
import {
  createDefaultQueryFilterConfig,
  normalizeQueryFilterConfig,
} from './QueryFilterBuilder';

type LegacyGraphFilter = {
  kind?: string;
  params?: {
    queryConfig?: QueryFilterConfig;
  };
};

const isLegacyQueryFilter = (kind?: string): boolean => {
  if (!kind) {
    return false;
  }
  const normalized = kind.toLowerCase();
  return normalized === 'query' || normalized === 'querytext';
};

export const extractQueryConfigFromRaw = (raw: unknown): QueryFilterConfig => {
  if (raw && typeof raw === 'object') {
    const record = raw as Record<string, unknown>;
    if (record.query) {
      return normalizeQueryFilterConfig(record.query as QueryFilterConfig);
    }

    const filters = record.filters;
    if (Array.isArray(filters)) {
      for (const candidate of filters as LegacyGraphFilter[]) {
        if (!candidate) {
          continue;
        }
        if (!isLegacyQueryFilter(candidate.kind)) {
          continue;
        }
        if (candidate.params?.queryConfig) {
          return normalizeQueryFilterConfig(candidate.params.queryConfig);
        }
      }
    }
  }

  return createDefaultQueryFilterConfig();
};
