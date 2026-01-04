<script setup>
import { computed, ref, watch } from 'vue';
import Beat from './Beat.vue';
import '../assets/track.css'

const props = defineProps(['current_beat', 'default_beat']);
const beats = defineModel('beats', { default: [] });
const tempo_scale = defineModel('tempo_scale', { default: 120 });
const note = (beat) => {
    if (beat === 'DefaultBeat') {
        let n = props.default_beat.note;
        return n.pitch_class.replace('s', '#').replace('f', 'b') + n.octave;
    } else if (beat === 'Unset') {
        return '---';
    } else {
        return beat.OverrideBeat.map(n => n.note.pitch_class.replace('s', '#').replace('f', 'b') + n.note.octave).toString()
    }
}
// const isToggled = ref(false)
// console.log(props.current_beat)

const computed_current_beat = computed({
    // Getter: 读取时，直接返回 props 的值
    get() {
        return props.current_beat;
    },
    // Setter: 写入时 (用户拖动滑块)，触发 emit 通知父组件
    set(new_current_beat) {
        emits('update:current_beat', new_current_beat);
    }
});

// watch(() => props.current_beat, (newVal, oldVal) => {
//     console.log(`${oldVal} -> ${newVal}`)
// })


</script>


<template>
    <section class="beats-start">
        <Beat :isToggled="i + 1 === current_beat" :key="i" :value="i" :note="note(beat)" v-for="(beat, i) in beats" />
    </section>
</template>


<style scoped>
.main-track {
    width: 100%;
    height: 5vh;
    padding-left: 5px;
    margin: 3px;
    background-color: lightcyan;

    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: left;

    cursor: default;
    user-select: none;
}

@media (prefers-color-scheme: dark) {
    .main-track {
        background-color: lightblue;
    }
}
</style>