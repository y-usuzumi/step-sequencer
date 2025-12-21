<script setup>
import { computed, ref, watch } from 'vue';
import Beat from './Beat.vue';

const props = defineProps(['current_beat']);

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

watch(() => props.current_beat, (newVal, oldVal) => {
    console.log(`${oldVal} -> ${newVal}`)
})
</script>


<template>
    <el-space :size="10">
        <!-- <el-checkbox-group v-model="computed_current_beat"> -->
        <Beat :isToggled="i === current_beat" :key="i" :value="i" :note="i" v-for="i in 20" />
        <!-- </el-checkbox-group> -->
    </el-space>
    <!-- <div class="main-track" @click.prevent="">
        <div >
        </div>
    </div> -->
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