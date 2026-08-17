import { mount } from "@wabou/core";
import { MemoryRouter, Route } from "@wabou/router";
import "virtual:wabou-stylesheet";
import { App } from "./app";

mount(() => (
  <MemoryRouter initialEntries={["/counter"]}>
    <Route path={["/", "/:task"]} component={App} />
  </MemoryRouter>
));
