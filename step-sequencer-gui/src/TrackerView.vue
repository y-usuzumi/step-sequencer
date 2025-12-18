<script setup>
import { ref, computed } from 'vue'
import Track from './Track.vue';

const props = defineProps(['current_beat']);

const count = ref(2)
const computed_current_beat = computed({
    // Getter: 读取时，直接返回 props 的值
    get() {
        return intBeat(props.current_beat) ? intBeat(props.current_beat) : 1;
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
</script>

<template>
    <el-card shadow="never" body-style="padding: 0;">
        <el-scrollbar height="400px" view-style="padding: 10px;">
            <el-container>
                <el-header style="height: 40px; padding: 0;">
                    <!-- header -->
                    <el-space :size="20" alignment="flex-start">
                        <el-row :gutter="10" justify="start" align="middle">
                            <link rel="preconnect" href="https://fonts.googleapis.com">
                            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                            <link href="https://fonts.googleapis.com/css2?family=Oswald:wght@200..700&display=swap"
                                rel="stylesheet">
                            <el-text style="font-family: 'Oswald'; font-size: 20px;">
                                Current beat:
                            </el-text>
                        </el-row>
                        <el-radio-group v-model="computed_current_beat">
                            <el-space>
                                <el-radio-button type="primary" v-for="i in 20" :value="i" :disabled="false"
                                    style="border: none;" checked>{{
                                        i
                                    }}</el-radio-button>
                            </el-space>
                        </el-radio-group>
                    </el-space>
                </el-header>
                <el-container>
                    <el-aside width="min" style="padding: 10px;">
                        <el-row :gutter="10" v-for="i in 6">
                            <el-checkbox-button type="primary" circle>M</el-checkbox-button>
                            <el-checkbox-button type="primary" circle>S</el-checkbox-button>
                        </el-row>
                    </el-aside>
                    <el-container style="margin: 0;">
                        <el-main style="margin: 0; padding: 10px;">
                            <!-- main tracks -->
                            <Track v-for="i in 6" :current_beat="computed_current_beat" />
                        </el-main>
                    </el-container>
                </el-container>
            </el-container>
            <el-space :size="20" direction="vertical">


            </el-space>
        </el-scrollbar>
    </el-card>
    <!-- <div class="main-tracker-view" @click.prevent="">
    </div> -->
</template>

<style scoped>
el-checkbox-button {
    size: default;
}

.root {
    font-family: Arial;

    cursor: default;
    user-select: none;
}

.main-tracker-view {
    width: 100%;
    height: fit-content;
    min-height: 40vh;
    padding: 10px;
    margin: 10px;
    background-color: lightblue;

    display: flex;
    flex-direction: column;
    align-items: left;
    justify-content: left;
    gap: 10px;

    overflow: hidden;
}

@media (prefers-color-scheme: dark) {
    .main-tracker-view {
        background-color: darkcyan;
    }
}
</style>