<script lang="ts">
  import type { DofusPacket } from "src/utils/PacketDecoder";

  export let message: DofusPacket | null;

  let stringify = (v: any) => {
    // typeof v === "bigint" && console.log(v);
    return typeof v === "bigint" ? v.toString() : v;
  };
</script>

<div class="p-2">
  {#if message}
    <pre class="text-slate-100 whitespace-pre-wrap break-words"><code>
            {JSON.stringify(message.body, (_, v) => stringify(v), 2).slice(
          1,
          -1
        )}
        </code>
    </pre>
  {/if}
</div>
