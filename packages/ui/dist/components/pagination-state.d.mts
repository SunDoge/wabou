//#region src/components/pagination-state.d.ts
export type PaginationRangeItem = number | "ellipsis-start" | "ellipsis-end";
export declare function normalizePageCount(count: number): number;
export declare function clampPage(page: number, count: number): number;
/**
 * Produces a stable, 1-indexed page range with explicit start/end ellipses.
 * A single hidden page is shown directly instead of being replaced by an
 * ellipsis, which keeps every item actionable and avoids misleading gaps.
 */
export declare function createPaginationRange(options: {
  count: number;
  page: number;
  siblingCount?: number;
  boundaryCount?: number;
}): PaginationRangeItem[];
//#endregion
//# sourceMappingURL=pagination-state.d.mts.map