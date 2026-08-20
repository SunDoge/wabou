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
  PaginationItems,
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

test("managed pagination owns range generation and boundary controls", () => {
  const changes: number[] = [];
  const screen = renderComponent(() => (
    <Pagination
      count={12}
      defaultPage={6}
      aria-label="Results pages"
      onPageChange={(page) => changes.push(page)}
    >
      <PaginationContent>
        <PaginationPrevious aria-label="Previous results page" />
        <PaginationItems />
        <PaginationNext aria-label="Next results page" />
      </PaginationContent>
    </Pagination>
  ));

  expect(
    screen.getByRole("link", { name: "Page 6" }).attribute("aria-current"),
  ).toBe("page");
  expect(screen.queryByRole("link", { name: "Page 2" })).toBeNull();

  screen.getByRole("button", { name: "Next results page" }).click();
  expect(changes).toEqual([7]);
  expect(
    screen.getByRole("link", { name: "Page 7" }).attribute("aria-current"),
  ).toBe("page");

  screen.getByRole("link", { name: "Page 12" }).click();
  expect(changes).toEqual([7, 12]);
  expect(
    screen
      .getByRole("button", { name: "Next results page" })
      .attribute("disabled"),
  ).toBe("true");
});

test("managed pagination remains controlled when its owner does not update", () => {
  const changes: number[] = [];
  const screen = renderComponent(() => (
    <Pagination count={4} page={2} onPageChange={(page) => changes.push(page)}>
      <PaginationNext aria-label="Advance" />
      <PaginationItems />
    </Pagination>
  ));

  screen.getByRole("button", { name: "Advance" }).click();
  expect(changes).toEqual([3]);
  expect(
    screen.getByRole("link", { name: "Page 2" }).attribute("aria-current"),
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
