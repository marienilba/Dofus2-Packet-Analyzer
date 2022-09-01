<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import type { DofusPacket } from "../utils/DofusPacket";
  import Display from "./Display.svelte";
  import Line from "./Line.svelte";

  let current: DofusPacket = null;
  let msgs: DofusPacket[] = [];

  const unlisten = listen<string>("rs2js", (event) => {
    let { data } = JSON.parse(event.payload) as {
      data: DofusPacket[];
    };

    msgs = [...data, ...msgs];
  });

  const handleSelect = (message: DofusPacket) => {
    current = message;
  };
  let count = 0;

  function getUniqueId() {
    count = count + 1;
    return count;
  }
</script>

<div
  class="flex flex-row min-w-screen h-screen bg-slate-700 p-2 gap-4 overflow-hidden"
>
  <div class="flex flex-grow flex-col gap-2 overflow-scroll no-scrollbar">
    {#each msgs as msg (`${msg.id}${msg.time}${getUniqueId()}`)}
      <Line message={msg} select={handleSelect} />
    {/each}
  </div>
  <div class="min-w-md min-h-screen w-[460px] bg-slate-900 overflow-scroll">
    <Display message={current} />
  </div>
</div>
