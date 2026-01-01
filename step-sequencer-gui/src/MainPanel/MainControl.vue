<script setup>
import { onMounted, ref, computed } from 'vue';
import PlayArrowIcon from '@material-design-icons/svg/outlined/play_arrow.svg'
import PauseIcon from '@material-design-icons/svg/outlined/pause.svg'
import StopIcon from '@material-design-icons/svg/outlined/stop.svg'

const props = defineProps(['status', 'current_beat', 'play', 'pause', 'stop', 'total_beats_num']);
const emits = defineEmits(['update:status', 'update:current_beat', 'update:tempo']);

const tempo = defineModel('tempo');

const computed_status = computed({
    // Getter: 读取时，直接返回 props 的值
    get() {
        return props.status;
    },
    // Setter: 写入时 (用户拖动滑块)，触发 emit 通知父组件
    set(new_status) {
        emits('update:status', new_status);
    }
});
const computed_tempo = computed({
    // Getter: 读取时，直接返回 props 的值
    get() {
        return props.tempo;
    },
    // Setter: 写入时 (用户拖动滑块)，触发 emit 通知父组件
    set(new_tempo) {
        emits('update:tempo', new_tempo);
    }
});
const computed_current_beat = computed({
    // Getter: 读取时，直接返回 props 的值
    get() {
        return intBeat(props.current_beat) ? intBeat(props.current_beat) % props.total_beats_num : 1;
    },
    // Setter: 写入时 (用户拖动滑块)，触发 emit 通知父组件
    set(new_current_beat) {
        emits('update:current_beat', new_current_beat);
    }
});

function intBeat(current_beat) {
    const parts = current_beat.split('/');
    if (parts.length === 2) {
        const numerator = parseFloat(parts[0]);
        const denominator = parseFloat(parts[1]);

        const result = numerator / denominator;
        return parseInt(result) + 1;
    } else {
        return NaN;
    }
}

const handlePlay = () => {
    emits('play');
}
const handlePause = () => {
    emits('pause');
}
const handleStop = () => {
    emits('stop');
}

onMounted(() => {
    console.log('MainControl mounted');
    console.log('status', props.status);
    console.log('tempo', typeof (props.tempo));
    console.log('current_beat', props.current_beat);
});
</script>

<template>
    <el-row type="flex" align="middle" justify="start" style="margin: 2rem;">
        <el-col :span="3">
            <el-button type="default" v-if="computed_status != 'playing'" @click="handlePlay"
                style="color: var(--el-text-color)">
                <PlayArrowIcon />
            </el-button>
            <el-button type="default" v-else-if="computed_status == 'playing'" @click="handlePause">
                <PauseIcon />
            </el-button>
            <el-button type="default" :disabled="computed_status == 'stopped'" @click="handleStop">
                <StopIcon />
            </el-button>
        </el-col>
        <el-col :span="9">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=Oswald:wght@200..700&display=swap" rel="stylesheet">
            <el-space :size="30">
                <el-text style="font-family: 'Oswald'; font-size: 20px;">

                    Tempo:
                    <el-input-number v-model="computed_tempo" :step="1">
                        <template #suffix>
                            <span>Bps</span>
                        </template>
                    </el-input-number>
                </el-text>
                <el-text style="font-family: 'Oswald'; font-size: 20px;">
                    Current beat: {{
                        computed_current_beat }}
                </el-text>
            </el-space>
        </el-col>
        <el-col :span="12">
            <el-slider v-model="computed_current_beat" :min="1" :max="total_beats_num" :step="1"></el-slider>
        </el-col>
    </el-row>
</template>


<style scoped>
.app-main {
    padding: 0;
    /* el-main 默认有 20px padding，这里去掉以便让 scrollbar 贴边 */
    overflow: hidden;
    /* 禁止 el-main 自身的滚动，交给内部的 el-scrollbar */
}

/* Make SVG icons inherit Element Plus theme colors */
:deep(svg) {
    fill: var(--el-text-color-primary);
}

/* Button hover state */
.el-button:hover :deep(svg) {
    fill: var(--el-color-primary);
}
</style>