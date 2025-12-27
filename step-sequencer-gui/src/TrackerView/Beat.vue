<script setup>
import { id } from 'element-plus/es/locales.mjs';
import { ref, defineProps, defineEmits, computed, onMounted, watch } from 'vue';
import '../assets/track.css'
const props = defineProps(['id', 'value', 'isToggled', 'note']);
const emit = defineEmits(['update:isToggled', 'update:note']);
const computed_isToggled = computed({
    get() {
        return props.isToggled;
    },
    set(value) {
        emit('updated:isToggled', value);
    }
});
// const note = ref('C1');
// function onClick() {
//     computed_isToggled.value = !computed_isToggled.value;
//     console.log(computed_isToggled);
// }

// watch(() => props.isToggled, (newVal, oldVal) => {
//     console.log(newVal, '=>', oldVal);
// })

</script>
<template>
    <el-checkbox-button class="beat" type="primary" :value="value" v-model="computed_isToggled">{{ note
        }}</el-checkbox-button>
</template>


<style scoped>
.main-beat {
    width: 4vh;
    height: 3.5vh;
    border: 1px solid;
    box-shadow: 3px;
    margin: 5px;
    color: darkslategray;
    background-color: azure;

    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;

    cursor: default;
    user-select: pointer;
}

.toggled {
    background-color: aquamarine;
}

@media (prefers-color-scheme: dark) {
    .main-beat {
        color: black;
        background-color: aquamarine;
    }

    .toggled {
        background-color: lightseagreen;
    }
}
</style>