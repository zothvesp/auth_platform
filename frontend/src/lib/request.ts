import Cookies from "js-cookie";

import { API_URL, authSessionCookieName, decodeSession } from "./auth-api";

type RequestOptions = {
  body?: unknown;
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  token?: string;
};

export class ApiError extends Error {
  code: string;
  status: number;

  constructor(message: string, status: number, code = "API_ERROR") {
    super(message);
    this.name = "ApiError";
    this.code = code;
    this.status = status;
  }
}

const getToken = (explicitToken?: string): string | undefined => {
  if (explicitToken) return explicitToken;
  return decodeSession(Cookies.get(authSessionCookieName))?.tokens.accessToken;
};

const isStateChanging = (method?: string): boolean =>
  method === "POST" || method === "PUT" || method === "PATCH" || method === "DELETE";

export const apiRequest = async <T>(
  path: string,
  options: RequestOptions = {},
): Promise<T> => {
  const token = getToken(options.token);
  const hasBody = options.body !== undefined;
  const method = options.method ?? "GET";
  const csrfToken = Cookies.get("csrf_token");

  const response = await fetch(`${API_URL}${path}`, {
    method,
    credentials: "include",
    headers: {
      ...(hasBody ? { "Content-Type": "application/json" } : {}),
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(isStateChanging(method) && csrfToken ? { "X-CSRF-Token": csrfToken } : {}),
    },
    body: hasBody ? JSON.stringify(options.body) : undefined,
  });

  if (!response.ok) {
    let message = "Request failed";
    let code = `HTTP_${response.status}`;

    try {
      const payload = await response.json();
      message = payload.message ?? payload.error ?? message;
      code = payload.code ?? code;
    } catch {
      message = response.statusText || message;
    }

    throw new ApiError(message, response.status, code);
  }

  if (response.status === 204) return undefined as T;
  return response.json() as Promise<T>;
};
