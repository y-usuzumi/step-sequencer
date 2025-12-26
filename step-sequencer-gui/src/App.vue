<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import TrackerView from './TrackerView/TrackerView.vue'
import MainControl from "./MainPanel/MainControl.vue";

type Sign = "Plus" | "Minus";

interface Rational {
  Rational: [Sign, [number, number]] // [Sign, [numer, denom]]
}

interface Infinity {
  Infinity: Sign,
}

type NaN = "NaN";

interface BeatTime {
  frac: Rational | Infinity | NaN,
}

interface Beat {
  Beat: BeatTime
}

type Pause = "Pause";
type Stop = "Stop";

type BeatSignal = Beat | Pause | Stop

const greetMsg = ref("Ready");
const tempo = ref(0);
const current_beat = ref("0");
const status = ref("stopped")

async function play() {
  console.log("play");
  greetMsg.value = await invoke("play");
  status.value = "playing";
}

async function stop() {
  console.log("stop");
  greetMsg.value = await invoke("stop");
  status.value = "stopped";
}

async function pause() {
  console.log("pause");
  greetMsg.value = await invoke("pause");
  status.value = "paused";
}

async function get_tempo() {
  console.log("get_tempo");
  tempo.value = await invoke("get_tempo");
}

async function init() {
  await get_tempo();
  await get_track_list();
}

async function set_tempo(new_tempo: number) {
  console.log("set_tempo", new_tempo);
  if (new_tempo > 0 && new_tempo <= 240) {
    let response = await invoke("set_tempo", { tempo: new_tempo });
    console.log(response);
  }
  get_tempo();
}

async function get_track_list() {
  let list = await invoke("get_track_list");
  console.log(list);
}

onMounted(async () => {
  await init();
  listen<BeatSignal>('beat-signal', (event) => {
    console.log(event);
    if ('Pause' === event.payload) {
      // Do nothing
    } else if ('Stop' === event.payload) {
      current_beat.value = "0";
    } else {
      let frac = event.payload.Beat.frac;
      if ('NaN' === frac || 'Infinity' in frac) {
        current_beat.value = `WTF (${event.payload})`;
      } else {
        let [sign, [numer, denom]] = frac.Rational;
        current_beat.value = `${sign === 'Plus' ? '' : '-'}${numer}/${denom}`;
      }
    }
  });
});

</script>

<template>
  <el-container class="container">
    <el-header>
      <h1>Welcome to Step-Sequencer</h1>
    </el-header>

    <el-main>
      <MainControl :status="status" :tempo="tempo" :current_beat="current_beat" @play="play()" @pause="pause()"
        @stop="stop()" @update:tempo="set_tempo" />
      <TrackerView :current_beat="current_beat" />
      <!-- <div class="tracker-view">
              </div> -->
      <form class="row" @submit.prevent="true">
        <button @click="play" v-if="status != 'playing'">▶️</button>
        <button @click="pause" v-if="status == 'playing'">⏸️</button>
        <button @click="stop" :disabled="status == 'stopped'">⏹️</button>
      </form>
      <p>{{ greetMsg }}</p>
    </el-main>
    <el-footer>
      <el-row :gutter="10" justify="end" align="middle">
        <a href="https://github.com/y-usuzumi/step-sequencer">@ Github</a>
      </el-row>
    </el-footer>
  </el-container>
</template>

<style>
:root {
  color: darkslategray;
  /* background-color: mintcream; */
  /* font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%; */
}

html,
body {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
}

#app {
  height: 100%;
  overflow: hidden;
}

.container {
  width: 100%;
  height: 100%;
  margin: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;

}

.row {
  /* display: flex;
  justify-content: center; */
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

  button:active {
    background-color: #0f0f0f69;
  }
}
</style>