"use client"
import init, { greet } from "@/wasm/gbemu_core";

export default function Page() {
  return (
    <main>
      <button onClick={() => init().then(() => greet("wasm"))}>Click here!</button>
    </main>
  );
}
