<script lang="ts">
  import type { DofusPacket } from "../utils/PacketDecoder";
  export let message: DofusPacket;
  export let select: (m: DofusPacket) => void;
</script>

<div
  class="flex flex-row gap-2 bg-slate-700 text-slate-100 border-solid border border-slate-800 p-2 hover:bg-slate-800 cursor-pointer rounded"
  on:click={() => select(message)}
>
  <p class="min-w-xs text-yellow-400 font-bold">{message.time}</p>
  <p>{message.name}</p>
</div>
