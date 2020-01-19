<template>
  <b-card-group deck>
    <transition-group name="list-complete" tag="div" class="relative-parent">
      <build-card
        v-for="build in builds"
        :key="build.id"
        :build="build"
        class="list-complete-item mb-4"
      />
    </transition-group>
  </b-card-group>
</template>

<script>
import build from "./Build.vue";

export default {
  props: ["builds"],
  name: "App",
  components: {
    "build-card": build
  }
};
</script>

<style scoped>
.relative-parent {
  position: relative;
}
.list-complete-item {
  transition: all 1s;
  display: inline-block;
  width: 350px;
}
.list-complete-enter,
.list-complete-leave-to {
  opacity: 0;
  transform: translateY(30px);
}
.list-complete-leave-active {
  position: absolute;
}
</style>