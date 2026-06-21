// Re-export shared API types as legacy admin-record names.
// Prefer importing directly from api-types.ts for new code.

import type {
  UserDto,
  RoleResponse,
  Permission,
  SystemConfig,
  OAuthAppResponse,
} from "./api-types";

export type UserRecord = UserDto;
export type RoleRecord = RoleResponse;
export type PermissionRecord = Permission;
export type SettingRecord = SystemConfig;
export type OAuthAppRecord = OAuthAppResponse;
