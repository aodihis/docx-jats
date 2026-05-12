/**
 * API contract — mirrors the Axum routes in backend/src/api/routes.rs.
 * Update both files together when adding or renaming endpoints.
 */

export const API_ROUTES = {
  health:  '/health',   // GET  → "ok"
  convert: '/convert',  // POST → multipart/form-data { file: File } → ConvertResponse
} as const;

export type ApiRoute = (typeof API_ROUTES)[keyof typeof API_ROUTES];
