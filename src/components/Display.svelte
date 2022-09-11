<script lang="ts">
  import type { DofusPacket } from "../utils/DofusPacket";

  export let message: DofusPacket | null;
</script>

<div class="p-2 text-slate-100">
  {#if message}
    <div class="flex flex-row gap-2 items-center">
      <p class="text-xl font-bold">{message.source}</p>
      <p>|</p>
      <p class="text-sm font-bold">{message.name}</p>
    </div>
    <pre class=" whitespace-pre-wrap break-words"><code>
            {JSON.stringify(message.body, null, 2).slice(1, -1)}
        </code>
    </pre>
  {/if}
</div>
