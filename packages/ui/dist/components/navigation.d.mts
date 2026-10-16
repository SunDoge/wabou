import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { PaginationRangeItem, clampPage, createPaginationRange, normalizePageCount } from "./pagination-state.mjs";
import { JSX } from "solid-js";
//#region src/components/navigation.d.ts
export interface BreadcrumbProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}
export declare function Breadcrumb(props: BreadcrumbProps): JSX.Element;
export declare function BreadcrumbList(props: ViewProps): JSX.Element;
export declare function BreadcrumbItem(props: ViewProps): JSX.Element;
export interface BreadcrumbLinkProps extends Omit<ButtonProps, "class" | "role" | "variant" | "size"> {
  class?: string;
}
export declare function BreadcrumbLink(props: BreadcrumbLinkProps): JSX.Element;
export interface BreadcrumbPageProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
}
export declare function BreadcrumbPage(props: BreadcrumbPageProps): JSX.Element;
export interface BreadcrumbSeparatorProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}
export declare function BreadcrumbSeparator(props: BreadcrumbSeparatorProps): JSX.Element;
export interface BreadcrumbEllipsisProps extends Omit<ViewProps, "class" | "role" | "children"> {
  class?: string;
}
export declare function BreadcrumbEllipsis(props: BreadcrumbEllipsisProps): JSX.Element;
export interface PaginationProps {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
  count: number;
  page?: number;
  defaultPage?: number;
  disabled?: boolean;
  onPageChange?: (page: number) => void;
}
export declare function Pagination(props: PaginationProps): JSX.Element;
export declare function PaginationContent(props: ViewProps): JSX.Element;
export declare function PaginationItem(props: ViewProps): JSX.Element;
export interface PaginationLinkProps extends Omit<ButtonProps, "variant" | "size"> {
  active?: boolean;
  /** Selects this page when used inside a managed Pagination. */
  page?: number;
}
export declare function PaginationLink(props: PaginationLinkProps): JSX.Element;
export declare function PaginationEllipsis(props: {
  class?: string;
}): JSX.Element;
export declare function PaginationItems(props: {
  siblingCount?: number;
  boundaryCount?: number;
  renderItem?: (page: number) => JSX.Element;
  renderEllipsis?: (side: "start" | "end") => JSX.Element;
}): JSX.Element;
export declare function PaginationPrevious(props: Omit<ButtonProps, "variant" | "size">): JSX.Element;
export declare function PaginationNext(props: Omit<ButtonProps, "variant" | "size">): JSX.Element;
//#endregion
export { type PaginationRangeItem, clampPage, createPaginationRange, normalizePageCount };
//# sourceMappingURL=navigation.d.mts.map