<script setup>
import { ref, computed, watch } from 'vue'
import Track from './Track.vue';
import '../assets/track.css'
import ContextMenu from '@imengyu/vue3-context-menu'

const props = defineProps(['current_beat', 'add_empty_track', 'remove_track']);
const emits = defineEmits(['update:current_beat']);

const tracks = defineModel('tracks', { default: [] });

const text = ref('Hello, World!');
const computed_current_beat = computed({
    // Getter: 读取时，直接返回 props 的值
    get() {
        return intBeat(props.current_beat) ? intBeat(props.current_beat) % total_beats_num.value : 1;
    },
    // Setter: 写入时 (用户拖动滑块)，触发 emit 通知父组件
    set(new_current_beat) {
        emits('update:current_beat', new_current_beat);
    }
});

// const tracklist = ref([...Array(20).keys()].map(i => i + 1));

const total_beats_num = computed(() => {
    if (!props.tracks || !Array.isArray(props.tracks)) return 16;
    let t = 16;
    for (const track of props.tracks) {
        if (track[1].beats.length > t) {
            t = track[1].beats.length;
        }
    }
    return t;
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

const onScroll = (e) => {
    // console.log(e);
    let controlPanel = document.getElementById("control-panel");
    controlPanel.style.top = -e.scrollTop + "px";
}

const refTrackPanel = ref(null);
const trackPanelSize = ref(null);

const onControlPanelContextMenu = (e) => {
    e.preventDefault();
    console.log('right click', e);
    ContextMenu.showContextMenu({
        x: e.x,
        y: e.y,
        // 菜单样式主题，可选 'default', 'dark' 等
        // theme: 'default',
        items: [
            {
                label: "add an empty track ",
                onClick: () => props.add_empty_track()
            },
        ]
    });
};
const onTrackContextMenu = (e, id, idx) => {
    e.preventDefault();
    console.log('right click', e, id, idx);
    ContextMenu.showContextMenu({
        x: e.x,
        y: e.y,
        // 菜单样式主题，可选 'default', 'dark' 等
        // theme: 'default',
        items: [
            {
                label: "add an empty track ",
                onClick: () => props.add_empty_track()
            },
            {
                label: "delete track",
                // onClick: () => console.log(idx)
                onClick: () => props.remove_track(idx)
            },
            // {
            //     label: "Delete",
            //     // 可以直接用简单的 string icon，也可以支持 h() 渲染图标
            //     icon: "icon-trash",
            //     onClick: () => console.log("Delete clicked")
            // },
            // {
            //     label: "Sub Menu", children: [
            //         { label: "Item 1" },
            //         { label: "Item 2" }
            //     ]
            // }
        ]
    });
};


watch(() => computed_current_beat.value, () => {
    // console.log('current_beat changed')
    // console.log(refTrackPanel)
    refTrackPanel.value.setScrollLeft((computed_current_beat.value - 3) * 50)
})

watch(() => tracks.value, () => {
    console.log('tracks changed', tracks.value)
})
</script>

<template>
    <el-card shadow="never" body-style="padding: 0;" style="margin: 2rem;">
        <!-- <el-container style="height: 100%; padding: 0.6rem;"> -->
        <!-- <el-scrollbar style="height:25rem; padding: 0.6rem;" height="100%" wrap-style="height: 25rem;" view-class=""> -->
        <el-splitter>
            <el-splitter-panel size="200px" collapsible class="no-overflow"
                style="overflow:hidden; width: 100%; height:25rem;">
                <!-- control panel -->
                <el-container id="control-panel" class="no-overflow simple-flex-center"
                    style="overflow:hidden;position: relative;">

                    <!-- <el-text style="font-family: 'Oswald'; font-size: 1rem; position:sticky; top:0;">
                            Timeline:
                        </el-text> -->
                    <el-button style="visibility: hidden;"></el-button> <!-- spacer -->
                    <section v-for="([id, track], idx) in tracks" style="width: 100%;" :id="id" :idx="idx"
                        @contextmenu="(e) => onTrackContextMenu(e, id, idx)">
                        <el-row :gutter="10" style="flex-wrap: nowrap;" class="simple-flex-end">
                            <el-text type="info"
                                style="display: inline-block; text-wrap: none; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: 'Oswald'; font-size: 1rem;">{{
                                    track.name }}</el-text>
                            <!-- <el-input v-model="text" placeholder=""></el-input> -->
                            <section>
                                <el-checkbox-button type="primary" circle>M</el-checkbox-button>
                                <el-checkbox-button type="primary" circle>S</el-checkbox-button>
                            </section>
                        </el-row>
                    </section>
                </el-container>
            </el-splitter-panel>
            <el-splitter-panel :size="trackPanelSize">
                <el-scrollbar ref="refTrackPanel" height="25rem" wrap-style="100%"
                    view-class="no-overflow simple-flex-start" @scroll="onScroll">
                    <!-- timeline -->
                    <el-radio-group size="default" v-model="computed_current_beat" class="beats-start beat"
                        style="position: sticky; top:0;">
                        <el-radio-button type="primary" v-for="i in total_beats_num" :value="i">{{
                            i
                        }}</el-radio-button>
                    </el-radio-group>
                    <!-- main tracks -->
                    <Track v-for="([id, track], idx) in tracks" :id="id" :idx="idx" :beats="track.beats"
                        :default_beat="track.default_beat" :tempo_scale="track.tempo_scale"
                        :current_beat="computed_current_beat" @contextmenu="(e) => onTrackContextMenu(e, id, idx)" />
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