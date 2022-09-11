<script lang="ts">
  import { tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import type { DofusPacket } from "../utils/DofusPacket";
  import Display from "./Display.svelte";
  import Line from "./Line.svelte";

  let current: DofusPacket = null;
  let msgs: DofusPacket[] = [];
  let client = true;
  let server = true;
  let playing = true;

  const handleClient = async () => {
    client = !client;
    msgs = msgs;
  };

  const handleServer = async () => {
    server = !server;
    msgs = msgs;
  };

  const handlePlay = () => (playing = !playing);

  const filterMessages = (msgs: DofusPacket[]): DofusPacket[] => {
    return msgs.filter((m) => {
      if (!client && m.source === "Client") return false;
      if (!server && m.source === "Server") return false;
      return true;
    });
  };

  let count = 0;
  function getUniqueId() {
    count = count + 1;
    return count;
  }

  const unlisten = listen<string>("rs2js", (event) => {
    if (!playing) return;

    let { data } = JSON.parse(event.payload) as {
      data: DofusPacket[];
    };

    for (const msg of data) {
      msg.id = `${msg.id}${msg.time}${getUniqueId()}`;
      msgs = [msg, ...msgs];
    }
  });

  const handleSelect = (message: DofusPacket) => {
    current = message;
  };
</script>

<div class="flex flex-col min-w-screen h-screen p-2 gap-4">
  <div class="flex flex-row gap-2 items-center p-4">
    <input type="checkbox" checked={client} on:click={handleClient} />
    <label class="text-xl text-slate-100" for="Client">Client</label><br />
    <input type="checkbox" checked={server} on:click={handleServer} />
    <label class="text-xl text-slate-100" for="Server">Server</label><br />
    <button
      on:click={handlePlay}
      class="px-2 py-1 bg-slate-400 rounded-full hover:opacity-75"
      >{playing ? "🔴" : "🟦"}</button
    >
  </div>
  <div class="flex flex-row p-2 gap-4 overflow-hidden h-full">
    <div
      class="flex flex-grow flex-col gap-2 overflow-scroll no-scrollbar h-full"
    >
      {#each filterMessages(msgs) as msg (msg.id)}
        <Line message={msg} select={handleSelect} />
      {/each}
    </div>
    <div class="min-w-md h-full  w-[460px] bg-slate-900 overflow-scroll">
      <Display message={current} />
    </div>
  </div>
</div>
