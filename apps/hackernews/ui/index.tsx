// Hacker News UI entry.
import "@wabou/core";
import "virtual:wabou-stylesheet";
import { mount } from "@wabou/core";
import { MemoryRouter, Route } from "@wabou/router";
import { AppShell } from "./AppShell";
import { ThemeProvider } from "./contexts/ThemeContext";
import { StoryDetail } from "./pages/StoryDetail";
import { StoryList } from "./pages/StoryList";

mount(() => (
  <ThemeProvider>
    <MemoryRouter root={AppShell}>
      <Route path="/" component={StoryList} />
      <Route path="/story/:id" component={StoryDetail} />
    </MemoryRouter>
  </ThemeProvider>
));
