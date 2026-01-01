<template>
    <el-card shadow="never" body-style="padding: 0;" style="margin: 2rem;" body-class="simple-flex-start">
        <template #header>
            <p>Detail Panel</p>
        </template>
        <p>velocity: {{ velocity }}</p>
        <p>channel: {{ channel }}</p>
        <p>note</p>
    </el-card>
</template>

<script setup>
import { ref } from 'vue'
import "../assets/track.css"

const velocity = ref(0);
const channel = ref(0);


</script>

<style scoped></style>