"use client";

import type {
  BaseRecord,
  CreateParams,
  DataProvider,
  DeleteOneParams,
  GetListParams,
  GetOneParams,
  UpdateParams,
} from "@refinedev/core";
import Cookies from "js-cookie";
import { API_URL, authSessionCookieName, decodeSession } from "@lib/auth-api";

type BackendResource = "audit_logs" | "permissions" | "roles" | "settings" | "users";

type PaginatedPayload<T> = {
  data: T[];
  page?: number;
  page_size?: number;
  total: number;
  total_pages?: number;
};

const resourcePath: Record<BackendResource, string> = {
  audit_logs: "/admin/audit-logs",
  permissions: "/permissions",
  roles: "/roles",
  settings: "/config",
  users: "/admin/users",
};

const isBackendResource = (resource: string): resource is BackendResource =>
  resource in resourcePath;

const getToken = () => decodeSession(Cookies.get(authSessionCookieName))?.tokens.accessToken;

const toQueryString = (params: Record<string, string | number | undefined>) => {
  const search = new URLSearchParams();
  Object.entries(params).forEach(([key, value]) => {
    if (value !== undefined && value !== "") search.set(key, String(value));
  });
  const query = search.toString();
  return query ? `?${query}` : "";
};

const getFilterValue = (params: GetListParams, field: string) => {
  const filter = params.filters?.find((item) => "field" in item && item.field === field);
  return filter && "value" in filter ? filter.value : undefined;
};

const request = async <T>(path: string, init: RequestInit = {}) => {
  const token = getToken();
  const response = await fetch(`${API_URL}${path}`, {
    ...init,
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...init.headers,
    },
  });

  if (!response.ok) {
    let message = response.statusText || "Request failed";
    try {
      const payload = await response.json();
      message = payload.message ?? payload.error ?? message;
    } catch {
      // Keep status text.
    }
    throw {
      message,
      statusCode: response.status,
    };
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
};

const listPath = (resource: BackendResource, params: GetListParams) => {
  const pagination = params.pagination;
  const currentPage = pagination?.currentPage ?? 1;
  const pageSize = pagination?.pageSize ?? (resource === "audit_logs" ? 50 : 20);

  if (resource === "users") {
    return `${resourcePath.users}${toQueryString({
      page: currentPage,
      page_size: pageSize,
      status: getFilterValue(params, "status"),
    })}`;
  }

  if (resource === "audit_logs") {
    return `${resourcePath.audit_logs}${toQueryString({
      page: currentPage,
      page_size: pageSize,
      user_id: getFilterValue(params, "user_id"),
    })}`;
  }

  return resourcePath[resource];
};

export const dataProvider: DataProvider = {
  getApiUrl: () => API_URL,

  getList: async <TData extends BaseRecord = BaseRecord>(params: GetListParams) => {
    if (!isBackendResource(params.resource)) {
      return { data: [], total: 0 };
    }

    const payload = await request<TData[] | PaginatedPayload<TData>>(
      listPath(params.resource, params),
    );

    if (Array.isArray(payload)) {
      if (params.resource === "settings") {
        const rows = payload.map((row) => ({
          id: String((row as BaseRecord).key),
          ...row,
        })) as TData[];
        return {
          data: rows,
          total: rows.length,
        };
      }

      return {
        data: payload,
        total: payload.length,
      };
    }

    return {
      data: payload.data,
      total: payload.total,
    };
  },

  getOne: async <TData extends BaseRecord = BaseRecord>({ resource, id }: GetOneParams) => {
    if (!isBackendResource(resource)) {
      throw { message: `Unknown resource: ${resource}`, statusCode: 404 };
    }

    if (resource === "settings") {
      const rows = await request<TData[]>(resourcePath.settings);
      const data = rows.find((row) => String(row.key ?? row.id) === String(id));
      if (!data) throw { message: "Setting not found", statusCode: 404 };
      return { data };
    }

    return {
      data: await request<TData>(`${resourcePath[resource]}/${id}`),
    };
  },

  create: async <TData extends BaseRecord = BaseRecord, TVariables = {}>({
    resource,
    variables,
  }: CreateParams<TVariables>) => {
    if (!isBackendResource(resource)) {
      throw { message: `Unknown resource: ${resource}`, statusCode: 404 };
    }

    return {
      data: await request<TData>(resourcePath[resource], {
        method: "POST",
        body: JSON.stringify(variables),
      }),
    };
  },

  update: async <TData extends BaseRecord = BaseRecord, TVariables = {}>({
    resource,
    id,
    variables,
  }: UpdateParams<TVariables>) => {
    if (!isBackendResource(resource)) {
      throw { message: `Unknown resource: ${resource}`, statusCode: 404 };
    }

    const path = resource === "settings" ? `/config/${id}` : `${resourcePath[resource]}/${id}`;

    return {
      data: await request<TData>(path, {
        method: "PUT",
        body: JSON.stringify(variables),
      }),
    };
  },

  deleteOne: async <TData extends BaseRecord = BaseRecord, TVariables = {}>({
    resource,
    id,
  }: DeleteOneParams<TVariables>) => {
    if (!isBackendResource(resource)) {
      throw { message: `Unknown resource: ${resource}`, statusCode: 404 };
    }

    await request<void>(`${resourcePath[resource]}/${id}`, { method: "DELETE" });
    return { data: { id } as TData };
  },
};
