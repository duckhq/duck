<template>
  <transition-group name="build" tag="div" class="grid md:grid-cols-1 gap-4 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
    <Build v-for="build in builds" :key="build.id" :build="build" />
  </transition-group>
</template>

<script>
import Build from "./Build.vue";

export default {
  props: ["builds"],
  components: {
    Build
  }
};
</script>

<style scoped>

</style>