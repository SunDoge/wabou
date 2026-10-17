import { createSignal } from "solid-js";

export function App() {
  const [count, setCount] = createSignal(0);
  return (
    <view
      class="flex flex-col w-full p-8 gap-5 text-primary"
      style={{ "font-family": "DejaVu Sans" }}
    >
      <view class="text-2xl font-bold">Rendered by Wabou</view>
      <view class="text-base text-secondary">
        Solid runs in your browser. Wabou lays out and paints this canvas.
      </view>
      <view class="text-5xl font-bold text-accent">{`Count: ${count()}`}</view>
      <view class="flex flex-wrap gap-3">
        <view
          role="button"
          class="bg-accent text-on-accent px-5 py-3 rounded-lg cursor-pointer"
          onClick={() => setCount(count() + 1)}
        >
          Increment
        </view>
        <view
          role="button"
          class="bg-control text-primary px-5 py-3 rounded-lg cursor-pointer"
          onClick={() => setCount(0)}
        >
          Reset
        </view>
      </view>
    </view>
  );
}
