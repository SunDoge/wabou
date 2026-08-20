import { renderComponent } from "@wabou/test/component";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
  Text,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("breadcrumb exposes explicit links and the current page", () => {
  let destination = "";
  const screen = renderComponent(() => (
    <Breadcrumb>
      <BreadcrumbList>
        <BreadcrumbItem>
          <BreadcrumbLink
            aria-label="Projects"
            onClick={() => (destination = "projects")}
          >
            Projects
          </BreadcrumbLink>
        </BreadcrumbItem>
        <BreadcrumbSeparator />
        <BreadcrumbItem>
          <BreadcrumbPage>Wabou</BreadcrumbPage>
        </BreadcrumbItem>
      </BreadcrumbList>
    </Breadcrumb>
  ));

  screen.getByRole("link", { name: "Projects" }).click();
  expect(destination).toBe("projects");
  expect(
    screen.getByRole("link", { name: "Wabou" }).attribute("aria-current"),
  ).toBe("page");
});

test("pagination composes controlled page navigation", () => {
  const Pager = () => {
    const [page, setPage] = createSignal(2);
    return (
      <Pagination aria-label={`Page ${page()}`}>
        <PaginationContent>
          <PaginationItem>
            <PaginationPrevious
              aria-label="Previous page"
              onClick={() => setPage((value) => Math.max(1, value - 1))}
            />
          </PaginationItem>
          {[1, 2, 3].map((value) => (
            <PaginationItem>
              <PaginationLink
                aria-label={`Page ${value}`}
                active={page() === value}
                onClick={() => setPage(value)}
              >
                {String(value)}
              </PaginationLink>
            </PaginationItem>
          ))}
          <PaginationItem>
            <PaginationNext
              aria-label="Next page"
              onClick={() => setPage((value) => Math.min(3, value + 1))}
            />
          </PaginationItem>
        </PaginationContent>
        <Text role="status" aria-label={`Current page ${page()}`}>
          {String(page())}
        </Text>
      </Pagination>
    );
  };
  const screen = renderComponent(Pager);

  expect(
    screen.getByRole("link", { name: "Page 2" }).attribute("aria-current"),
  ).toBe("page");
  screen.getByRole("button", { name: "Next page" }).click();
  expect(screen.getByRole("status", { name: "Current page 3" }).text).toBe("3");
  expect(
    screen.getByRole("link", { name: "Page 3" }).attribute("aria-current"),
  ).toBe("page");
});
