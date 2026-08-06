// Deterministic UI fixtures.
import type { Feed, Story } from "./stories";

const titles = [
  "A layout engine should keep measurement and painting on the same shaped text",
  "Building a native UI runtime with Rust, Solid, Parley, Taffy, and Vello",
  "Why inline formatting contexts are more than flex rows with wrapping",
  "Small deterministic test applications make rendering regressions reproducible",
  "SVG replaced elements, intrinsic ratios, baselines, and content boxes",
  "中文排版测试：长标题需要在有限宽度内正确换行",
  "Text input editing, selection, caret geometry, and IME composition",
  "Retained layout and paint caches without stale frames or visual ghosts",
] as const;

const authors = ["alice", "bob", "carol", "dave", "eve"] as const;

export const fixtureStories: Record<Feed, Story[]> = {
  top: makeFeed(1000, 0),
  new: makeFeed(2000, 3),
  best: makeFeed(3000, 5),
};

function makeFeed(idBase: number, titleOffset: number): Story[] {
  const now = Math.floor(Date.now() / 1000);
  return Array.from({ length: 30 }, (_, index) => ({
    id: idBase + index,
    title: titles[(index + titleOffset) % titles.length],
    // Deliberately empty: the rendering test app must never initiate network
    // navigation, even when a story row or detail action is activated.
    url: "",
    by: authors[index % authors.length],
    score: 420 - index * 7,
    descendants: (index * 13) % 97,
    time: now - (index + 1) * 11 * 60,
  }));
}
