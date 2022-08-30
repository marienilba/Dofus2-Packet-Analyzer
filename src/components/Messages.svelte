<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import PacketDecoder, { type DofusPacket } from "../utils/PacketDecoder";
  import Display from "./Display.svelte";
  import Line from "./Line.svelte";

  let current: DofusPacket = null;
  let msgs: DofusPacket[] = [];
  let pd = new PacketDecoder();

  const unlisten = listen<string>("rs2js", (event) => {
    let new_message = JSON.parse(event.payload);
    pd.decodeTcp(new_message.remaining, 5555);
    msgs = [...pd.getQueue(), ...msgs];
  });

  const handleSelect = (m: DofusPacket) => {
    current = m;
  };
</script>

<div
  class="flex flex-row min-w-screen h-screen bg-slate-700 p-2 gap-4 overflow-hidden"
>
  <div class="flex flex-grow flex-col gap-2 overflow-scroll no-scrollbar">
    {#each msgs as msg}
      <Line message={msg} select={handleSelect} />
    {/each}
  </div>
  <div class="min-w-md min-h-screen w-[460px] bg-slate-900 overflow-scroll">
    <Display message={current} />
  </div>
</div>
