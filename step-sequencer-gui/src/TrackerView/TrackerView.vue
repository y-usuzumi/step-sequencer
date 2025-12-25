<script setup>
import { ref, computed } from 'vue'
import Track from './Track.vue';
import '../assets/track.css'

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

const tracklist = ref([...Array(20).keys()].map(i => i + 1));

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

const onScroll = (e) => {
    console.log(e);
    let controlPanel = document.getElementById("control-panel");
    controlPanel.style.top = -e.scrollTop + "px";
}
</script>

<template>
    <el-card shadow="never" body-style="padding: 0;" style="margin: 2rem;">
        <!-- <el-container style="height: 100%; padding: 0.6rem;"> -->
        <!-- <el-scrollbar style="height:25rem; padding: 0.6rem;" height="100%" wrap-style="height: 25rem;" view-class=""> -->
        <el-splitter>
            <el-splitter-panel :size="20" collapsible class="no-overflow"
                style="overflow:hidden; width: 100%; height:25rem;">
                <!-- control panel -->
                <el-container id="control-panel" class="no-overflow simple-flex-center"
                    style="overflow:hidden;position: relative;">
                    <link rel="preconnect" href="https://fonts.googleapis.com">
                    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                    <link href="https://fonts.googleapis.com/css2?family=Oswald:wght@200..700&display=swap"
                        rel="stylesheet">
                    <!-- <el-text style="font-family: 'Oswald'; font-size: 1rem; position:sticky; top:0;">
                            Timeline:
                        </el-text> -->
                    <el-button type="" style="visibility: hidden;"></el-button> <!-- spacer -->
                    <el-row :gutter="10" v-for="i in tracklist" style="flex-wrap: nowrap">
                        <el-checkbox-button type="primary" circle>M</el-checkbox-button>
                        <el-checkbox-button type="primary" circle>S</el-checkbox-button>
                    </el-row>
                </el-container>
            </el-splitter-panel>
            <el-splitter-panel :size="90">
                <el-scrollbar height="25rem" wrap-style="100%" view-class="no-overflow simple-flex-start"
                    @scroll="onScroll">
                    <!-- timeline -->
                    <el-radio-group size="default" v-model="computed_current_beat" class="beats-start beat"
                        style="position: sticky; top:0;">
                        <el-radio-button type="primary" v-for="i in 20" :value="i">{{
                            i
                        }}</el-radio-button>
                    </el-radio-group>
                    <!-- main tracks -->
                    <Track v-for="i in tracklist" :current_beat="computed_current_beat" />
                </el-scrollbar>
            </el-splitter-panel>
        </el-splitter>
        <!-- </el-scrollbar> -->

        <!-- </el-container> -->
        <el-space :size="20" direction="vertical">


        </el-space>
    </el-card>
    <!-- <div class="main-tracker-view" @click.prevent="">
    </div> -->
</template>

<style scoped>
.root {
    font-family: Arial;

    cursor: default;
    user-select: none;
}

.main-tracker-view {
    width: 100%;
    height: fit-content;
    min-height: 40rem;
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