<script lang="ts">
  import { onMount } from "svelte";

  import type { DofusPacket } from "../utils/DofusPacket";
  export let message: DofusPacket;
  export let select: (m: DofusPacket) => void;
  // Urgh too lazy to work with Date, chrono in rust return always the same value sometime (dunno why...)
  let time_format = "";
  onMount(() => {
    time_format = formatTime(new Date());
  });

  function formatTime(time: Date) {
    let hours = time.getHours();
    let minutes = time.getMinutes();
    let seconds = time.getSeconds();

    return [
      hours < 10 ? `0${hours}` : hours,
      minutes < 10 ? `0${minutes}` : minutes,
      seconds < 10 ? `0${seconds}` : seconds,
    ].join(":");
  }
</script>

<div
  class={`flex flex-row gap-2 ${
    message.source === "Server" ? "bg-slate-700" : "bg-slate-500"
  }  text-slate-100 border-solid border border-slate-800 p-2 hover:bg-slate-800 cursor-pointer rounded`}
  on:click={() => select(message)}
>
  <p class="min-w-xs text-yellow-400 font-bold">{time_format}</p>
  <p>{message.name}</p>
</div>
